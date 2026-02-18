// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{types::ContractParam, VdrError};
use serde_derive::{Deserialize, Serialize};

/// Definition of recoverable ECDSA signature
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignatureData {
    /// recovery ID using for public key recovery
    pub recovery_id: u64,
    /// ECDSA signature
    pub signature: Vec<u8>,
}

impl SignatureData {
    pub(crate) fn v(&self) -> SignatureV {
        SignatureV(self.recovery_id)
    }

    pub(crate) fn r(&self) -> SignatureR {
        SignatureR(self.signature[..32].to_vec())
    }

    pub(crate) fn s(&self) -> SignatureS {
        SignatureS(self.signature[32..].to_vec())
    }
}

#[derive(Debug)]
pub(crate) struct SignatureV(pub u64);

impl TryFrom<&SignatureV> for ContractParam {
    type Error = VdrError;

    fn try_from(value: &SignatureV) -> Result<Self, Self::Error> {
        Ok(ContractParam::Uint((value.0 + 27).into()))
    }
}

#[derive(Debug)]
pub(crate) struct SignatureR(pub Vec<u8>);

impl TryFrom<&SignatureR> for ContractParam {
    type Error = VdrError;

    fn try_from(value: &SignatureR) -> Result<Self, Self::Error> {
        Ok(ContractParam::FixedBytes(value.0.to_vec()))
    }
}

#[derive(Debug)]
pub(crate) struct SignatureS(pub Vec<u8>);

impl TryFrom<&SignatureS> for ContractParam {
    type Error = VdrError;

    fn try_from(value: &SignatureS) -> Result<Self, Self::Error> {
        Ok(ContractParam::FixedBytes(value.0.to_vec()))
    }
}
