// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use serde_derive::{Deserialize, Serialize};

/// Ledger status: whether connected node and network are alive
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PingStatus {
    pub status: Status,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Status {
    Ok {
        block_number: u64,
        block_timestamp: u64,
    },
    Err {
        msg: String,
    },
}

impl PingStatus {
    pub fn ok(block_number: u64, block_timestamp: u64) -> PingStatus {
        PingStatus {
            status: Status::Ok {
                block_number,
                block_timestamp,
            },
        }
    }

    pub fn err(err: &str) -> PingStatus {
        PingStatus {
            status: Status::Err {
                msg: err.to_string(),
            },
        }
    }
}
