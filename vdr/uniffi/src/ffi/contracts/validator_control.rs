// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{
    ffi::{
        client::LedgerClient,
        error::{VdrError, VdrResult},
        transaction::Transaction,
    },
    JsonValue,
};
use indy_besu_vdr::{validator_control, Address};
use serde_json::json;

#[uniffi::export(async_runtime = "tokio")]
pub async fn build_add_validator_transaction(
    client: &LedgerClient,
    from: &str,
    validator_address: &str,
) -> VdrResult<Transaction> {
    validator_control::build_add_validator_transaction(
        &client.client,
        &Address::from(from),
        &Address::from(validator_address),
    )
    .await
    .map(Transaction::from)
    .map_err(VdrError::from)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn build_remove_validator_transaction(
    client: &LedgerClient,
    from: &str,
    validator_address: &str,
) -> VdrResult<Transaction> {
    validator_control::build_remove_validator_transaction(
        &client.client,
        &Address::from(from),
        &Address::from(validator_address),
    )
    .await
    .map(Transaction::from)
    .map_err(VdrError::from)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn build_get_validators_transaction(client: &LedgerClient) -> VdrResult<Transaction> {
    validator_control::build_get_validators_transaction(&client.client)
        .await
        .map(Transaction::from)
        .map_err(VdrError::from)
}

#[uniffi::export]
pub fn parse_get_validators_result(client: &LedgerClient, bytes: Vec<u8>) -> VdrResult<JsonValue> {
    let validators = validator_control::parse_get_validators_result(&client.client, &bytes)?;
    Ok(JsonValue::from(json!(validators)))
}
