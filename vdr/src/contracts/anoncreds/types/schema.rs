// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::VdrError,
    types::{ContractOutput, ContractParam},
    SchemaId, VdrResult,
};
use std::collections::HashSet;

use crate::contracts::did::types::did::DID;
use serde_derive::{Deserialize, Serialize};

/// Schema Record stored in the Registry
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaRecord {
    pub schema: Schema,
    pub metadata: SchemaMetadata,
}

/// Definition of AnonCreds Schema object matching to the specification - `<https://hyperledger.github.io/anoncreds-spec/#term:schema>`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Schema {
    #[serde(rename = "issuerId")]
    pub issuer_id: DID,
    pub name: String,
    pub version: String,
    #[serde(rename = "attrNames")]
    pub attr_names: HashSet<String>,
}

impl Schema {
    pub fn new(
        issuer_id: DID,
        name: String,
        version: String,
        attr_names: HashSet<String>,
    ) -> Schema {
        Schema {
            issuer_id,
            name,
            version,
            attr_names,
        }
    }

    pub fn id(&self) -> SchemaId {
        SchemaId::build(&self.issuer_id, &self.name, &self.version)
    }

    pub(crate) fn validate(&self) -> VdrResult<()> {
        if self.name.is_empty() {
            return Err(VdrError::InvalidSchema("Name is not provided".to_string()));
        }

        if self.version.is_empty() {
            return Err(VdrError::InvalidSchema(
                "Version is not provided".to_string(),
            ));
        }

        if self.attr_names.is_empty() {
            return Err(VdrError::InvalidSchema(
                "Attributes are not provided".to_string(),
            ));
        }

        Ok(())
    }

    pub fn to_string(&self) -> VdrResult<String> {
        serde_json::to_string(self).map_err(|err| {
            VdrError::InvalidSchema(format!(
                "Unable to serialize Schema as JSON. Err: {:?}",
                err
            ))
        })
    }

    pub fn from_string(value: &str) -> VdrResult<Schema> {
        serde_json::from_str(value).map_err(|err| {
            VdrError::InvalidSchema(format!(
                "Unable to parse Schema from JSON. Err: {:?}",
                err.to_string()
            ))
        })
    }
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct SchemaMetadata {
    pub created: u64,
}

impl TryFrom<&Schema> for ContractParam {
    type Error = VdrError;

    fn try_from(value: &Schema) -> Result<Self, Self::Error> {
        serde_json::to_vec(value)
            .map(ContractParam::Bytes)
            .map_err(|_| VdrError::ContractInvalidInputData)
    }
}

impl TryFrom<&ContractOutput> for Schema {
    type Error = VdrError;

    fn try_from(value: &ContractOutput) -> Result<Self, Self::Error> {
        serde_json::from_slice(&value.get_bytes(0)?).map_err(|err| {
            VdrError::ContractInvalidResponseData(format!(
                "Unable to parse Schema from the response. Err: {:?}",
                err
            ))
        })
    }
}

impl TryFrom<ContractOutput> for SchemaMetadata {
    type Error = VdrError;

    fn try_from(value: ContractOutput) -> Result<Self, Self::Error> {
        let created = value.get_u128(0)?;
        let schema_metadata = SchemaMetadata {
            created: created as u64,
        };
        Ok(schema_metadata)
    }
}

impl TryFrom<ContractOutput> for SchemaRecord {
    type Error = VdrError;

    fn try_from(value: ContractOutput) -> Result<Self, Self::Error> {
        let output_tuple = value.get_tuple(0)?;
        let schema = Schema::try_from(&output_tuple)?;
        let metadata = output_tuple.get_tuple(1)?;

        let schema_with_meta = SchemaRecord {
            schema,
            metadata: SchemaMetadata::try_from(metadata)?,
        };
        Ok(schema_with_meta)
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::{contracts::did::types::did_doc::test::TEST_ETHR_DID, utils::rand_string};
    use once_cell::sync::Lazy;

    pub const SCHEMA_ID: &str =
        "did:ethr:testnet:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5/anoncreds/v0/SCHEMA/F1DClaFEzi3t/1.0.0";
    pub const SCHEMA_ID_WITHOUT_NETWORK: &str =
        "did:ethr:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5/anoncreds/v0/SCHEMA/F1DClaFEzi3t/1.0.0";
    pub const SCHEMA_NAME: &str = "F1DClaFEzi3t";
    pub const SCHEMA_VERSION: &str = "1.0.0";
    pub const SCHEMA_ATTRIBUTE_FIRST_NAME: &str = "First Name";
    pub static SCHEMA_ATTRIBUTES: Lazy<HashSet<String>> = Lazy::new(|| {
        let mut attr_names: HashSet<String> = HashSet::new();
        attr_names.insert(SCHEMA_ATTRIBUTE_FIRST_NAME.to_string());
        attr_names
    });

    pub fn schema(issuer_id: &DID, name: Option<&str>) -> Schema {
        let name = name.map(String::from).unwrap_or_else(rand_string);
        let mut attr_names: HashSet<String> = HashSet::new();
        attr_names.insert(SCHEMA_ATTRIBUTE_FIRST_NAME.to_string());

        Schema {
            issuer_id: issuer_id.clone(),
            name,
            version: SCHEMA_VERSION.to_string(),
            attr_names: SCHEMA_ATTRIBUTES.clone(),
        }
    }

    fn schema_param() -> ContractParam {
        let schema = schema(&DID::from(TEST_ETHR_DID), Some(SCHEMA_NAME));
        ContractParam::Bytes(serde_json::to_vec(&schema).unwrap())
    }

    mod convert_into_contract_param {
        use super::*;

        #[test]
        fn convert_schema_into_contract_param_test() {
            let schema = schema(&DID::from(TEST_ETHR_DID), Some(SCHEMA_NAME));
            let param: ContractParam = (&schema).try_into().unwrap();
            assert_eq!(schema_param(), param);
        }
    }

    mod convert_into_object {
        use super::*;

        #[test]
        fn convert_contract_output_into_schema() {
            let data = ContractOutput::new(vec![schema_param()]);
            let converted = Schema::try_from(&data).unwrap();
            let schema = schema(&DID::from(TEST_ETHR_DID), Some(SCHEMA_NAME));
            assert_eq!(schema, converted);
        }
    }
}
