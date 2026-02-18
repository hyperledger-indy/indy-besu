// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{
    contracts::did::types::did::DID,
    error::{VdrError, VdrResult},
    Schema,
};
use indy_data_types::{
    anoncreds::schema::{AttributeNames, Schema as IndySchema, SchemaV1 as IndySchemaV1},
    did::DidValue,
    SchemaId as IndySchemaId,
};
use log_derive::{logfn, logfn_inputs};

impl Schema {
    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub fn from_indy_format_str(schema: &str, issuer_did: &str) -> VdrResult<Schema> {
        let schema: IndySchema = serde_json::from_str(schema).map_err(|err| {
            VdrError::CommonInvalidData(format!("Unable to parse indy schema. Err: {:?}", err))
        })?;
        Schema::from_indy_format(&schema, issuer_did)
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub fn from_indy_format(schema: &IndySchema, issuer_did: &str) -> VdrResult<Schema> {
        match schema {
            IndySchema::SchemaV1(schema) => {
                let besu_schema = Schema {
                    issuer_id: DID::from(issuer_did),
                    name: schema.name.to_string(),
                    version: schema.version.to_string(),
                    attr_names: schema.attr_names.clone().0,
                };

                Ok(besu_schema)
            }
        }
    }
}

impl Into<IndySchema> for &Schema {
    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    fn into(self) -> IndySchema {
        IndySchema::SchemaV1(IndySchemaV1 {
            id: IndySchemaId::new(
                &DidValue(self.issuer_id.to_string()),
                &self.name,
                &self.version,
            ),
            name: self.name.to_string(),
            version: self.version.to_string(),
            attr_names: AttributeNames(self.attr_names.clone()),
            seq_no: None,
        })
    }
}
