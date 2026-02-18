// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{
    contracts::did::types::did::DID, types::ContractParam, CredentialDefinitionId, VdrError,
    VdrResult,
};

use crate::contracts::types::did::ParsedDid;
use serde_derive::{Deserialize, Serialize};
use sha3::Digest;

/// Wrapper structure for AnonCreds Revocation Registry Definition ID
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct RevocationRegistryDefinitionId(String);

impl RevocationRegistryDefinitionId {
    const ID_PATH: &'static str = "anoncreds/v0/REV_REG_DEF";

    pub fn build(issuer_id: &DID, cred_def_id: &str, tag: &str) -> RevocationRegistryDefinitionId {
        let rev_reg_def_id_without_issuer = cred_def_id
            .replace(CredentialDefinitionId::ID_PATH, Self::ID_PATH)
            .split('/')
            .skip(1)
            .collect::<Vec<&str>>()
            .join("/");

        RevocationRegistryDefinitionId::from(
            format!(
                "{}/{}/{}",
                issuer_id.as_ref(),
                rev_reg_def_id_without_issuer,
                tag
            )
            .as_str(),
        )
    }

    pub(crate) fn hash(&self) -> Vec<u8> {
        sha3::Keccak256::digest(self.0.as_bytes()).to_vec()
    }

    pub(crate) fn to_filter(&self) -> String {
        self.hash()
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect()
    }

    pub fn without_network(&self) -> VdrResult<RevocationRegistryDefinitionId> {
        ParsedRevocationRegistryDefinitionId::try_from(self)?.as_short_id()
    }
}

impl TryFrom<&RevocationRegistryDefinitionId> for ContractParam {
    type Error = VdrError;

    fn try_from(value: &RevocationRegistryDefinitionId) -> Result<Self, Self::Error> {
        Ok(ContractParam::FixedBytes(value.hash()))
    }
}

impl From<&str> for RevocationRegistryDefinitionId {
    fn from(id: &str) -> Self {
        RevocationRegistryDefinitionId(id.to_string())
    }
}

impl AsRef<str> for RevocationRegistryDefinitionId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl ToString for RevocationRegistryDefinitionId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct ParsedRevocationRegistryDefinitionId {
    pub(crate) issuer_id: DID,
    pub(crate) cred_def_id: String,
    pub(crate) tag: String,
    pub(crate) network: Option<String>,
}

impl ParsedRevocationRegistryDefinitionId {
    pub(crate) fn as_short_id(&self) -> VdrResult<RevocationRegistryDefinitionId> {
        Ok(RevocationRegistryDefinitionId::build(
            &self.issuer_id.without_network()?,
            &self.cred_def_id,
            &self.tag,
        ))
    }
}

impl TryFrom<&RevocationRegistryDefinitionId> for ParsedRevocationRegistryDefinitionId {
    type Error = VdrError;

    //TODO: 6? 7?
    fn try_from(rev_reg_def_id: &RevocationRegistryDefinitionId) -> Result<Self, Self::Error> {
        let parts = rev_reg_def_id.as_ref().split('/').collect::<Vec<&str>>();
        if parts.len() != 7 {
            return Err(VdrError::CommonInvalidData(
                "Invalid revocation registry definition id provided".to_string(),
            ));
        }
        let issuer_id = DID::from(parts[0]);
        let tag = parts[6];
        let cred_def_id = parts
            .iter()
            .take(parts.len() - 1)
            .cloned()
            .collect::<Vec<&str>>()
            .join("/");

        let parsed_issuer_id = ParsedDid::try_from(&issuer_id)?;
        let cred_def_id = CredentialDefinitionId::from(cred_def_id.as_str());
        Ok(ParsedRevocationRegistryDefinitionId {
            issuer_id,
            cred_def_id: cred_def_id.to_string(),
            tag: tag.to_string(),
            network: parsed_issuer_id.network,
        })
    }
}

//TODO:
// #[cfg(test)]
// pub mod test {
//     use super::*;
//     use crate::contracts::anoncreds::types::revocation_registry_definition::test::{
//         REVOCATION_REGISTRY_DEFINITION_ID, REVOCATION_REGISTRY_DEFINITION_ID_WITHOUT_NETWORK,
//     };
// }
