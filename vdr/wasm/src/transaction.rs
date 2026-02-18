// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use indy_besu_vdr::{SignatureData, Transaction, TransactionEndorsingData};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

use crate::error::{JsResult, Result};

#[wasm_bindgen(js_name = Transaction)]
pub struct TransactionWrapper(pub(crate) RefCell<Transaction>);

#[wasm_bindgen(js_class = Transaction)]
impl TransactionWrapper {
    pub fn to(&self) -> Result<String> {
        Ok(self.0.borrow().to.to_string())
    }

    #[wasm_bindgen(js_name = getSigningBytes)]
    pub fn get_signing_bytes(&self) -> Result<Vec<u8>> {
        let bytes = self.0.borrow().get_signing_bytes().as_js()?;
        Ok(bytes)
    }

    #[wasm_bindgen(js_name = setSignature)]
    pub fn set_signature(&mut self, signature_data: JsValue) -> Result<()> {
        let signature_data: SignatureData = serde_wasm_bindgen::from_value(signature_data)?;
        self.0.get_mut().set_signature(signature_data);
        Ok(())
    }
}

impl From<Transaction> for TransactionWrapper {
    fn from(transaction: Transaction) -> TransactionWrapper {
        TransactionWrapper(RefCell::new(transaction))
    }
}

#[wasm_bindgen(js_name = TransactionEndorsingData)]
pub struct TransactionEndorsingDataWrapper(pub(crate) RefCell<TransactionEndorsingData>);

#[wasm_bindgen(js_class = TransactionEndorsingData)]
impl TransactionEndorsingDataWrapper {
    #[wasm_bindgen(js_name = getSigningBytes)]
    pub fn get_signing_bytes(&self) -> Result<Vec<u8>> {
        let bytes = self.0.borrow().get_signing_bytes().as_js()?;
        Ok(bytes)
    }

    #[wasm_bindgen(js_name = setSignature)]
    pub fn set_signature(&mut self, signature_data: JsValue) -> Result<()> {
        let signature_data: SignatureData = serde_wasm_bindgen::from_value(signature_data)?;
        self.0.get_mut().set_signature(signature_data);
        Ok(())
    }
}

impl From<TransactionEndorsingData> for TransactionEndorsingDataWrapper {
    fn from(data: TransactionEndorsingData) -> TransactionEndorsingDataWrapper {
        TransactionEndorsingDataWrapper(RefCell::new(data))
    }
}
