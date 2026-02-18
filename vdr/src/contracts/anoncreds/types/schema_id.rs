// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{contracts::did::types::did::DID, types::ContractParam, VdrError, VdrResult};

use crate::contracts::types::did::ParsedDid;
use serde_derive::{Deserialize, Serialize};
use sha3::Digest;

/// Wrapper structure for AnonCreds Schema ID
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SchemaId(String);

impl SchemaId {
    const ID_PATH: &'static str = "/anoncreds/v0/SCHEMA";

    pub fn build(issuer_id: &DID, name: &str, version: &str) -> SchemaId {
        SchemaId::from(
            format!(
                "{}{}/{}/{}",
                issuer_id.as_ref(),
                Self::ID_PATH,
                name,
                version
            )
            .as_str(),
        )
    }

    // A unique identifier for the schema needed for cred def
    // referencing to https://hyperledger.github.io/indy-did-method/#cred-def
    pub fn unique_id(&self) -> String {
        self.0.replace(Self::ID_PATH, "").replace('/', ":")
    }

    pub(crate) fn hash(&self) -> Vec<u8> {
        sha3::Keccak256::digest(self.0.as_bytes()).to_vec()
    }

    pub fn without_network(&self) -> VdrResult<SchemaId> {
        ParsedSchemaId::try_from(self)?.as_short_id()
    }
}

impl TryFrom<&SchemaId> for ContractParam {
    type Error = VdrError;

    fn try_from(value: &SchemaId) -> Result<Self, Self::Error> {
        Ok(ContractParam::FixedBytes(value.hash()))
    }
}

impl From<&str> for SchemaId {
    fn from(id: &str) -> Self {
        SchemaId(id.to_string())
    }
}

impl AsRef<str> for SchemaId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl ToString for SchemaId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct ParsedSchemaId {
    pub(crate) issuer_id: DID,
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) network: Option<String>,
}

impl ParsedSchemaId {
    pub(crate) fn as_short_id(&self) -> VdrResult<SchemaId> {
        Ok(SchemaId::build(
            &self.issuer_id.without_network()?,
            &self.name,
            &self.version,
        ))
    }
}

impl TryFrom<&SchemaId> for ParsedSchemaId {
    type Error = VdrError;

    fn try_from(schema_id: &SchemaId) -> Result<Self, Self::Error> {
        let parts = schema_id.as_ref().split('/').collect::<Vec<&str>>();
        if parts.len() != 6 {
            return Err(VdrError::CommonInvalidData(
                "Invalid schema id provided".to_string(),
            ));
        }
        let issuer_id = DID::from(parts[0]);
        let parsed_issuer_id = ParsedDid::try_from(&issuer_id)?;
        Ok(ParsedSchemaId {
            issuer_id: DID::from(parts[0]),
            name: parts[4].to_string(),
            version: parts[5].to_string(),
            network: parsed_issuer_id.network,
        })
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::contracts::anoncreds::types::schema::test::{SCHEMA_ID, SCHEMA_ID_WITHOUT_NETWORK};

    #[test]
    fn schema_id_without_network() {
        assert_eq!(
            SchemaId::from(SCHEMA_ID_WITHOUT_NETWORK),
            SchemaId::from(SCHEMA_ID).without_network().unwrap()
        )
    }
}
