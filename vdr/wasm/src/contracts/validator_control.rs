// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use indy_besu_vdr::{validator_control, Address};
use wasm_bindgen::prelude::*;

use crate::{
    client::LedgerClientWrapper,
    error::{JsResult, Result},
    transaction::TransactionWrapper,
};

#[wasm_bindgen(js_name = ValidatorControl)]
pub struct ValidatorControl;

#[wasm_bindgen(js_class = ValidatorControl)]
impl ValidatorControl {
    #[wasm_bindgen(js_name = buildAddValidatorTransaction)]
    pub async fn build_add_validator_transaction(
        client: &LedgerClientWrapper,
        from: &str,
        validator_address: &str,
    ) -> Result<TransactionWrapper> {
        let from = Address::from(from);
        let validator_address = Address::from(validator_address);
        validator_control::build_add_validator_transaction(&client.0, &from, &validator_address)
            .await
            .as_js()
            .map(TransactionWrapper::from)
            .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = buildRemoveValidatorTransaction)]
    pub async fn build_remove_validator_transaction(
        client: &LedgerClientWrapper,
        from: &str,
        validator_address: &str,
    ) -> Result<TransactionWrapper> {
        let from = Address::from(from);
        let validator_address = Address::from(validator_address);
        validator_control::build_remove_validator_transaction(&client.0, &from, &validator_address)
            .await
            .as_js()
            .map(TransactionWrapper::from)
            .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = buildGetValidatorsTransaction)]
    pub async fn build_get_validators_transaction(
        client: &LedgerClientWrapper,
    ) -> Result<TransactionWrapper> {
        validator_control::build_get_validators_transaction(&client.0)
            .await
            .as_js()
            .map(TransactionWrapper::from)
            .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = parseGetValidatorsResult)]
    pub fn parse_get_validators_result(
        client: &LedgerClientWrapper,
        bytes: Vec<u8>,
    ) -> Result<JsValue> {
        let validators =
            validator_control::parse_get_validators_result(&client.0, &bytes).as_js()?;
        let result: JsValue = serde_wasm_bindgen::to_value(&validators)?;
        Ok(result)
    }
}
