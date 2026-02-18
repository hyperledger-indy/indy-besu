// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use ethereum_types::U256;
use once_cell::sync::Lazy;

/// As the network is fee and gas free, use the max available gas value for each transaction
pub const GAS: u64 = 9_007_199_254_719_927;
pub static GAS_PRICE: Lazy<U256> = Lazy::new(|| U256([0, 0, 0, 0]));
pub static GAS_LIMIT: Lazy<U256> = Lazy::new(|| U256([GAS, 0, 0, 0]));
