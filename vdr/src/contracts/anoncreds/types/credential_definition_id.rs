// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{contracts::did::types::did::DID, types::ContractParam, VdrError, VdrResult};

use crate::contracts::types::did::ParsedDid;
use serde_derive::{Deserialize, Serialize};
use sha3::Digest;

/// Wrapper structure for AnonCreds Credential Definition ID
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CredentialDefinitionId(String);

impl CredentialDefinitionId {
    pub const ID_PATH: &'static str = "anoncreds/v0/CLAIM_DEF";

    pub fn build(issuer_id: &DID, schema_id: &str, tag: &str) -> CredentialDefinitionId {
        CredentialDefinitionId::from(
            format!(
                "{}/{}/{}/{}",
                issuer_id.as_ref(),
                Self::ID_PATH,
                schema_id,
                tag
            )
            .as_str(),
        )
    }

    pub(crate) fn hash(&self) -> Vec<u8> {
        sha3::Keccak256::digest(self.0.as_bytes()).to_vec()
    }

    pub fn without_network(&self) -> VdrResult<CredentialDefinitionId> {
        ParsedCredentialDefinitionId::try_from(self)?.as_short_id()
    }
}

impl TryFrom<&CredentialDefinitionId> for ContractParam {
    type Error = VdrError;

    fn try_from(value: &CredentialDefinitionId) -> Result<Self, Self::Error> {
        Ok(ContractParam::FixedBytes(value.hash()))
    }
}

impl From<&str> for CredentialDefinitionId {
    fn from(id: &str) -> Self {
        CredentialDefinitionId(id.to_string())
    }
}

impl AsRef<str> for CredentialDefinitionId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl ToString for CredentialDefinitionId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct ParsedCredentialDefinitionId {
    pub(crate) issuer_id: DID,
    pub(crate) schema_id: String,
    pub(crate) tag: String,
    pub(crate) network: Option<String>,
}

impl ParsedCredentialDefinitionId {
    pub(crate) fn as_short_id(&self) -> VdrResult<CredentialDefinitionId> {
        Ok(CredentialDefinitionId::build(
            &self.issuer_id.without_network()?,
            &self.schema_id,
            &self.tag,
        ))
    }
}

impl TryFrom<&CredentialDefinitionId> for ParsedCredentialDefinitionId {
    type Error = VdrError;

    fn try_from(cred_def_id: &CredentialDefinitionId) -> Result<Self, Self::Error> {
        let parts = cred_def_id.as_ref().split('/').collect::<Vec<&str>>();
        if parts.len() != 6 {
            return Err(VdrError::CommonInvalidData(
                "Invalid credential definition id provided".to_string(),
            ));
        }
        let issuer_id = DID::from(parts[0]);
        let parsed_issuer_id = ParsedDid::try_from(&issuer_id)?;
        let parsed_schema_id = ParsedCredentialDefinitionSchemaId::try_from(parts[4])?;
        Ok(ParsedCredentialDefinitionId {
            issuer_id: DID::from(parts[0]),
            schema_id: parsed_schema_id.as_short_id()?,
            tag: parts[5].to_string(),
            network: parsed_issuer_id.network,
        })
    }
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct ParsedCredentialDefinitionSchemaId {
    pub(crate) issuer_id: DID,
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) network: Option<String>,
}

impl ParsedCredentialDefinitionSchemaId {
    pub(crate) fn as_short_id(&self) -> VdrResult<String> {
        Ok(format!(
            "{}:{}:{}",
            self.issuer_id.without_network()?.as_ref(),
            self.name,
            self.version
        ))
    }
}

impl TryFrom<&str> for ParsedCredentialDefinitionSchemaId {
    type Error = VdrError;

    fn try_from(schema_id: &str) -> Result<Self, Self::Error> {
        let parts = schema_id.split(':').collect::<Vec<&str>>();
        if parts.len() == 6 {
            return Ok(ParsedCredentialDefinitionSchemaId {
                issuer_id: DID::build(parts[1], None, parts[3]),
                name: parts[4].to_string(),
                version: parts[5].to_string(),
                network: Some(parts[2].to_string()),
            });
        }
        if parts.len() == 5 {
            return Ok(ParsedCredentialDefinitionSchemaId {
                issuer_id: DID::build(parts[1], None, parts[2]),
                name: parts[3].to_string(),
                version: parts[4].to_string(),
                network: None,
            });
        }
        return Err(VdrError::CommonInvalidData(
            "Invalid credential definition id provided".to_string(),
        ));
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::contracts::anoncreds::types::credential_definition::test::{
        CREDENTIAL_DEFINITION_ID, CREDENTIAL_DEFINITION_ID_WITHOUT_NETWORK,
    };

    #[test]
    fn cred_def_id_id_without_network() {
        assert_eq!(
            CredentialDefinitionId::from(CREDENTIAL_DEFINITION_ID_WITHOUT_NETWORK),
            CredentialDefinitionId::from(CREDENTIAL_DEFINITION_ID)
                .without_network()
                .unwrap()
        )
    }
}
