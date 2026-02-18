// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::VdrError,
    types::{ContractOutput, ContractParam},
    CredentialDefinitionId, VdrResult,
};

use crate::contracts::{anoncreds::types::schema_id::SchemaId, did::types::did::DID};
use serde_derive::{Deserialize, Serialize};

/// Credential Definition Record stored in the Registry
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialDefinitionRecord {
    pub credential_definition: CredentialDefinition,
    pub metadata: CredentialDefinitionMetadata,
}

pub use indy_data_types::anoncreds::cred_def::SignatureType;

/// Definition of AnonCreds Credential Definition object matching to the specification - `<https://hyperledger.github.io/anoncreds-spec/#term:credential-definition>`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CredentialDefinition {
    #[serde(rename = "issuerId")]
    pub issuer_id: DID,
    #[serde(rename = "schemaId")]
    pub schema_id: SchemaId,
    #[serde(rename = "credDefType")]
    pub cred_def_type: SignatureType,
    pub tag: String,
    pub value: serde_json::Value,
}

impl CredentialDefinition {
    pub fn new(
        issuer_id: DID,
        schema_id: SchemaId,
        cred_def_type: SignatureType,
        tag: String,
        value: serde_json::Value,
    ) -> CredentialDefinition {
        CredentialDefinition {
            issuer_id,
            schema_id,
            cred_def_type,
            tag,
            value,
        }
    }

    pub fn id(&self) -> CredentialDefinitionId {
        CredentialDefinitionId::build(&self.issuer_id, &self.schema_id.unique_id(), &self.tag)
    }

    pub(crate) fn validate(&self) -> VdrResult<()> {
        if self.tag.is_empty() {
            return Err(VdrError::InvalidCredentialDefinition(
                "Tag is not provided".to_string(),
            ));
        }

        if self.value.is_null() {
            return Err(VdrError::InvalidCredentialDefinition(
                "Value is not provided".to_string(),
            ));
        }

        Ok(())
    }

    pub fn to_string(&self) -> VdrResult<String> {
        serde_json::to_string(self).map_err(|err| {
            VdrError::InvalidCredentialDefinition(format!(
                "Unable to serialize Credential Definition as JSON. Err: {:?}",
                err
            ))
        })
    }

    pub fn from_string(value: &str) -> VdrResult<CredentialDefinition> {
        serde_json::from_str(value).map_err(|err| {
            VdrError::InvalidCredentialDefinition(format!(
                "Unable to parse Credential Definition from JSON. Err: {:?}",
                err.to_string()
            ))
        })
    }
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct CredentialDefinitionMetadata {
    pub created: u64,
}

impl TryFrom<&CredentialDefinition> for ContractParam {
    type Error = VdrError;

    fn try_from(value: &CredentialDefinition) -> Result<Self, Self::Error> {
        serde_json::to_vec(value)
            .map(ContractParam::Bytes)
            .map_err(|_| VdrError::ContractInvalidInputData)
    }
}

impl TryFrom<&ContractOutput> for CredentialDefinition {
    type Error = VdrError;

    fn try_from(value: &ContractOutput) -> Result<Self, Self::Error> {
        serde_json::from_slice(&value.get_bytes(0)?).map_err(|err| {
            VdrError::ContractInvalidResponseData(format!(
                "Unable to parse CredentialDefinition from the response. Err: {:?}",
                err
            ))
        })
    }
}

impl TryFrom<ContractOutput> for CredentialDefinitionMetadata {
    type Error = VdrError;

    fn try_from(value: ContractOutput) -> Result<Self, Self::Error> {
        let created = value.get_u128(0)?;
        let cred_def_metadata = CredentialDefinitionMetadata {
            created: created as u64,
        };
        Ok(cred_def_metadata)
    }
}

impl TryFrom<ContractOutput> for CredentialDefinitionRecord {
    type Error = VdrError;

    fn try_from(value: ContractOutput) -> Result<Self, Self::Error> {
        let output_tuple = value.get_tuple(0)?;
        let credential_definition = CredentialDefinition::try_from(&output_tuple)?;
        let metadata = output_tuple.get_tuple(1)?;

        let cred_def_with_metadata = CredentialDefinitionRecord {
            credential_definition,
            metadata: CredentialDefinitionMetadata::try_from(metadata)?,
        };
        Ok(cred_def_with_metadata)
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::{
        contracts::{
            anoncreds::types::schema::test::SCHEMA_ID, did::types::did_doc::test::TEST_ETHR_DID,
        },
        utils::rand_string,
    };
    use serde_json::json;

    pub const CREDENTIAL_DEFINITION_ID: &str = "did:ethr:testnet:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5/anoncreds/v0/CLAIM_DEF/did:ethr:testnet:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5:F1DClaFEzi3t:1.0.0/default";
    pub const CREDENTIAL_DEFINITION_ID_WITHOUT_NETWORK: &str = "did:ethr:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5/anoncreds/v0/CLAIM_DEF/did:ethr:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5:F1DClaFEzi3t:1.0.0/default";
    pub const CREDENTIAL_DEFINITION_TAG: &str = "default";

    pub fn credential_definition_value() -> serde_json::Value {
        json!({
            "n": "779...397",
            "rctxt": "774...977",
            "s": "750..893",
            "z":
            "632...005"
        })
    }

    pub fn credential_definition(
        issuer_id: &DID,
        schema_id: &SchemaId,
        tag: Option<&str>,
    ) -> CredentialDefinition {
        let tag = tag.map(String::from).unwrap_or_else(rand_string);
        CredentialDefinition {
            issuer_id: issuer_id.clone(),
            schema_id: SchemaId::from(schema_id.as_ref()),
            cred_def_type: SignatureType::CL,
            tag: tag.to_string(),
            value: credential_definition_value(),
        }
    }

    fn cred_def_param() -> ContractParam {
        let cred_def = credential_definition(
            &DID::from(TEST_ETHR_DID),
            &SchemaId::from(SCHEMA_ID),
            Some(CREDENTIAL_DEFINITION_TAG),
        );
        ContractParam::Bytes(serde_json::to_vec(&cred_def).unwrap())
    }

    mod convert_into_contract_param {
        use super::*;

        #[test]
        fn convert_cred_def_into_contract_param_test() {
            let credential_definition = credential_definition(
                &DID::from(TEST_ETHR_DID),
                &SchemaId::from(SCHEMA_ID),
                Some(CREDENTIAL_DEFINITION_TAG),
            );
            let param: ContractParam = (&credential_definition).try_into().unwrap();
            assert_eq!(cred_def_param(), param);
        }
    }

    mod convert_into_object {
        use super::*;

        #[test]
        fn convert_contract_output_into_cred_def() {
            let data = ContractOutput::new(vec![cred_def_param()]);
            let converted = CredentialDefinition::try_from(&data).unwrap();
            let credential_definition = credential_definition(
                &DID::from(TEST_ETHR_DID),
                &SchemaId::from(SCHEMA_ID),
                Some(CREDENTIAL_DEFINITION_TAG),
            );
            assert_eq!(credential_definition, converted);
        }
    }
}
