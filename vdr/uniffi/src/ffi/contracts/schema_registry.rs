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
use indy_besu_vdr::{schema_registry, Address, Schema as Schema_, SchemaId, DID};
use serde_json::json;
use std::collections::HashSet;

#[uniffi::export(async_runtime = "tokio")]
pub async fn build_create_schema_transaction(
    client: &LedgerClient,
    from: &str,
    schema: &Schema,
) -> VdrResult<Transaction> {
    schema_registry::build_create_schema_transaction(
        &client.client,
        &Address::from(from),
        &Schema_::from(schema),
    )
    .await
    .map(Transaction::from)
    .map_err(VdrError::from)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn build_create_schema_endorsing_data(
    client: &LedgerClient,
    schema: &Schema,
) -> VdrResult<TransactionEndorsingData> {
    schema_registry::build_create_schema_endorsing_data(&client.client, &Schema_::from(schema))
        .await
        .map(TransactionEndorsingData::from)
        .map_err(VdrError::from)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn build_resolve_schema_transaction(
    client: &LedgerClient,
    id: &str,
) -> VdrResult<Transaction> {
    schema_registry::build_resolve_schema_transaction(&client.client, &SchemaId::from(id))
        .await
        .map(Transaction::from)
        .map_err(VdrError::from)
}

#[uniffi::export]
pub fn parse_resolve_schema_result(client: &LedgerClient, bytes: Vec<u8>) -> VdrResult<JsonValue> {
    let record = schema_registry::parse_resolve_schema_result(&client.client, &bytes)?;
    Ok(JsonValue::from(json!(record)))
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn resolve_schema(client: &LedgerClient, id: &str) -> VdrResult<Schema> {
    schema_registry::resolve_schema(&client.client, &SchemaId::from(id))
        .await
        .map(Schema::from)
        .map_err(VdrError::from)
}

#[derive(uniffi::Record)]
pub struct Schema {
    pub issuer_id: String,
    pub name: String,
    pub version: String,
    pub attr_names: Vec<String>,
}

impl From<Schema_> for Schema {
    fn from(schema: Schema_) -> Self {
        Schema {
            issuer_id: schema.issuer_id.as_ref().to_string(),
            name: schema.name,
            version: schema.version,
            attr_names: schema.attr_names.iter().cloned().collect(),
        }
    }
}

impl From<&Schema> for Schema_ {
    fn from(schema: &Schema) -> Self {
        Schema_ {
            issuer_id: DID::from(schema.issuer_id.as_str()),
            name: schema.name.to_string(),
            version: schema.version.to_string(),
            attr_names: HashSet::from_iter(schema.attr_names.iter().cloned()),
        }
    }
}

#[uniffi::export]
pub fn schema_get_id(schema: &Schema) -> String {
    Schema_::from(schema).id().as_ref().to_string()
}

#[uniffi::export]
pub fn schema_to_string(data: &Schema) -> VdrResult<String> {
    Schema_::from(data).to_string().map_err(VdrError::from)
}

#[uniffi::export]
pub fn schema_from_string(string: &str) -> VdrResult<Schema> {
    Schema_::from_string(string)
        .map(Schema::from)
        .map_err(VdrError::from)
}
