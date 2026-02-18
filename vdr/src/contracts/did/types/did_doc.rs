// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{
    contracts::did::types::did::{DID, DID_URL_REGEX, RELATIVE_DID_URL_REGEX},
    error::{VdrError, VdrResult},
    types::{ContractOutput, ContractParam},
    utils::is_unique,
    Address, Block,
};

use log::warn;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;

pub const BASE_CONTEXT: &str = "https://www.w3.org/ns/did/v1";
pub const SECPK_CONTEXT: &str = "https://w3id.org/security/suites/secp256k1recovery-2020/v2";
pub const KEYS_CONTEXT: &str = "https://w3id.org/security/v3-unstable";
pub const DID_RESOLUTION_FORMAT: &str = "application/did+ld+json";

/// DID Document with associate metadata
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DidDocumentWithMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub did_document: Option<DidDocument>,
    pub did_document_metadata: DidMetadata,
    pub did_resolution_metadata: DidResolutionMetadata,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DidResolutionMetadata {
    pub content_type: Option<String>,
    pub error: Option<DidResolutionError>,
    pub message: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DidResolutionOptions {
    pub accept: Option<String>,
    pub block_tag: Option<Block>,
}

/// Definition of DID Document object matching to the specification - `<https://www.w3.org/TR/did-core/>`
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DidDocument {
    #[serde(rename = "@context")]
    pub context: StringOrVector,
    pub id: DID,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub controller: Option<StringOrVector>,
    pub verification_method: Vec<VerificationMethod>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub authentication: Vec<VerificationMethodOrReference>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub assertion_method: Vec<VerificationMethodOrReference>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub capability_invocation: Vec<VerificationMethodOrReference>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub capability_delegation: Vec<VerificationMethodOrReference>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub key_agreement: Vec<VerificationMethodOrReference>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub service: Vec<Service>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub also_known_as: Option<Vec<String>>,
}

impl DidDocument {
    pub(crate) fn validate(&self) -> VdrResult<()> {
        self.id.validate()?;

        // Validate verification methods
        for verification_method in &self.verification_method {
            verification_method.validate(&self.id)?;
        }

        let verification_relationships = self
            .assertion_method
            .iter()
            .chain(self.authentication.iter())
            .chain(self.capability_delegation.iter())
            .chain(self.capability_invocation.iter())
            .chain(self.key_agreement.iter())
            .collect::<Vec<_>>();

        // Validate verification relationships
        verification_relationships
            .iter()
            .try_for_each(|relationship| {
                relationship.validate(&self.id, &self.verification_method)
            })?;

        // Check for unique verification method IDs
        let verification_method_ids = verification_relationships
            .iter()
            .filter_map(|relationship| {
                if let VerificationMethodOrReference::VerificationMethod(vm) = relationship {
                    Some(vm)
                } else {
                    None
                }
            })
            .chain(self.verification_method.iter())
            .map(|vm| &vm.id);

        if !is_unique(verification_method_ids) {
            return Err(VdrError::InvalidDidDocument(
                "Verification method ID must be unique".to_string(),
            ));
        }

        // Check for unique service IDs
        let service_ids = self.service.iter().map(|value| &value.id);
        if !is_unique(service_ids) {
            return Err(VdrError::InvalidDidDocument(
                "Service ID must be unique".to_string(),
            ));
        }

        // Validate services
        for service in self.service.iter() {
            service.validate(&self.id)?;
        }

        Ok(())
    }
}

/// DID Record stored in the IndyBesu DID Registry
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DidRecord {
    pub document: DidDocument,
    pub metadata: DidMetadata,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DidMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deactivated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_version_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_update: Option<u64>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationMethod {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: VerificationKeyType,
    pub controller: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blockchain_account_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key_multibase: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key_hex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key_base58: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key_base64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key_jwk: Option<String>,
}

impl VerificationMethod {
    fn validate(&self, did: &DID) -> VdrResult<()> {
        if !self.id.starts_with(did.as_ref()) {
            return Err(VdrError::InvalidDidDocument(format!(
                "Invalid verefication method ID: {}",
                self.id
            )));
        }

        DID::from(self.controller.as_ref())
            .validate()
            .map_err(|_| {
                VdrError::InvalidDidDocument(format!(
                    "Invalid controller syntax in the verification method: {}",
                    json!(self).to_string()
                ))
            })?;

        let key_materials = [
            &self.blockchain_account_id,
            &self.public_key_multibase,
            &self.public_key_hex,
            &self.public_key_base58,
            &self.public_key_base64,
            &self.public_key_jwk,
        ];

        let key_materials_count = key_materials.iter().filter(|key| key.is_some()).count();

        if key_materials_count == 0 {
            return Err(VdrError::InvalidDidDocument(format!(
                "No public key was found for the verification method with ID: {}",
                self.id
            )));
        }

        if key_materials_count > 1 {
            return Err(VdrError::InvalidDidDocument(format!(
                "Multiple public keys detected in the verification method with ID: {}",
                self.id
            )));
        }

        Ok(())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub enum VerificationKeyType {
    #[default]
    Ed25519VerificationKey2018,
    X25519KeyAgreementKey2019,
    Ed25519VerificationKey2020,
    X25519KeyAgreementKey2020,
    JsonWebKey2020,
    EcdsaSecp256k1VerificationKey2019,
    EcdsaSecp256k1VerificationKey2020,
    EcdsaSecp256k1RecoveryMethod2020,
}

impl ToString for VerificationKeyType {
    fn to_string(&self) -> String {
        match self {
            VerificationKeyType::Ed25519VerificationKey2018 => {
                "Ed25519VerificationKey2018".to_string()
            }
            VerificationKeyType::X25519KeyAgreementKey2019 => {
                "X25519KeyAgreementKey2019".to_string()
            }
            VerificationKeyType::Ed25519VerificationKey2020 => {
                "Ed25519VerificationKey2020".to_string()
            }
            VerificationKeyType::X25519KeyAgreementKey2020 => {
                "X25519KeyAgreementKey2020".to_string()
            }
            VerificationKeyType::JsonWebKey2020 => "JsonWebKey2020".to_string(),
            VerificationKeyType::EcdsaSecp256k1VerificationKey2019
            | VerificationKeyType::EcdsaSecp256k1VerificationKey2020 => {
                "EcdsaSecp256k1VerificationKey2019".to_string()
            }
            VerificationKeyType::EcdsaSecp256k1RecoveryMethod2020 => {
                "EcdsaSecp256k1RecoveryMethod2020".to_string()
            }
        }
    }
}

impl TryFrom<&str> for VerificationKeyType {
    type Error = VdrError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Ed25519VerificationKey2018" => Ok(VerificationKeyType::Ed25519VerificationKey2018),
            "X25519KeyAgreementKey2019" => Ok(VerificationKeyType::X25519KeyAgreementKey2019),
            "Ed25519VerificationKey2020" => Ok(VerificationKeyType::Ed25519VerificationKey2020),
            "X25519KeyAgreementKey2020" => Ok(VerificationKeyType::X25519KeyAgreementKey2020),
            "JsonWebKey2020" => Ok(VerificationKeyType::JsonWebKey2020),
            "EcdsaSecp256k1RecoveryMethod2020" => {
                Ok(VerificationKeyType::EcdsaSecp256k1RecoveryMethod2020)
            }
            "EcdsaSecp256k1VerificationKey2019" => {
                Ok(VerificationKeyType::EcdsaSecp256k1VerificationKey2020)
            }
            _type => Err({
                let vdr_error = VdrError::CommonInvalidData(format!(
                    "Unexpected verification key type {}",
                    _type
                ));

                warn!(
                    "Error: {} during converting VerificationKeyType from String: {} ",
                    vdr_error, value
                );

                vdr_error
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum VerificationMethodOrReference {
    String(String),
    VerificationMethod(VerificationMethod),
}

impl VerificationMethodOrReference {
    fn validate(&self, did: &DID, verification_methods: &[VerificationMethod]) -> VdrResult<()> {
        match &self {
            VerificationMethodOrReference::VerificationMethod(verification_method) => {
                verification_method.validate(did)?;
            }
            VerificationMethodOrReference::String(verification_reference) => {
                Self::validate_verification_reference(
                    did.as_ref(),
                    verification_reference,
                    verification_methods,
                )?;
            }
        };

        Ok(())
    }

    fn validate_verification_reference(
        did: &str,
        verification_reference: &str,
        verification_methods: &[VerificationMethod],
    ) -> VdrResult<()> {
        let full_reference = if RELATIVE_DID_URL_REGEX.is_match(verification_reference) {
            format!("{}{}", did, verification_reference)
        } else {
            verification_reference.to_string()
        };

        if full_reference.starts_with(did) {
            let exist = verification_methods
                .iter()
                .any(|vm| vm.id == full_reference);
            if !exist {
                return Err(VdrError::InvalidDidDocument(format!(
                    "Verification method not found for reference ID: {verification_reference}",
                )));
            }
        } else if !DID_URL_REGEX.is_match(verification_reference.as_ref()) {
            return Err(VdrError::InvalidDidDocument(format!(
                "Invalid verification reference ID: {verification_reference}"
            )));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: ServiceType,
    pub service_endpoint: ServiceEndpoint,
}

impl Service {
    fn validate(&self, did: &DID) -> VdrResult<()> {
        if !(self.id.starts_with(did.as_ref())) {
            return Err(VdrError::InvalidDidDocument(format!(
                "Invalid service ID: {}",
                self.id
            )));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum ServiceType {
    LinkedDomains,
    DIDCommMessaging,
    CredentialRegistry,
    OID4VCI,
    OID4VP,
}

impl ToString for ServiceType {
    fn to_string(&self) -> String {
        match self {
            ServiceType::LinkedDomains => "LinkedDomains".to_string(),
            ServiceType::DIDCommMessaging => "DIDCommMessaging".to_string(),
            ServiceType::CredentialRegistry => "CredentialRegistry".to_string(),
            ServiceType::OID4VCI => "OID4VCI".to_string(),
            ServiceType::OID4VP => "OID4VP".to_string(),
        }
    }
}

impl TryFrom<&str> for ServiceType {
    type Error = VdrError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "LinkedDomains" => Ok(ServiceType::LinkedDomains),
            "DIDCommMessaging" => Ok(ServiceType::DIDCommMessaging),
            "CredentialRegistry" => Ok(ServiceType::CredentialRegistry),
            "OID4VCI" => Ok(ServiceType::OID4VCI),
            "OID4VP" => Ok(ServiceType::OID4VP),
            _type => Err({
                let vdr_error =
                    VdrError::CommonInvalidData(format!("Unexpected service type {}", _type));

                warn!(
                    "Error: {} during converting ServiceType from String: {} ",
                    vdr_error, value
                );

                vdr_error
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ServiceEndpoint {
    String(String),
    Object(ServiceEndpointObject),
    Set(Vec<ServiceEndpoint>),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceEndpointObject {
    pub uri: String,
    pub accept: Vec<String>,
    pub routing_keys: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum StringOrVector {
    String(String),
    Vector(Vec<String>),
}

impl Default for StringOrVector {
    fn default() -> Self {
        StringOrVector::Vector(Vec::new())
    }
}

impl TryFrom<&DidDocument> for ContractParam {
    type Error = VdrError;

    fn try_from(value: &DidDocument) -> Result<Self, Self::Error> {
        serde_json::to_vec(value)
            .map(ContractParam::Bytes)
            .map_err(|_| VdrError::ContractInvalidInputData)
    }
}

impl TryFrom<&ContractOutput> for DidDocument {
    type Error = VdrError;

    fn try_from(value: &ContractOutput) -> Result<Self, Self::Error> {
        serde_json::from_slice(&value.get_bytes(0)?).map_err(|err| {
            VdrError::ContractInvalidResponseData(format!(
                "Unable to parse DID Document from the response. Err: {:?}",
                err
            ))
        })
    }
}

impl TryFrom<ContractOutput> for DidMetadata {
    type Error = VdrError;

    fn try_from(value: ContractOutput) -> Result<Self, Self::Error> {
        let owner = value.get_address(0)?;
        let created = value.get_u128(1)? as u64;
        let updated = value.get_u128(2)? as u64;
        let version_id = value.get_u128(3)? as u64;
        let deactivated = value.get_bool(4)?;
        let did_metadata = DidMetadata {
            owner: Some(owner),
            deactivated: Some(deactivated),
            created: Some(created),
            version_id: Some(version_id),
            updated: Some(updated),
            next_version_id: None,
            next_update: None,
        };
        Ok(did_metadata)
    }
}

impl TryFrom<ContractOutput> for DidRecord {
    type Error = VdrError;

    fn try_from(value: ContractOutput) -> Result<Self, Self::Error> {
        let output_tuple = value.get_tuple(0)?;
        let did_document = DidDocument::try_from(&output_tuple)?;
        let metadata = output_tuple.get_tuple(1)?;
        let did_doc_with_metadata = DidRecord {
            document: did_document,
            metadata: DidMetadata::try_from(metadata)?,
        };
        Ok(did_doc_with_metadata)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum DidResolutionError {
    /*
     * The resolver has failed to construct the DID document.
     * This can be caused by a network issue, a wrong registry address or malformed logs while parsing the registry
     * history. Please inspect the `DIDResolutionMetadata.message` to debug further.
     */
    #[serde(rename = "notFound")]
    NotFound,
    /*
     * The resolver does not know how to resolve the given DID. Most likely it is not a `did:ethr`.
     */
    #[serde(rename = "invalidDid")]
    InvalidDid,
    /*
     * The resolver does not support the 'accept' format requested with `DidResolutionOptions`.
     */
    #[serde(rename = "representationNotSupported")]
    RepresentationNotSupported,
    /*
     * If an invalid DID URL is detected
     */
    #[serde(rename = "invalidDidUrl")]
    InvalidDidUrl,
    /*
     * DID method is not supported
     */
    #[serde(rename = "methodNotSupported")]
    MethodNotSupported,
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::{contracts::ETHR_DID_METHOD, did_indy_registry::INDYBESU_DID_METHOD};

    pub const TEST_IDENTITY: &str = "0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5";
    pub const TEST_INDYBESU_DID: &str = "did:indybesu:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5";
    pub const TEST_ETHR_DID: &str = "did:ethr:testnet:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5";
    pub const TEST_ETHR_DID_WITHOUT_NETWORK: &str =
        "did:ethr:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5";
    pub const SERVICE_ENDPOINT: &str = "http://example.com";
    pub const MULTIBASE_KEY: &'static str = "zAKJP3f7BD6W4iWEQ9jwndVTCBq8ua2Utt8EEjJ6Vxsf";
    pub const BASE58_KEY: &'static str = "H3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV";
    pub const KEY_1: &'static str = "KEY-1";

    pub fn service(id: &str) -> Service {
        Service {
            id: id.to_string(),
            type_: ServiceType::LinkedDomains,
            service_endpoint: ServiceEndpoint::String(SERVICE_ENDPOINT.to_string()),
        }
    }

    pub fn verification_method(id: &str) -> VerificationMethod {
        let (controller, _) = id.split_once('#').unwrap_or_default();
        VerificationMethod {
            id: id.to_string(),
            type_: VerificationKeyType::Ed25519VerificationKey2018,
            controller: controller.to_string(),
            blockchain_account_id: None,
            public_key_multibase: Some(MULTIBASE_KEY.to_string()),
            public_key_hex: None,
            public_key_base58: None,
            public_key_base64: None,
            public_key_jwk: None,
        }
    }

    pub fn verification_relationship(id: &str) -> VerificationMethodOrReference {
        VerificationMethodOrReference::String(id.to_string())
    }

    pub fn did_doc(identity: &str) -> DidDocument {
        let id = DID::build(INDYBESU_DID_METHOD, None, identity);
        let kid = format!("{}#{}", id.as_ref(), KEY_1);
        DidDocument {
            context: StringOrVector::Vector(vec![BASE_CONTEXT.to_string()]),
            id: id.clone(),
            controller: None,
            verification_method: vec![verification_method(&kid)],
            authentication: vec![verification_relationship(&kid)],
            assertion_method: vec![],
            capability_invocation: vec![],
            capability_delegation: vec![],
            key_agreement: vec![],
            service: vec![],
            also_known_as: None,
        }
    }

    pub fn default_ethr_did_document(identity: &str, chain_id: Option<u64>) -> DidDocument {
        let id = DID::build(ETHR_DID_METHOD, None, identity);
        let chain_id = chain_id.unwrap_or(1);
        let blockchain_account_id = format!("eip155:{}:{}", chain_id, identity);
        let kid = format!("{}#controller", id.as_ref());
        DidDocument {
            context: StringOrVector::Vector(vec![
                BASE_CONTEXT.to_string(),
                SECPK_CONTEXT.to_string(),
                KEYS_CONTEXT.to_string(),
            ]),
            id: id.clone(),
            controller: None,
            verification_method: vec![VerificationMethod {
                id: kid.clone(),
                type_: VerificationKeyType::EcdsaSecp256k1RecoveryMethod2020,
                controller: id.to_string(),
                blockchain_account_id: Some(blockchain_account_id),
                public_key_multibase: None,
                public_key_hex: None,
                public_key_base58: None,
                public_key_base64: None,
                public_key_jwk: None,
            }],
            authentication: vec![verification_relationship(&kid)],
            assertion_method: vec![verification_relationship(&kid)],
            capability_invocation: vec![],
            capability_delegation: vec![],
            key_agreement: vec![],
            service: vec![],
            also_known_as: None,
        }
    }

    fn did_doc_param() -> ContractParam {
        ContractParam::Bytes(serde_json::to_vec(&did_doc(TEST_IDENTITY)).unwrap())
    }

    mod validate_did_doc {
        use super::*;

        #[test]
        fn invalid_did_syntax() {
            let did_document = did_doc("did:test:example");

            let expected_error = VdrError::InvalidDidDocument(format!(
                "Incorrect DID: {}",
                did_document.id.as_ref()
            ));

            let actual_error = did_document.validate().unwrap_err();

            assert_eq!(actual_error, expected_error);
        }

        #[test]
        fn invalid_verification_method_id() {
            let mut did_document = did_doc(TEST_IDENTITY);
            did_document.verification_method[0].id = "did:test:example#test".to_string();

            let expected_error = VdrError::InvalidDidDocument(format!(
                "Invalid verefication method ID: did:test:example#test",
            ));

            let actual_error = did_document.validate().unwrap_err();

            assert_eq!(actual_error, expected_error);
        }

        #[test]
        fn invalid_verification_method_controller_syntax() {
            let mut did_document = did_doc(TEST_IDENTITY);
            did_document.verification_method[0].controller = "test".to_string();

            let expected_error = VdrError::InvalidDidDocument(format!(
                "Invalid controller syntax in the verification method: {}",
                json!(did_document.verification_method[0])
            ));

            let actual_error = did_document.validate().unwrap_err();

            assert_eq!(actual_error, expected_error);
        }

        #[test]
        fn invalid_verification_reference() {
            let mut did_document = did_doc(TEST_IDENTITY);
            did_document
                .authentication
                .push(verification_relationship("did:test:example#test"));

            let expected_error = VdrError::InvalidDidDocument(format!(
                "Invalid verification reference ID: did:test:example#test",
            ));

            let actual_error = did_document.validate().unwrap_err();

            assert_eq!(actual_error, expected_error);
        }

        #[test]
        fn nonexistent_verification_method() {
            let mut did_document = did_doc(TEST_IDENTITY);
            let key_id = format!("{}#{}", TEST_INDYBESU_DID, "key-2");
            did_document
                .authentication
                .push(verification_relationship(&key_id));

            let expected_error = VdrError::InvalidDidDocument(format!(
                "Verification method not found for reference ID: {key_id}"
            ));

            let actual_error = did_document.validate().unwrap_err();

            assert_eq!(actual_error, expected_error);
        }

        #[test]
        fn duplicate_verification_method_id() {
            let mut did_document = did_doc(TEST_IDENTITY);
            did_document
                .authentication
                .push(VerificationMethodOrReference::VerificationMethod(
                    verification_method(&did_document.verification_method[0].id),
                ));

            let expected_error =
                VdrError::InvalidDidDocument("Verification method ID must be unique".to_string());

            let actual_error = did_document.validate().unwrap_err();

            assert_eq!(actual_error, expected_error);
        }

        #[test]
        fn verification_method_without_key_material() {
            let mut did_document = did_doc(TEST_IDENTITY);
            did_document.verification_method[0].public_key_multibase = None;

            let expected_error = VdrError::InvalidDidDocument(format!(
                "No public key was found for the verification method with ID: {}",
                &did_document.verification_method[0].id
            ));

            let actual_error = did_document.validate().unwrap_err();

            assert_eq!(actual_error, expected_error);
        }

        #[test]
        fn verification_method_with_multiple_key_materials() {
            let mut did_document = did_doc(TEST_IDENTITY);
            did_document.verification_method[0].public_key_base58 = Some(BASE58_KEY.to_string());

            let expected_error = VdrError::InvalidDidDocument(format!(
                "Multiple public keys detected in the verification method with ID: {}",
                &did_document.verification_method[0].id
            ));

            let actual_error = did_document.validate().unwrap_err();

            assert_eq!(actual_error, expected_error);
        }

        #[test]
        fn invalid_service_id() {
            let mut did_document = did_doc(TEST_IDENTITY);
            did_document.service = vec![service("test")];

            let expected_error =
                VdrError::InvalidDidDocument("Invalid service ID: test".to_string());

            let actual_error = did_document.validate().unwrap_err();

            assert_eq!(actual_error, expected_error);
        }

        #[test]
        fn duplicate_service_id() {
            let mut did_document = did_doc(TEST_IDENTITY);
            let service_id = format!("{}#{}", TEST_INDYBESU_DID, "service");
            did_document.service = vec![service(&service_id), service(&service_id)];

            let expected_error =
                VdrError::InvalidDidDocument("Service ID must be unique".to_string());

            let actual_error = did_document.validate().unwrap_err();

            assert_eq!(actual_error, expected_error);
        }

        #[test]
        fn valid_did_document() {
            let mut did_document = did_doc(TEST_IDENTITY);
            let kid = format!("#{}", KEY_1);
            did_document.assertion_method = vec![verification_relationship(&kid)];

            let service_id = format!("{}#{}", TEST_INDYBESU_DID, "service");
            did_document.service = vec![service(&service_id)];

            let result = did_document.validate();

            assert_eq!(result, Ok(()));
        }
    }

    mod convert_into_contract_param {
        use super::*;

        #[test]
        fn convert_did_doc_into_contract_param_test() {
            let param: ContractParam = (&did_doc(TEST_IDENTITY)).try_into().unwrap();
            assert_eq!(did_doc_param(), param);
        }
    }

    mod convert_into_object {
        use super::*;

        #[test]
        fn convert_contract_output_into_did_doc() {
            let data = ContractOutput::new(vec![did_doc_param()]);
            let converted = DidDocument::try_from(&data).unwrap();
            assert_eq!(did_doc(TEST_IDENTITY), converted);
        }
    }
}
