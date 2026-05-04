// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0
use std::sync::Arc;

use crate::{
    ffi::{
        error::VdrResult,
        event_query::{EventLog, EventQuery},
        transaction::Transaction,
        types::{ContractConfig, PingStatus, QuorumConfig},
    },
    VdrError,
};
use indy_besu_vdr::{ContractConfig as ContractConfig_, LedgerClient as LedgerClient_};

#[derive(uniffi::Object)]
pub struct LedgerClient {
    pub client: Arc<LedgerClient_>,
}

#[uniffi::export(async_runtime = "tokio")]
impl LedgerClient {
    #[uniffi::constructor]
    pub fn new(
        chain_id: u64,
        node_address: String,
        contract_configs: Vec<ContractConfig>,
        network: Option<String>,
        quorum_config: Option<QuorumConfig>,
    ) -> VdrResult<LedgerClient> {
        let contract_configs: Vec<ContractConfig_> = contract_configs
            .into_iter()
            .map(ContractConfig::into)
            .collect();
        let quorum_config = quorum_config.map(QuorumConfig::into);
        let client = Arc::new(LedgerClient_::new(
            chain_id,
            &node_address,
            &contract_configs,
            network.as_deref(),
            quorum_config.as_ref(),
        )?);
        Ok(LedgerClient { client })
    }

    pub async fn ping(&self) -> VdrResult<PingStatus> {
        let ping = self.client.ping().await?;
        Ok(ping.into())
    }

    pub async fn submit_transaction(&self, transaction: &Transaction) -> VdrResult<Vec<u8>> {
        self.client
            .submit_transaction(&transaction.into())
            .await
            .map_err(VdrError::from)
    }

    pub async fn query_events(&self, query: &EventQuery) -> VdrResult<Vec<EventLog>> {
        Ok(self
            .client
            .query_events(&query.into())
            .await?
            .into_iter()
            .map(EventLog::from)
            .collect())
    }

    pub async fn get_receipt(&self, hash: Vec<u8>) -> VdrResult<String> {
        self.client.get_receipt(&hash).await.map_err(VdrError::from)
    }

    pub fn network(&self) -> VdrResult<String> {
        Ok(self
            .client
            .network()
            .cloned()
            .unwrap_or_else(|| "".to_string()))
    }
}
