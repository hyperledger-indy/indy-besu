// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{types::ContractParam, VdrError};

/// Wrapper structure for ED25519 signature
#[derive(Debug, Default)]
pub struct Ed25519Signature(Vec<u8>);

impl From<&[u8]> for Ed25519Signature {
    fn from(value: &[u8]) -> Self {
        Ed25519Signature(value.to_vec())
    }
}

impl TryFrom<&Ed25519Signature> for ContractParam {
    type Error = VdrError;

    fn try_from(value: &Ed25519Signature) -> Result<Self, Self::Error> {
        Ok(ContractParam::Bytes(value.0.to_vec()))
    }
}
