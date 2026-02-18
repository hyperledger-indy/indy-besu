// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{
    ffi::{
        client::LedgerClient,
        endorsing_data::TransactionEndorsingData,
        error::{VdrError, VdrResult},
        transaction::Transaction,
    },
    JsonValue,
};

use indy_besu_vdr::{
    credential_definition_registry, Address, CredentialDefinition as CredentialDefinition_,
    CredentialDefinitionId, SchemaId, SignatureType, DID,
};
use serde_json::json;

#[uniffi::export(async_runtime = "tokio")]
pub async fn build_create_credential_definition_transaction(
    client: &LedgerClient,
    from: &str,
    credential_definition: &CredentialDefinition,
) -> VdrResult<Transaction> {
    credential_definition_registry::build_create_credential_definition_transaction(
        &client.client,
        &Address::from(from),
        &CredentialDefinition_::from(credential_definition),
    )
    .await
    .map(Transaction::from)
    .map_err(VdrError::from)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn build_create_credential_definition_endorsing_data(
    client: &LedgerClient,
    credential_definition: &CredentialDefinition,
) -> VdrResult<TransactionEndorsingData> {
    credential_definition_registry::build_create_credential_definition_endorsing_data(
        &client.client,
        &CredentialDefinition_::from(credential_definition),
    )
    .await
    .map(TransactionEndorsingData::from)
    .map_err(VdrError::from)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn build_resolve_credential_definition_transaction(
    client: &LedgerClient,
    id: &str,
) -> VdrResult<Transaction> {
    credential_definition_registry::build_resolve_credential_definition_transaction(
        &client.client,
        &CredentialDefinitionId::from(id),
    )
    .await
    .map(Transaction::from)
    .map_err(VdrError::from)
}

#[uniffi::export]
pub fn parse_resolve_credential_definition_result(
    client: &LedgerClient,
    bytes: Vec<u8>,
) -> VdrResult<JsonValue> {
    let cred_def = credential_definition_registry::parse_resolve_credential_definition_result(
        &client.client,
        &bytes,
    )?;
    Ok(JsonValue::from(json!(cred_def)))
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn resolve_credential_definition(
    client: &LedgerClient,
    id: &str,
) -> VdrResult<CredentialDefinition> {
    credential_definition_registry::resolve_credential_definition(
        &client.client,
        &CredentialDefinitionId::from(id),
    )
    .await
    .map(CredentialDefinition::from)
    .map_err(VdrError::from)
}

#[derive(uniffi::Record)]
pub struct CredentialDefinition {
    pub issuer_id: String,
    pub schema_id: String,
    pub cred_def_type: String,
    pub tag: String,
    pub value: JsonValue,
}

impl From<CredentialDefinition_> for CredentialDefinition {
    fn from(cred_def: CredentialDefinition_) -> Self {
        CredentialDefinition {
            issuer_id: cred_def.issuer_id.as_ref().to_string(),
            schema_id: cred_def.schema_id.as_ref().to_string(),
            cred_def_type: cred_def.cred_def_type.to_str().to_string(),
            tag: cred_def.tag.to_string(),
            value: JsonValue::from(cred_def.value),
        }
    }
}

impl From<&CredentialDefinition> for CredentialDefinition_ {
    fn from(cred_def: &CredentialDefinition) -> Self {
        CredentialDefinition_ {
            issuer_id: DID::from(cred_def.issuer_id.as_str()),
            schema_id: SchemaId::from(cred_def.schema_id.as_ref()),
            cred_def_type: SignatureType::CL,
            tag: cred_def.tag.to_string(),
            value: cred_def.value.clone().into_inner(),
        }
    }
}

#[uniffi::export]
pub fn credential_definition_get_id(cred_def: &CredentialDefinition) -> String {
    CredentialDefinition_::from(cred_def)
        .id()
        .as_ref()
        .to_string()
}

#[uniffi::export]
pub fn credential_definition_to_string(data: &CredentialDefinition) -> VdrResult<String> {
    CredentialDefinition_::from(data)
        .to_string()
        .map_err(VdrError::from)
}

#[uniffi::export]
pub fn credential_definition_from_string(string: &str) -> VdrResult<CredentialDefinition> {
    CredentialDefinition_::from_string(string)
        .map(CredentialDefinition::from)
        .map_err(VdrError::from)
}
