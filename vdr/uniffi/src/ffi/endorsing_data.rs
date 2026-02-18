// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{JsonValue, SignatureData, VdrError, VdrResult};
use indy_besu_vdr::{
    Address, Nonce, SignatureData as SignatureData_,
    TransactionEndorsingData as TransactionEndorsingData_,
};
use serde_json::json;

#[derive(uniffi::Record)]
pub struct TransactionEndorsingData {
    pub to: String,
    pub from: String,
    pub nonce: Option<u64>,
    pub contract: String,
    pub method: String,
    pub endorsing_method: String,
    pub params: Vec<JsonValue>,
    pub signature: Option<SignatureData>,
}

#[uniffi::export]
pub fn transaction_endorsing_data_create(
    to: &str,
    from: &str,
    contract: &str,
    method: &str,
    endorsing_method: &str,
    params: Vec<JsonValue>,
    nonce: Option<u64>,
    signature: Option<SignatureData>,
) -> TransactionEndorsingData {
    TransactionEndorsingData {
        to: to.to_string(),
        from: from.to_string(),
        nonce,
        contract: contract.to_string(),
        method: method.to_string(),
        endorsing_method: endorsing_method.to_string(),
        params,
        signature,
    }
}

#[uniffi::export]
pub fn transaction_endorsing_data_get_signing_bytes(
    data: &TransactionEndorsingData,
) -> VdrResult<Vec<u8>> {
    TransactionEndorsingData_::from(data)
        .get_signing_bytes()
        .map_err(VdrError::from)
}

#[uniffi::export]
pub fn transaction_endorsing_data_to_string(data: &TransactionEndorsingData) -> VdrResult<String> {
    TransactionEndorsingData_::from(data)
        .to_string()
        .map_err(VdrError::from)
}

#[uniffi::export]
pub fn transaction_endorsing_data_from_string(value: &str) -> VdrResult<TransactionEndorsingData> {
    TransactionEndorsingData_::from_string(value)
        .map(TransactionEndorsingData::from)
        .map_err(VdrError::from)
}

impl From<TransactionEndorsingData_> for TransactionEndorsingData {
    fn from(data: TransactionEndorsingData_) -> Self {
        TransactionEndorsingData {
            to: data.to.as_ref().to_string(),
            from: data.from.as_ref().to_string(),
            nonce: data.nonce.map(|nonce| nonce.value()),
            contract: data.contract.to_string(),
            method: data.method.to_string(),
            endorsing_method: data.endorsing_method.to_string(),
            params: data
                .params
                .into_iter()
                .map(|param| JsonValue::from(json!(param)))
                .collect(),
            signature: data.signature.as_ref().map(|data| data.into()),
        }
    }
}

impl From<&TransactionEndorsingData> for TransactionEndorsingData_ {
    fn from(data: &TransactionEndorsingData) -> Self {
        TransactionEndorsingData_ {
            to: Address::from(data.to.as_ref()),
            from: Address::from(data.from.as_ref()),
            nonce: data.nonce.map(Nonce::from),
            contract: data.contract.to_string(),
            method: data.method.to_string(),
            endorsing_method: data.endorsing_method.to_string(),
            params: data
                .params
                .iter()
                .flat_map(|param| serde_json::from_value(param.to_owned().into_inner()))
                .collect(),
            signature: data.signature.as_ref().map(|signature| SignatureData_ {
                recovery_id: signature.recovery_id.clone(),
                signature: signature.signature.clone(),
            }),
        }
    }
}
