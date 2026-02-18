// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

pub mod client;
pub mod constants;
pub mod implementation;
pub mod quorum;

use crate::{error::VdrResult, types::Address, BlockDetails, Transaction};
use async_trait::async_trait;
use ethabi::{AbiError, Event, Function};
use std::fmt::Debug;

pub use client::LedgerClient;
pub use constants::*;
pub use quorum::{QuorumConfig, QuorumHandler};

use crate::types::{EventLog, EventQuery};
#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[cfg_attr(not(feature = "wasm"), async_trait)]
#[cfg_attr(feature = "wasm", async_trait(?Send))]
pub trait Client: Sync + Send + Debug {
    /// Retrieve count of transaction for the given account
    ///
    /// # Params
    /// - `address` [Address] address of an account to get number of written transactions
    ///
    /// # Returns
    /// number of transactions
    async fn get_transaction_count(&self, address: &Address) -> VdrResult<u64>;

    /// Submit transaction to the ledger
    ///
    /// # Params
    /// - `transaction` [Transaction] transaction to submit
    ///
    /// # Returns
    /// hash of a block in which transaction included
    async fn submit_transaction(&self, transaction: &[u8]) -> VdrResult<Vec<u8>>;

    /// Submit read transaction to the ledger
    ///
    /// # Params
    /// - `transaction` [Transaction] prepared transaction to submit
    ///
    /// # Returns
    /// result data of transaction execution
    async fn call_transaction(&self, to: &str, transaction: &[u8]) -> VdrResult<Vec<u8>>;

    /// Send a prepared query for retrieving log events on the ledger
    ///
    /// #Params
    /// - `query` [EventQuery] query to send
    ///
    /// #Returns
    ///   logs - list of received events
    async fn query_events(&self, query: &EventQuery) -> VdrResult<Vec<EventLog>>;

    /// Get the receipt for the given block hash
    ///
    /// # Params
    /// - `hash` hash of a block to get the receipt
    ///
    /// # Returns
    /// receipt as JSON string for the requested block
    async fn get_receipt(&self, hash: &[u8]) -> VdrResult<String>;

    /// Get details for the given block
    ///
    /// # Returns
    ///  Block details
    async fn get_block(&self, block: Option<u64>) -> VdrResult<BlockDetails>;

    /// Get the transaction for the given transaction hash
    ///
    /// # Params
    /// - `hash` hash of a transaction to get
    ///
    /// # Returns
    /// transaction for the requested hash
    async fn get_transaction(&self, hash: &[u8]) -> VdrResult<Option<Transaction>>;
}

pub trait Contract: Sync + Send + Debug {
    /// Get the address of deployed contract
    ///
    /// # Returns
    /// address of the deployed contract. Should be used to execute contract methods
    fn address(&self) -> &Address;

    /// Get the contract function
    ///
    /// # Params
    /// - `name` name of a contract method
    ///
    /// # Returns
    /// Contract function
    fn function(&self, name: &str) -> VdrResult<&Function>;

    /// Get the contract event
    ///
    /// # Params
    /// - `name` name of a contract event
    ///
    /// # Returns
    /// Contract event
    fn event(&self, name: &str) -> VdrResult<&Event>;

    /// Get the contract errors
    ///
    /// # Returns
    /// Contract errors
    fn errors(&self) -> Vec<&AbiError>;
}
