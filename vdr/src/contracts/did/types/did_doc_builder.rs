// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use log::warn;
use log_derive::{logfn, logfn_inputs};

use crate::{
    contracts::{
        did::{
            types::did_doc::{
                Service, ServiceEndpoint, ServiceType, StringOrVector, VerificationMethod,
                VerificationMethodOrReference, BASE_CONTEXT,
            },
            KEYS_CONTEXT, SECPK_CONTEXT,
        },
        types::did::ParsedDid,
    },
    error::{VdrError, VdrResult},
    Address, DidDocument, VerificationKeyType, DID,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DidDocumentBuilder {
    context: StringOrVector,
    id: DID,
    controller: Option<StringOrVector>,
    verification_method: Vec<(String, VerificationMethod)>,
    authentication: Vec<(String, VerificationMethodOrReference)>,
    assertion_method: Vec<(String, VerificationMethodOrReference)>,
    capability_invocation: Vec<(String, VerificationMethodOrReference)>,
    capability_delegation: Vec<(String, VerificationMethodOrReference)>,
    key_agreement: Vec<(String, VerificationMethodOrReference)>,
    service: Vec<(String, Service)>,
    also_known_as: Option<Vec<String>>,
    deactivated: bool,
    key_index: u32,
    service_index: u32,
}

impl DidDocumentBuilder {
    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub fn new() -> DidDocumentBuilder {
        DidDocumentBuilder {
            context: StringOrVector::String(BASE_CONTEXT.to_string()),
            ..DidDocumentBuilder::default()
        }
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub fn base_for_did(did: &DID, chain_id: u64) -> VdrResult<DidDocumentBuilder> {
        let parsed_did = ParsedDid::try_from(did)?;
        let did = parsed_did.as_short_did();
        let identity = Address::try_from(&did)?;
        let kid = "controller";
        let id = format!("{}#controller", did.as_ref());

        let mut did_doc_builder = DidDocumentBuilder::new();
        did_doc_builder.add_context(SECPK_CONTEXT);
        did_doc_builder.add_context(KEYS_CONTEXT);
        did_doc_builder.set_id(&did);
        did_doc_builder.add_verification_method(
            kid,
            &id,
            &VerificationKeyType::EcdsaSecp256k1RecoveryMethod2020,
            Some(identity.as_blockchain_id(chain_id).as_str()),
            None,
            None,
            None,
            None,
            None,
        );
        did_doc_builder.add_authentication_reference(kid)?;
        did_doc_builder.add_assertion_method_reference(kid)?;
        Ok(did_doc_builder)
    }

    #[logfn_inputs(Trace)]
    pub fn add_context(&mut self, context: &str) {
        match self.context {
            StringOrVector::String(ref base_context) => {
                self.context =
                    StringOrVector::Vector(vec![base_context.to_string(), context.to_string()])
            }
            StringOrVector::Vector(ref mut contexts) => contexts.push(context.to_string()),
        };
    }

    #[logfn_inputs(Trace)]
    pub fn set_id(&mut self, id: &DID) {
        self.id = id.to_owned();
    }

    #[logfn_inputs(Trace)]
    pub fn set_controller(&mut self, controller: &str) {
        self.controller = Some(StringOrVector::String(controller.to_string()));
    }

    #[logfn_inputs(Trace)]
    #[allow(clippy::too_many_arguments)]
    pub fn add_verification_method(
        &mut self,
        key: &str,
        id: &str,
        type_: &VerificationKeyType,
        blockchain_account_id: Option<&str>,
        public_key_multibase: Option<&str>,
        public_key_hex: Option<&str>,
        public_key_base58: Option<&str>,
        public_key_base64: Option<&str>,
        public_key_jwk: Option<&str>,
    ) {
        let verification_method = VerificationMethod {
            id: id.to_string(),
            type_: type_.to_owned(),
            controller: self.id.to_string(),
            blockchain_account_id: blockchain_account_id.map(String::from),
            public_key_multibase: public_key_multibase.map(String::from),
            public_key_hex: public_key_hex.map(String::from),
            public_key_base58: public_key_base58.map(String::from),
            public_key_base64: public_key_base64.map(String::from),
            public_key_jwk: public_key_jwk.map(String::from),
        };
        self.verification_method
            .push((key.to_string(), verification_method));
    }

    #[logfn_inputs(Trace)]
    #[allow(clippy::too_many_arguments)]
    pub fn add_delegate_key(
        &mut self,
        key: &str,
        type_: &VerificationKeyType,
        blockchain_account_id: Option<&str>,
        public_key_multibase: Option<&str>,
        public_key_hex: Option<&str>,
        public_key_base58: Option<&str>,
        public_key_base64: Option<&str>,
        public_key_jwk: Option<&str>,
    ) {
        self.key_index += 1;
        let id = format!("{}#delegate-{}", self.id.as_ref(), self.key_index);
        let verification_method = VerificationMethod {
            id,
            type_: type_.to_owned(),
            controller: self.id.to_string(),
            blockchain_account_id: blockchain_account_id.map(String::from),
            public_key_multibase: public_key_multibase.map(String::from),
            public_key_hex: public_key_hex.map(String::from),
            public_key_base58: public_key_base58.map(String::from),
            public_key_base64: public_key_base64.map(String::from),
            public_key_jwk: public_key_jwk.map(String::from),
        };
        self.verification_method
            .push((key.to_string(), verification_method));
    }

    #[logfn_inputs(Trace)]
    pub fn remove_delegate_key(&mut self, key: &str) -> VdrResult<()> {
        // revocation is counted as an event, so we need to increment index
        self.key_index += 1;
        let index = self
            .verification_method
            .iter()
            .position(|(k, _)| k == key)
            .ok_or_else(|| {
                VdrError::CommonInvalidData(format!("Unable to find key for id: {}", key))
            })?;
        self.verification_method.remove(index);
        Ok(())
    }

    #[logfn_inputs(Trace)]
    pub fn add_authentication_reference(&mut self, key: &str) -> VdrResult<()> {
        let verification_method = Self::get_verification_method(self, key)?;
        let auth_reference =
            VerificationMethodOrReference::String(verification_method.id.to_string());
        self.authentication.push((key.to_string(), auth_reference));
        Ok(())
    }

    #[logfn_inputs(Trace)]
    pub fn remove_authentication_reference(&mut self, key: &str) -> VdrResult<()> {
        let index = self
            .authentication
            .iter()
            .position(|(k, _)| k == key)
            .ok_or_else(|| {
                VdrError::CommonInvalidData(format!("Unable to find key reference for id: {}", key))
            })?;
        self.authentication.remove(index);
        Ok(())
    }

    #[logfn_inputs(Trace)]
    pub fn add_assertion_method_reference(&mut self, key: &str) -> VdrResult<()> {
        let verification_method = Self::get_verification_method(self, key)?;
        let assertion_reference =
            VerificationMethodOrReference::String(verification_method.id.to_string());
        self.assertion_method
            .push((key.to_string(), assertion_reference));
        Ok(())
    }

    #[logfn_inputs(Trace)]
    pub fn remove_assertion_method_reference(&mut self, key: &str) -> VdrResult<()> {
        let index = self
            .assertion_method
            .iter()
            .position(|(k, _)| k == key)
            .ok_or_else(|| {
                VdrError::CommonInvalidData(format!("Unable to find key reference for id: {}", key))
            })?;
        self.assertion_method.remove(index);
        Ok(())
    }

    #[logfn_inputs(Trace)]
    pub fn add_capability_invocation_reference(&mut self, key: &str) -> VdrResult<()> {
        let verification_method = Self::get_verification_method(self, key)?;
        let capability_invocation_reference =
            VerificationMethodOrReference::String(verification_method.id.to_string());
        self.capability_invocation
            .push((key.to_string(), capability_invocation_reference));
        Ok(())
    }

    #[logfn_inputs(Trace)]
    pub fn remove_capability_invocation_reference(&mut self, key: &str) -> VdrResult<()> {
        let index = self
            .capability_invocation
            .iter()
            .position(|(k, _)| k == key)
            .ok_or_else(|| {
                VdrError::CommonInvalidData(format!("Unable to find key reference for id: {}", key))
            })?;
        self.capability_invocation.remove(index);
        Ok(())
    }

    #[logfn_inputs(Trace)]
    pub fn add_capability_delegation_reference(&mut self, key: &str) -> VdrResult<()> {
        let verification_method = Self::get_verification_method(self, key)?;
        let capability_delegation_reference =
            VerificationMethodOrReference::String(verification_method.id.to_string());
        self.capability_delegation
            .push((key.to_string(), capability_delegation_reference));
        Ok(())
    }

    #[logfn_inputs(Trace)]
    pub fn remove_capability_delegation_reference(&mut self, key: &str) -> VdrResult<()> {
        let index = self
            .capability_delegation
            .iter()
            .position(|(k, _)| k == key)
            .ok_or_else(|| {
                VdrError::CommonInvalidData(format!("Unable to find key reference for id: {}", key))
            })?;
        self.capability_delegation.remove(index);
        Ok(())
    }

    #[logfn_inputs(Trace)]
    pub fn add_key_agreement_reference(&mut self, key: &str) -> VdrResult<()> {
        let verification_method = Self::get_verification_method(self, key)?;
        let key_agreement_reference =
            VerificationMethodOrReference::String(verification_method.id.to_string());
        self.key_agreement
            .push((key.to_string(), key_agreement_reference));
        Ok(())
    }

    #[logfn_inputs(Trace)]
    pub fn remove_key_agreement_reference(&mut self, key: &str) -> VdrResult<()> {
        let index = self
            .key_agreement
            .iter()
            .position(|(k, _)| k == key)
            .ok_or_else(|| {
                VdrError::CommonInvalidData(format!("Unable to find key reference for id: {}", key))
            })?;
        self.key_agreement.remove(index);
        Ok(())
    }

    #[logfn_inputs(Trace)]
    pub fn add_service(
        &mut self,
        key: &str,
        id: Option<&str>,
        type_: &ServiceType,
        endpoint: &ServiceEndpoint,
    ) {
        self.service_index += 1;
        let id = id
            .map(String::from)
            .unwrap_or_else(|| format!("{}#service-{}", self.id.as_ref(), self.service_index));
        let service = Service {
            id,
            type_: type_.clone(),
            service_endpoint: endpoint.clone(),
        };
        self.service.push((key.to_string(), service));
    }

    #[logfn_inputs(Trace)]
    pub fn remove_service(&mut self, key: &str) -> VdrResult<()> {
        self.service_index += 1;
        let index = self
            .service
            .iter()
            .position(|(k, _)| k == key)
            .ok_or_else(|| {
                VdrError::CommonInvalidData(format!("Unable to find service for id: {}", key))
            })?;
        self.service.remove(index);
        Ok(())
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    fn get_verification_method(&self, key: &str) -> VdrResult<&VerificationMethod> {
        self
            .verification_method
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, key)| key)
            .ok_or_else(|| {
                let vdr_error =
                    VdrError::CommonInvalidData(
                        "Missing verification method".to_string()
                    );

                warn!(
                "Error: {} during getting verification method by key: {} from DidDocumentBuilder: {:?}",
                vdr_error, key, self
            );

                vdr_error
            })
    }

    pub fn deactivate(&mut self) {
        self.deactivated = true;
    }

    pub fn deactivated(&self) -> bool {
        self.deactivated
    }

    #[logfn(Trace)]
    pub fn build(self) -> DidDocument {
        if self.deactivated {
            return DidDocument {
                context: StringOrVector::String(BASE_CONTEXT.to_string()),
                id: self.id,
                ..DidDocument::default()
            };
        }

        DidDocument {
            context: self.context,
            id: self.id,
            controller: self.controller,
            verification_method: self
                .verification_method
                .into_iter()
                .map(|(_, key)| key)
                .collect(),
            authentication: self
                .authentication
                .into_iter()
                .map(|(_, key)| key)
                .collect(),
            assertion_method: self
                .assertion_method
                .into_iter()
                .map(|(_, key)| key)
                .collect(),
            capability_invocation: self
                .capability_invocation
                .into_iter()
                .map(|(_, key)| key)
                .collect(),
            capability_delegation: self
                .capability_delegation
                .into_iter()
                .map(|(_, key)| key)
                .collect(),
            key_agreement: self.key_agreement.into_iter().map(|(_, key)| key).collect(),
            service: self.service.into_iter().map(|(_, key)| key).collect(),
            also_known_as: self.also_known_as,
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    use crate::{
        client::client::test::TEST_ACCOUNT,
        contracts::{
            types::did_doc::test::{
                default_ethr_did_document, SERVICE_ENDPOINT, TEST_ETHR_DID,
                TEST_ETHR_DID_WITHOUT_NETWORK,
            },
            ServiceType,
        },
    };

    const KEY_1_INDEX: &str = "KEY_1";
    const KEY_2_INDEX: &str = "KEY_2";
    const KEY_3_INDEX: &str = "KEY_3";
    const SERVICE_1_INDEX: &str = "SERVICE_1";

    #[test]
    fn build_base_ethr_did_document_test() {
        let did_document = DidDocumentBuilder::base_for_did(&DID::from(TEST_ETHR_DID), 1)
            .unwrap()
            .build();
        assert_eq!(
            default_ethr_did_document(TEST_ACCOUNT.as_ref(), None),
            did_document
        );
    }

    #[test]
    fn build_did_document_test() {
        let mut builder = DidDocumentBuilder::base_for_did(&DID::from(TEST_ETHR_DID), 1).unwrap();
        builder.add_delegate_key(
            KEY_1_INDEX,
            &VerificationKeyType::EcdsaSecp256k1RecoveryMethod2020,
            Some("eip155:1:0x12345678c498d9e26865f34fcaa57dbb935b0d74"),
            None,
            None,
            None,
            None,
            None,
        );
        builder.add_delegate_key(
            KEY_2_INDEX,
            &VerificationKeyType::X25519KeyAgreementKey2020,
            None,
            None,
            None,
            Some("FbQWLPRhTH95MCkQUeFYdiSoQt8zMwetqfWoxqPgaq7x"),
            None,
            None,
        );
        builder.add_delegate_key(
            KEY_3_INDEX,
            &VerificationKeyType::EcdsaSecp256k1VerificationKey2020,
            None,
            None,
            Some("02b97c30de767f084ce3080168ee293053ba33b235d7116a3263d29f1450936b71"),
            None,
            None,
            None,
        );
        builder.add_assertion_method_reference(KEY_1_INDEX).unwrap();
        builder.add_assertion_method_reference(KEY_2_INDEX).unwrap();
        builder.add_authentication_reference(KEY_1_INDEX).unwrap();
        builder.add_key_agreement_reference(KEY_3_INDEX).unwrap();

        builder.add_service(
            SERVICE_1_INDEX,
            None,
            &ServiceType::LinkedDomains,
            &ServiceEndpoint::String(SERVICE_ENDPOINT.to_string()),
        );
        let did_document = builder.build();

        assert_eq!(4, did_document.verification_method.len());
        assert_eq!(3, did_document.assertion_method.len());
        assert_eq!(2, did_document.authentication.len());
        assert_eq!(1, did_document.key_agreement.len());
        assert_eq!(1, did_document.service.len());

        assert_eq!(
            format!("{}#controller", TEST_ETHR_DID_WITHOUT_NETWORK),
            did_document.verification_method[0].id
        );
        assert_eq!(
            format!("{}#delegate-1", TEST_ETHR_DID_WITHOUT_NETWORK),
            did_document.verification_method[1].id
        );
        assert_eq!(
            format!("{}#delegate-2", TEST_ETHR_DID_WITHOUT_NETWORK),
            did_document.verification_method[2].id
        );
        assert_eq!(
            format!("{}#delegate-3", TEST_ETHR_DID_WITHOUT_NETWORK),
            did_document.verification_method[3].id
        );
        assert_eq!(
            format!("{}#service-1", TEST_ETHR_DID_WITHOUT_NETWORK),
            did_document.service[0].id
        );
    }
}
