// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

mod common;

use log_derive::{logfn, logfn_inputs};
use std::{collections::HashSet, hash::Hash};

use crate::{VdrError, VdrResult};
#[cfg(test)]
pub use common::{init_env_logger, rand_string};

#[logfn(Trace)]
#[logfn_inputs(Trace)]
pub fn format_bytes32_string(string: &str) -> VdrResult<[u8; 32]> {
    let str_bytes = string.as_bytes();
    if str_bytes.len() > 32 {
        return Err(VdrError::CommonInvalidData(
            "Unable to represent string as bytes32".to_string(),
        ));
    }

    let mut bytes32: [u8; 32] = [0u8; 32];
    bytes32[..str_bytes.len()].copy_from_slice(str_bytes);

    Ok(bytes32)
}

#[logfn(Trace)]
#[logfn_inputs(Trace)]
pub fn format_bytes32(bytes: &[u8]) -> VdrResult<[u8; 32]> {
    if bytes.len() > 32 {
        return Err(VdrError::CommonInvalidData(
            "Unable to represent string as bytes32".to_string(),
        ));
    }

    let mut bytes32: [u8; 32] = [0u8; 32];
    bytes32[32 - bytes.len()..32].copy_from_slice(bytes);

    Ok(bytes32)
}

#[logfn(Trace)]
#[logfn_inputs(Trace)]
pub fn parse_bytes32_string(bytes: &[u8]) -> VdrResult<&str> {
    let mut length = 0;
    while length < 32 && bytes[length] != 0 {
        length += 1;
    }

    std::str::from_utf8(&bytes[..length]).map_err(|err| {
        VdrError::CommonInvalidData(format!(
            "Unable to decode string from bytes. Err: {:?}",
            err
        ))
    })
}

pub fn is_unique<T>(iter: T) -> bool
where
    T: IntoIterator,
    T::Item: Eq + Hash,
{
    let mut unique = HashSet::new();
    iter.into_iter().all(|item| unique.insert(item))
}
