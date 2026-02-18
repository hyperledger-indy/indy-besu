// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

pub mod credential_definition;
pub mod schema;

pub use credential_definition::*;
pub use schema::*;

// FIXME: network and did_method should passed as module/library settings or function params
pub const NETWORK: &'static str = "testnet";
pub const DID_METHOD: &'static str = "indy";
