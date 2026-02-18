// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{
    contracts::did::types::did::DID,
    error::{VdrError, VdrResult},
    CredentialDefinition, SchemaId,
};
use log_derive::{logfn, logfn_inputs};
use serde_json::json;

use indy_data_types::{
    anoncreds::cred_def::{
        CredentialDefinition as IndyCredentialDefinition,
        CredentialDefinitionV1 as IndyCredentialDefinitionV1,
    },
    did::DidValue,
    CredentialDefinitionId as IndyCredentialDefinitionId, SchemaId as IndySchemaId,
};

impl CredentialDefinition {
    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub fn from_indy_format_str(
        cred_def: &str,
        issuer_did: &str,
        schema_id: &str,
    ) -> VdrResult<CredentialDefinition> {
        let cred_def: IndyCredentialDefinition = serde_json::from_str(cred_def).map_err(|err| {
            VdrError::CommonInvalidData(format!(
                "Unable to parse indy credential definition. Err: {:?}",
                err
            ))
        })?;
        CredentialDefinition::from_indy_format(&cred_def, issuer_did, schema_id)
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub fn from_indy_format(
        cred_def: &IndyCredentialDefinition,
        issuer_did: &str,
        schema_id: &str,
    ) -> VdrResult<CredentialDefinition> {
        match cred_def {
            IndyCredentialDefinition::CredentialDefinitionV1(cred_def) => {
                let besu_schema = CredentialDefinition {
                    issuer_id: DID::from(issuer_did),
                    schema_id: SchemaId::from(schema_id),
                    cred_def_type: cred_def.signature_type.clone(),
                    tag: cred_def.tag.to_string(),
                    value: json!(cred_def.value),
                };

                Ok(besu_schema)
            }
        }
    }
}

impl Into<IndyCredentialDefinition> for &CredentialDefinition {
    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    fn into(self) -> IndyCredentialDefinition {
        let value = serde_json::from_str(&self.value.to_string())
            .expect("Unable to parse credential definition data");

        IndyCredentialDefinition::CredentialDefinitionV1(IndyCredentialDefinitionV1 {
            id: IndyCredentialDefinitionId::new(
                &DidValue(self.issuer_id.to_string()),
                &IndySchemaId(self.schema_id.to_string()),
                &self.cred_def_type.to_str(),
                &self.tag,
            ),
            schema_id: IndySchemaId(self.schema_id.to_string()),
            signature_type: self.cred_def_type.clone(),
            tag: self.tag.to_string(),
            value,
        })
    }
}
