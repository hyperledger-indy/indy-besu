// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::ffi::{
    client::LedgerClient,
    error::{VdrError, VdrResult},
    transaction::Transaction,
};
use indy_besu_vdr::{role_control, Address, Role};

#[uniffi::export(async_runtime = "tokio")]
pub async fn build_assign_role_transaction(
    client: &LedgerClient,
    from: &str,
    role: u8,
    account: &str,
) -> VdrResult<Transaction> {
    role_control::build_assign_role_transaction(
        &client.client,
        &Address::from(from),
        &Role::try_from(role)?,
        &Address::from(account),
    )
    .await
    .map(Transaction::from)
    .map_err(VdrError::from)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn build_revoke_role_transaction(
    client: &LedgerClient,
    from: &str,
    role: u8,
    account: &str,
) -> VdrResult<Transaction> {
    role_control::build_revoke_role_transaction(
        &client.client,
        &Address::from(from),
        &Role::try_from(role)?,
        &Address::from(account),
    )
    .await
    .map(Transaction::from)
    .map_err(VdrError::from)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn build_has_role_transaction(
    client: &LedgerClient,
    role: u8,
    account: &str,
) -> VdrResult<Transaction> {
    role_control::build_has_role_transaction(
        &client.client,
        &Role::try_from(role)?,
        &Address::from(account),
    )
    .await
    .map(Transaction::from)
    .map_err(VdrError::from)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn build_get_role_transaction(
    client: &LedgerClient,
    account: &str,
) -> VdrResult<Transaction> {
    role_control::build_get_role_transaction(&client.client, &Address::from(account))
        .await
        .map(Transaction::from)
        .map_err(VdrError::from)
}

#[uniffi::export]
pub fn parse_has_role_result(client: &LedgerClient, bytes: Vec<u8>) -> VdrResult<bool> {
    role_control::parse_has_role_result(&client.client, &bytes).map_err(VdrError::from)
}

#[uniffi::export]
pub fn parse_get_role_result(client: &LedgerClient, bytes: Vec<u8>) -> VdrResult<u8> {
    let role = role_control::parse_get_role_result(&client.client, &bytes)?;
    Ok(role as u8)
}
