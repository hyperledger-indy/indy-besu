// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::ffi::{
    client::LedgerClient,
    endorsing_data::TransactionEndorsingData,
    error::{VdrError, VdrResult},
    transaction::Transaction,
};
use indy_besu_vdr::{endorsing, Address};

#[uniffi::export(async_runtime = "tokio")]
pub async fn build_endorsement_transaction(
    client: &LedgerClient,
    from: &str,
    endorsing_data: &TransactionEndorsingData,
) -> VdrResult<Transaction> {
    endorsing::build_endorsement_transaction(
        &client.client,
        &Address::from(from),
        &endorsing_data.into(),
    )
    .await
    .map(Transaction::from)
    .map_err(VdrError::from)
}
