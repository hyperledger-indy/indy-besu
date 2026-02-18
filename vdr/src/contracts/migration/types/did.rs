// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{types::ContractParam, VdrError};
use serde_derive::{Deserialize, Serialize};

/// Wrapper structure for legacy Indy DID identifier
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct LegacyDid(String);

impl AsRef<str> for LegacyDid {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl ToString for LegacyDid {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl From<&str> for LegacyDid {
    fn from(value: &str) -> Self {
        LegacyDid(value.to_string())
    }
}

impl TryFrom<&LegacyDid> for ContractParam {
    type Error = VdrError;

    fn try_from(value: &LegacyDid) -> Result<Self, Self::Error> {
        Ok(ContractParam::String(value.as_ref().to_string()))
    }
}

/// Wrapper structure for legacy ED25519 verification key
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct LegacyVerkey(String);

impl From<&str> for LegacyVerkey {
    fn from(value: &str) -> Self {
        LegacyVerkey(value.to_string())
    }
}

impl TryFrom<&LegacyVerkey> for ContractParam {
    type Error = VdrError;

    fn try_from(value: &LegacyVerkey) -> Result<Self, Self::Error> {
        let legacy_identifier = bs58::decode(&value.0).into_vec().map_err(|err| {
            VdrError::CommonInvalidData(format!(
                "Unable to decode base58 verkey key. Err: {:?}",
                err
            ))
        })?;
        Ok(ContractParam::FixedBytes(legacy_identifier))
    }
}
