// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{
    types::TransactionBuilder, Address, LedgerClient, Transaction, TransactionEndorsingData,
    TransactionType, VdrError, VdrResult,
};
use log_derive::{logfn, logfn_inputs};

/// Build transaction to endorse author prepared transaction
///
/// # Params
/// - `client` client connected to the network where contract will be executed
/// - `from` transaction sender account address
/// - `endorsing_data` prepared transaction endorsing data.
///
/// # Returns
/// Write transaction to sign and submit
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_endorsement_transaction(
    client: &LedgerClient,
    from: &Address,
    endorsing_data: &TransactionEndorsingData,
) -> VdrResult<Transaction> {
    let signature = endorsing_data
        .signature
        .as_ref()
        .ok_or_else(|| {
            VdrError::ClientInvalidEndorsementData("Missing author signature".to_string())
        })?
        .clone();

    TransactionBuilder::new()
        .set_contract(&endorsing_data.contract)
        .set_method(&endorsing_data.endorsing_method)
        .add_param(&endorsing_data.from)?
        .add_param(&signature.v())?
        .add_param(&signature.r())?
        .add_param(&signature.s())?
        .add_contract_params(endorsing_data.params.as_slice())?
        .set_type(TransactionType::Write)
        .set_from(from)
        .build(client)
        .await
}
