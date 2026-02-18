// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

pub mod did_ethr_registry;
pub mod did_indy_registry;
pub mod did_resolver;
pub mod types;

pub use did_ethr_registry::*;
pub use did_indy_registry::*;
pub use types::{did::DID, did_doc::*, did_doc_attribute::*};
