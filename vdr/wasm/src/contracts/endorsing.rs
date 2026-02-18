// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use indy_besu_vdr::{endorsing, Address};
use wasm_bindgen::prelude::*;

use crate::{
    client::LedgerClientWrapper,
    error::{JsResult, Result},
    transaction::{TransactionEndorsingDataWrapper, TransactionWrapper},
};

#[wasm_bindgen(js_name = Endorsement)]
pub struct Endorsement;

#[wasm_bindgen(js_class = Endorsement)]
impl Endorsement {
    #[wasm_bindgen(js_name = buildEndorsementTransaction)]
    pub async fn build_endorsement_transaction(
        client: &LedgerClientWrapper,
        from: &str,
        endorsing_data: &TransactionEndorsingDataWrapper,
    ) -> Result<TransactionWrapper> {
        let from = Address::from(from);
        endorsing::build_endorsement_transaction(&client.0, &from, &endorsing_data.0.borrow())
            .await
            .as_js()
            .map(TransactionWrapper::from)
            .map_err(JsValue::from)
    }
}
