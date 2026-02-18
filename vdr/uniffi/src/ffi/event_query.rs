// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use indy_besu_vdr::{Address, Block, EventLog as EventLog_, EventQuery as EventQuery_};

#[derive(uniffi::Record)]
pub struct EventQuery {
    pub address: String,
    pub from_block: Option<u64>,
    pub to_block: Option<u64>,
    pub event_signature: Option<String>,
    pub event_filter: Option<String>,
}

impl From<&EventQuery> for EventQuery_ {
    fn from(query: &EventQuery) -> Self {
        EventQuery_ {
            address: Address::from(query.address.as_ref()),
            from_block: query.from_block.map(Block::from),
            to_block: query.to_block.map(Block::from),
            event_signature: query.event_signature.to_owned(),
            event_filter: query.event_filter.to_owned(),
        }
    }
}

impl From<EventQuery_> for EventQuery {
    fn from(query: EventQuery_) -> Self {
        EventQuery {
            address: query.address.as_ref().to_string(),
            from_block: query.from_block.map(|block| block.value()),
            to_block: query.to_block.map(|block| block.value()),
            event_signature: query.event_signature,
            event_filter: query.event_filter,
        }
    }
}

#[derive(uniffi::Record)]
pub struct EventLog {
    pub topics: Vec<Vec<u8>>,
    pub data: Vec<u8>,
    pub block: u64,
}

impl From<EventLog_> for EventLog {
    fn from(log: EventLog_) -> Self {
        EventLog {
            topics: log
                .topics
                .into_iter()
                .map(|topic| topic.0.to_vec())
                .collect(),
            data: log.data,
            block: log.block.value(),
        }
    }
}

impl Into<EventLog_> for EventLog {
    fn into(self) -> EventLog_ {
        EventLog_::new(self.topics, self.data, self.block)
    }
}
