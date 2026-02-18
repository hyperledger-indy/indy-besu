// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use indy_besu_vdr::{did_resolver, DidResolutionOptions, DID};
use wasm_bindgen::prelude::*;

use crate::{
    client::LedgerClientWrapper,
    error::{JsResult, Result},
};

#[wasm_bindgen(js_name = DidResolver)]
pub struct DidResolver;

#[wasm_bindgen(js_class = DidResolver)]
impl DidResolver {
    #[wasm_bindgen(js_name = resolveDid)]
    pub async fn resolve_did(
        client: &LedgerClientWrapper,
        did: &str,
        options: JsValue,
    ) -> Result<JsValue> {
        let did = DID::from(did);
        let options: Option<DidResolutionOptions> = serde_wasm_bindgen::from_value(options).ok();
        let did_with_meta = did_resolver::resolve_did(&client.0, &did, options.as_ref())
            .await
            .as_js()?;
        let result: JsValue = serde_wasm_bindgen::to_value(&did_with_meta)?;
        Ok(result)
    }
}
