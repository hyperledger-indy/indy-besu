// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use js_sys::Promise;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

use indy_besu_vdr::{ContractConfig, LedgerClient, QuorumConfig};

use crate::{
    error::{JsResult, Result},
    event_query::EventQueryWrapper,
    transaction::TransactionWrapper,
};

#[wasm_bindgen(js_name = LedgerClient)]
pub struct LedgerClientWrapper(pub(crate) Rc<LedgerClient>);

#[wasm_bindgen(js_class = LedgerClient)]
impl LedgerClientWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new(
        chain_id: u32,
        node_address: String,
        contract_configs: JsValue,
        network: Option<String>,
        quorum_config: JsValue,
    ) -> Result<LedgerClientWrapper> {
        console_error_panic_hook::set_once();
        let contract_configs: Vec<ContractConfig> =
            serde_wasm_bindgen::from_value(contract_configs)?;
        let quorum_config: Option<QuorumConfig> =
            serde_wasm_bindgen::from_value(quorum_config).ok();
        let client = LedgerClient::new(
            chain_id as u64,
            &node_address,
            &contract_configs,
            network.as_deref(),
            quorum_config.as_ref(),
        )
        .as_js()?;
        Ok(LedgerClientWrapper(Rc::new(client)))
    }

    pub async fn ping(&self) -> Promise {
        let client = self.0.clone();
        future_to_promise(async move {
            let status = client.ping().await.as_js()?;
            let result: JsValue = serde_wasm_bindgen::to_value(&status)?;
            Ok(result)
        })
    }

    #[wasm_bindgen(js_name = submitTransaction)]
    pub async fn submit_transaction(&self, transaction: &TransactionWrapper) -> Promise {
        let client = self.0.clone();
        let transaction = transaction.0.clone();
        future_to_promise(async move {
            let transaction = transaction.borrow();
            let response = client.submit_transaction(&transaction).await.as_js()?;
            let result: JsValue = serde_wasm_bindgen::to_value(&response)?;
            Ok(result)
        })
    }

    #[wasm_bindgen(js_name = queryEvents)]
    pub async fn query_events(&self, query: &EventQueryWrapper) -> Promise {
        let client = self.0.clone();
        let query = query.0.clone();
        future_to_promise(async move {
            let response = client.query_events(&query).await.as_js()?;
            let result: JsValue = serde_wasm_bindgen::to_value(&response)?;
            Ok(result)
        })
    }

    #[wasm_bindgen(js_name = getReceipt)]
    pub async fn get_receipt(&self, hash: Vec<u8>) -> Promise {
        let client = self.0.clone();
        future_to_promise(async move {
            let receipt = client.get_receipt(&hash).await.as_js()?;
            let result: JsValue = serde_wasm_bindgen::to_value(&receipt)?;
            Ok(result)
        })
    }
}
