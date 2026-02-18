// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{
    types::{transaction::Block, ContractEvent},
    Address, LedgerClient, VdrError, VdrResult,
};
use ethabi::{Hash, RawLog};
use log::warn;
use log_derive::{logfn, logfn_inputs};
use serde_derive::{Deserialize, Serialize};
use std::fmt::Debug;

/// Definition of query object to query logged events from the ledger
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventQuery {
    pub address: Address,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_block: Option<Block>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_block: Option<Block>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_filter: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct EventQueryBuilder {
    contract: String,
    from_block: Option<Block>,
    to_block: Option<Block>,
    event_signature: Option<String>,
    event_filter: Option<String>,
}

impl EventQueryBuilder {
    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub fn new() -> EventQueryBuilder {
        EventQueryBuilder::default()
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub fn set_contract(mut self, contract: &str) -> EventQueryBuilder {
        self.contract = contract.to_string();
        self
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub fn set_from_block(mut self, from_block: Option<Block>) -> EventQueryBuilder {
        self.from_block = from_block;
        self
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub fn set_to_block(mut self, to_block: Option<Block>) -> EventQueryBuilder {
        self.to_block = to_block;
        self
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    #[allow(unused)]
    pub fn set_event_signature(mut self, event_signature: String) -> EventQueryBuilder {
        self.event_signature = Some(event_signature);
        self
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub fn set_event_filer(mut self, event_filter: String) -> EventQueryBuilder {
        self.event_filter = Some(event_filter);
        self
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub fn build(self, client: &LedgerClient) -> VdrResult<EventQuery> {
        let contract = client.contract(&self.contract)?;
        let query = EventQuery {
            address: contract.address().to_owned(),
            from_block: self.from_block,
            to_block: self.to_block,
            event_signature: self.event_signature,
            event_filter: self.event_filter,
        };
        Ok(query)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct EventParser {
    contract: String,
    event: String,
}

impl EventParser {
    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub fn new() -> EventParser {
        EventParser::default()
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub fn set_contract(mut self, contract: &str) -> EventParser {
        self.contract = contract.to_string();
        self
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub fn set_event(mut self, event: &str) -> EventParser {
        self.event = event.to_string();
        self
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub fn parse<T: TryFrom<ContractEvent, Error = VdrError> + Debug>(
        self,
        client: &LedgerClient,
        log: &EventLog,
    ) -> VdrResult<T> {
        if log.data.is_empty() {
            let vdr_error =
                VdrError::ContractInvalidResponseData("Unable to parse event log".to_string());

            warn!("Error: {:?} during event log parsing", vdr_error);

            return Err(vdr_error);
        }

        let contract = client.contract(&self.contract)?;
        let event = contract.event(&self.event)?;
        let raw_log = RawLog {
            topics: log.topics.clone(),
            data: log.data.clone(),
        };
        let parsed_log = event.parse_log(raw_log).map(ContractEvent::from)?;

        if parsed_log.is_empty() {
            let vdr_error =
                VdrError::ContractInvalidResponseData("Unable to parse response".to_string());

            warn!("Error: {:?} during transaction output parse", vdr_error);

            return Err(vdr_error);
        }

        T::try_from(parsed_log)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct EventLog {
    pub topics: Vec<Hash>,
    pub data: Vec<u8>,
    pub block: Block,
}

impl EventLog {
    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub fn new(topics: Vec<Vec<u8>>, data: Vec<u8>, block: u64) -> EventLog {
        EventLog {
            topics: topics
                .iter()
                .map(|topic| ethereum_types::H256::from_slice(topic))
                .collect(),
            data,
            block: Block::from(block),
        }
    }
}
