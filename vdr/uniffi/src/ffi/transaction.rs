// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::ffi::{
    error::{VdrError, VdrResult},
    types::{SignatureData, TransactionType},
};
use indy_besu_vdr::{Address, Transaction as Transaction_};

#[derive(uniffi::Record)]
pub struct Transaction {
    pub type_: TransactionType,
    pub from: Option<String>,
    pub to: String,
    pub nonce: Option<u64>,
    pub chain_id: u64,
    pub data: Vec<u8>,
    pub signature: Option<SignatureData>,
    pub hash: Option<Vec<u8>>,
}

impl From<Transaction_> for Transaction {
    fn from(transaction: Transaction_) -> Self {
        Transaction {
            type_: TransactionType::from(&transaction.type_),
            from: transaction.from.map(|from| from.as_ref().to_string()),
            to: transaction.to.to_string(),
            nonce: transaction.nonce,
            chain_id: transaction.chain_id,
            data: transaction.data,
            signature: transaction.signature.as_ref().map(SignatureData::from),
            hash: transaction.hash,
        }
    }
}

impl From<&Transaction> for Transaction_ {
    fn from(transaction: &Transaction) -> Self {
        Transaction_ {
            type_: (&transaction.type_).into(),
            from: transaction.from.as_deref().map(Address::from),
            to: Address::from(transaction.to.as_ref()),
            nonce: transaction.nonce.clone(),
            chain_id: transaction.chain_id,
            data: transaction.data.to_owned(),
            signature: transaction.signature.as_ref().map(|data| data.into()),
            hash: transaction.hash.to_owned(),
        }
    }
}

#[uniffi::export]
pub fn transaction_create(
    type_: TransactionType,
    to: &str,
    from: Option<String>,
    nonce: Option<u64>,
    chain_id: u64,
    data: Vec<u8>,
    signature: Option<SignatureData>,
    hash: Option<Vec<u8>>,
) -> Transaction {
    Transaction {
        type_,
        to: to.to_string(),
        from: from,
        nonce,
        chain_id,
        data,
        signature,
        hash,
    }
}

#[uniffi::export]
pub fn transaction_get_signing_bytes(data: &Transaction) -> VdrResult<Vec<u8>> {
    Transaction_::from(data)
        .get_signing_bytes()
        .map_err(VdrError::from)
}

#[uniffi::export]
pub fn transaction_to_string(data: &Transaction) -> VdrResult<String> {
    Transaction_::from(data).to_string().map_err(VdrError::from)
}

#[uniffi::export]
pub fn transaction_from_string(value: &str) -> VdrResult<Transaction> {
    Transaction_::from_string(value)
        .map(Transaction::from)
        .map_err(VdrError::from)
}
