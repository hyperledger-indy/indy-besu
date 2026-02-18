// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::VdrError,
    types::{ContractOutput, ContractParam},
    CredentialDefinitionId, VdrResult,
};

use crate::contracts::did::types::did::DID;

use serde_derive::{Deserialize, Serialize};

/// Credential Definition Record stored in the Registry
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevocationRegistryDefinitionRecord {
    pub revocation_registry_definition: RevocationRegistryDefinition,
    pub metadata: RevocationRegistryDefinitionMetadata,
}

pub use indy_data_types::anoncreds::rev_reg_def::RegistryType;

use super::{
    credential_definition_id::ParsedCredentialDefinitionId,
    revocation_registry_definition_id::RevocationRegistryDefinitionId,
};

/// Definition of AnonCreds Revocation Registry Definition object matching to the specification - `<https://hyperledger.github.io/anoncreds-spec/#term:revocation-registry-definition>`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RevocationRegistryDefinition {
    #[serde(rename = "issuerId")]
    pub issuer_id: DID,
    #[serde(rename = "revocDefType")]
    pub revoc_def_type: RegistryType,
    #[serde(rename = "credDefId")]
    pub cred_def_id: CredentialDefinitionId,
    pub tag: String,
    pub value: RevocationRegistryDefinitionValue,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RevocationRegistryDefinitionValue {
    #[serde(rename = "maxCredNum")]
    pub max_cred_num: u32,
    #[serde(rename = "publicKeys")]
    pub public_keys: PublicKeys,
    #[serde(rename = "tailsHash")]
    pub tails_hash: String,
    #[serde(rename = "tailsLocation")]
    pub tails_location: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccumKey {
    pub z: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PublicKeys {
    #[serde(rename = "accumKey")]
    pub accum_key: AccumKey,
}

impl RevocationRegistryDefinition {
    pub fn new(
        issuer_id: DID,
        revoc_def_type: RegistryType,
        cred_def_id: CredentialDefinitionId,
        tag: String,
        value: RevocationRegistryDefinitionValue,
    ) -> RevocationRegistryDefinition {
        RevocationRegistryDefinition {
            issuer_id,
            revoc_def_type,
            cred_def_id,
            tag,
            value,
        }
    }

    pub fn id(&self) -> RevocationRegistryDefinitionId {
        RevocationRegistryDefinitionId::build(
            &self.issuer_id,
            &self.cred_def_id.to_string(),
            &self.tag,
        )
    }

    pub(crate) fn validate(&self) -> VdrResult<()> {
        if self.tag.is_empty() {
            return Err(VdrError::InvalidRevocationRegistryDefinition(
                "Tag is not provided".to_string(),
            ));
        }

        if self.value.max_cred_num == 0 {
            return Err(VdrError::InvalidRevocationRegistryDefinition(
                "Max Cred Num cannot be zero".to_string(),
            ));
        }

        if self.value.tails_location.is_empty() {
            return Err(VdrError::InvalidRevocationRegistryDefinition(
                "Tails Location is not provided".to_string(),
            ));
        }

        if self.value.tails_hash.is_empty() {
            return Err(VdrError::InvalidRevocationRegistryDefinition(
                "Tails Location is not provided".to_string(),
            ));
        }

        if self.value.public_keys.accum_key.z.is_empty() {
            return Err(VdrError::InvalidRevocationRegistryDefinition(
                "Public Key is not provided".to_string(),
            ));
        }

        if self
            .issuer_id
            .as_ref()
            .ne(ParsedCredentialDefinitionId::try_from(&self.cred_def_id)?
                .issuer_id
                .as_ref())
        {
            return Err(VdrError::InvalidRevocationRegistryDefinition(
                "Revocation Registry Definition Issuer differs from Credential Definition Issuer"
                    .to_string(),
            ));
        }

        Ok(())
    }

    pub fn to_string(&self) -> VdrResult<String> {
        serde_json::to_string(self).map_err(|err| {
            VdrError::InvalidRevocationRegistryDefinition(format!(
                "Unable to serialize Revocation Registry Definition as JSON. Err: {:?}",
                err
            ))
        })
    }

    pub fn from_string(value: &str) -> VdrResult<RevocationRegistryDefinition> {
        serde_json::from_str(value).map_err(|err| {
            VdrError::InvalidRevocationRegistryDefinition(format!(
                "Unable to parse Revocation Registry Definition from JSON. Err: {:?}",
                err.to_string()
            ))
        })
    }
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct RevocationRegistryDefinitionMetadata {
    pub created: u64,
}

impl TryFrom<&RevocationRegistryDefinition> for ContractParam {
    type Error = VdrError;

    fn try_from(value: &RevocationRegistryDefinition) -> Result<Self, Self::Error> {
        serde_json::to_vec(value)
            .map(ContractParam::Bytes)
            .map_err(|_| VdrError::ContractInvalidInputData)
    }
}

impl TryFrom<&ContractOutput> for RevocationRegistryDefinition {
    type Error = VdrError;

    fn try_from(value: &ContractOutput) -> Result<Self, Self::Error> {
        serde_json::from_slice(&value.get_bytes(0)?).map_err(|err| {
            VdrError::ContractInvalidResponseData(format!(
                "Unable to parse RevocationRegistryDefinition from the response. Err: {:?}",
                err
            ))
        })
    }
}

impl TryFrom<ContractOutput> for RevocationRegistryDefinitionMetadata {
    type Error = VdrError;

    fn try_from(value: ContractOutput) -> Result<Self, Self::Error> {
        let created = value.get_u128(0)?;
        let cred_def_metadata = RevocationRegistryDefinitionMetadata {
            created: created as u64,
        };
        Ok(cred_def_metadata)
    }
}

impl TryFrom<ContractOutput> for RevocationRegistryDefinitionRecord {
    type Error = VdrError;

    fn try_from(value: ContractOutput) -> Result<Self, Self::Error> {
        let output_tuple = value.get_tuple(0)?;
        let revocation_registry_definition = RevocationRegistryDefinition::try_from(&output_tuple)?;
        let metadata = output_tuple.get_tuple(1)?;

        let rev_reg_def_with_metadata = RevocationRegistryDefinitionRecord {
            revocation_registry_definition,
            metadata: RevocationRegistryDefinitionMetadata::try_from(metadata)?,
        };
        Ok(rev_reg_def_with_metadata)
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::{
        contracts::{
            anoncreds::types::credential_definition::test::CREDENTIAL_DEFINITION_ID,
            did::types::did_doc::test::TEST_ETHR_DID,
        },
        utils::rand_string,
        CredentialDefinitionId,
    };
    use serde_json::json;

    pub const REVOCATION_REGISTRY_DEFINITION_ID: &str = "did:ethr:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5/anoncreds/v0/REV_REG_DEF/did:ethr:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5:F1DClaFEzi3t:1.0.0/default/tag";
    pub const REVOCATION_REGISTRY_DEFINITION_TAG: &str = "tag";

    pub fn revocation_registry_definition(
        issuer_id: &DID,
        cred_def_id: &CredentialDefinitionId,
        tag: Option<&str>,
    ) -> RevocationRegistryDefinition {
        let tag = tag.map(String::from).unwrap_or_else(rand_string);
        let accum_key = AccumKey {
            z: String::from("1 0BB...386"),
        };
        let public_keys = PublicKeys { accum_key };
        let value = RevocationRegistryDefinitionValue {
            max_cred_num: 20,
            public_keys,
            tails_location: String::from("https://my.revocations.tails/tailsfile.txt"),
            tails_hash: String::from("91zvq2cFmBZmHCcLqFyzv7bfehHH5rMhdAG5wTjqy2PE"),
        };
        RevocationRegistryDefinition {
            issuer_id: issuer_id.clone(),
            cred_def_id: CredentialDefinitionId::from(cred_def_id.as_ref()),
            revoc_def_type: RegistryType::CL_ACCUM,
            tag: tag.to_string(),
            value,
        }
    }

    fn rev_reg_def_param() -> ContractParam {
        let rev_reg_def = revocation_registry_definition(
            &DID::from(TEST_ETHR_DID),
            &CredentialDefinitionId::from(CREDENTIAL_DEFINITION_ID),
            Some(REVOCATION_REGISTRY_DEFINITION_TAG),
        );
        ContractParam::Bytes(serde_json::to_vec(&rev_reg_def).unwrap())
    }

    mod convert_into_contract_param {
        use super::*;

        #[test]
        fn convert_rev_reg_def_into_contract_param_test() {
            let rev_reg_def = revocation_registry_definition(
                &DID::from(TEST_ETHR_DID),
                &CredentialDefinitionId::from(CREDENTIAL_DEFINITION_ID),
                Some(REVOCATION_REGISTRY_DEFINITION_TAG),
            );
            let param: ContractParam = (&rev_reg_def).try_into().unwrap();
            assert_eq!(rev_reg_def_param(), param);
        }
    }

    mod convert_into_object {
        use super::*;

        #[test]
        fn convert_contract_output_into_rev_reg_def() {
            let data = ContractOutput::new(vec![rev_reg_def_param()]);
            let converted = RevocationRegistryDefinition::try_from(&data).unwrap();
            let rev_reg_def = revocation_registry_definition(
                &DID::from(TEST_ETHR_DID),
                &&CredentialDefinitionId::from(CREDENTIAL_DEFINITION_ID),
                Some(REVOCATION_REGISTRY_DEFINITION_TAG),
            );
            assert_eq!(rev_reg_def, converted);
        }
    }
}
