// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use ethabi::Uint;
use log_derive::{logfn, logfn_inputs};
use serde::{Deserialize, Serialize};
use sha3::Digest;
use std::fmt::Debug;

use crate::{
    types::{ContractParam, MethodStringParam, MethodUintBytesParam},
    Address, LedgerClient, Nonce, SignatureData, VdrError, VdrResult,
};

/// Definition of transaction endorsing data for off-chain author signature
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TransactionEndorsingData {
    pub to: Address,
    pub from: Address,
    pub nonce: Option<Nonce>,
    pub contract: String,
    pub method: String,
    pub endorsing_method: String,
    pub params: Vec<ContractParam>,
    pub signature: Option<SignatureData>,
}

impl TransactionEndorsingData {
    const PREFIX: u8 = 0x19;
    const VERSION: u8 = 0x0;

    /// Get transaction bytes which are need to be signed by the author before passing it to sender who will submit transaction on the ledger
    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub fn get_signing_bytes(&self) -> VdrResult<Vec<u8>> {
        let mut tokens = vec![
            ContractParam::Uint(Uint::from(Self::PREFIX)),
            ContractParam::FixedBytes(vec![Self::VERSION]),
            (&self.to).try_into()?,
        ];
        if let Some(ref nonce) = self.nonce {
            tokens.push(nonce.try_into()?)
        }
        tokens.push((&self.from).try_into()?);
        tokens.push((&MethodStringParam::from(self.method.as_str())).try_into()?);
        for param in self.params.iter() {
            match param {
                ContractParam::Uint(uint) => {
                    // uint values represented as bytes on endorsing signature verification
                    tokens.push((&MethodUintBytesParam::from(uint.as_u64())).try_into()?)
                }
                _ => tokens.push(param.to_owned()),
            };
        }

        let encoded = ethers_core::abi::encode_packed(&tokens).unwrap();
        let hash = sha3::Keccak256::digest(encoded).to_vec();
        Ok(hash)
    }

    /// Set author's transaction signature
    #[logfn(Info)]
    #[logfn_inputs(Debug)]
    pub fn set_signature(&mut self, signature_data: SignatureData) {
        self.signature = Some(signature_data)
    }

    /// Serialize transaction endorsement as JSON string
    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub fn to_string(&self) -> VdrResult<String> {
        serde_json::to_string(self).map_err(|err| {
            VdrError::ClientInvalidEndorsementData(format!(
                "Unable to serialize endorsement data as JSON. Err: {:?}",
                err
            ))
        })
    }

    /// Deserialize transaction endorsement data from JSON string
    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub fn from_string(value: &str) -> VdrResult<Self> {
        serde_json::from_str(value).map_err(|err| {
            VdrError::ClientInvalidEndorsementData(format!(
                "Unable to deserialize endorsement data from JSON. Err: {:?}",
                err
            ))
        })
    }
}

#[derive(Debug, Default)]
pub(crate) struct TransactionEndorsingDataBuilder {
    contract: String,
    identity: Address,
    nonce: Option<Nonce>,
    method: String,
    endorsing_method: String,
    params: Vec<ContractParam>,
}

impl TransactionEndorsingDataBuilder {
    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub fn new() -> TransactionEndorsingDataBuilder {
        TransactionEndorsingDataBuilder::default()
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub fn set_contract(mut self, contract: &str) -> TransactionEndorsingDataBuilder {
        self.contract = contract.to_string();
        self
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub fn set_identity(mut self, identity: &Address) -> TransactionEndorsingDataBuilder {
        self.identity = identity.to_owned();
        self
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub fn set_method(mut self, method: &str) -> TransactionEndorsingDataBuilder {
        self.method = method.to_owned();
        self
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub fn set_endorsing_method(mut self, method: &str) -> TransactionEndorsingDataBuilder {
        self.endorsing_method = method.to_owned();
        self
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub fn set_nonce(mut self, nonce: &Nonce) -> TransactionEndorsingDataBuilder {
        self.nonce = Some(nonce.to_owned());
        self
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub fn add_param<T: TryInto<ContractParam, Error = VdrError> + Debug>(
        mut self,
        param: T,
    ) -> VdrResult<TransactionEndorsingDataBuilder> {
        self.params.push(param.try_into()?);
        Ok(self)
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub async fn build(self, client: &LedgerClient) -> VdrResult<TransactionEndorsingData> {
        let contract = client.contract(&self.contract)?;
        Ok(TransactionEndorsingData {
            to: contract.address().to_owned(),
            from: self.identity.to_owned(),
            nonce: self.nonce,
            method: self.method,
            contract: self.contract,
            endorsing_method: self.endorsing_method,
            params: self.params,
            signature: Default::default(),
        })
    }
}
