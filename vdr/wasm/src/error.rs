// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use indy_besu_vdr::VdrResult;
use js_sys::Error as JsError;
use wasm_bindgen::JsValue;

pub type Result<T> = core::result::Result<T, JsValue>;

pub(crate) trait JsResult<T> {
    #[allow(clippy::wrong_self_convention)]
    fn as_js(self) -> core::result::Result<T, JsError>;
}

impl<T> JsResult<T> for VdrResult<T> {
    fn as_js(self) -> core::result::Result<T, JsError> {
        self.map_err(|e| JsError::new(&e.to_string()))
    }
}
