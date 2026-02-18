// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{
    client::Client,
    error::{VdrError, VdrResult},
    types::EventQuery,
    Address, Block, BlockDetails, Transaction,
};

use async_std::task;
use async_trait::async_trait;
use ethereum_types::{H160, U64};
use log::{trace, warn};
use log_derive::{logfn, logfn_inputs};
use serde_json::json;
use std::{
    fmt::{Debug, Formatter},
    str::FromStr,
    time::Duration,
};

#[cfg(not(feature = "wasm"))]
use web3::{
    api::Eth,
    transports::Http,
    types::{
        Address as EthAddress, BlockId, BlockNumber, Bytes, CallRequest, FilterBuilder,
        TransactionId, H256,
    },
    Web3,
};

use crate::types::EventLog;
#[cfg(feature = "wasm")]
use web3_wasm::{
    api::Eth,
    transports::Http,
    types::{
        Address as EthAddress, BlockId, BlockNumber, Bytes, CallRequest, FilterBuilder,
        TransactionId, H256,
    },
    Web3,
};

pub struct Web3Client {
    client: Web3<Http>,
}

const POLL_INTERVAL: u64 = 200;
const NUMBER_TX_CONFIRMATIONS: usize = 1; // FIXME: what number of confirmation events should we wait? 2n+1?

impl Web3Client {
    #[logfn(Info)]
    #[logfn_inputs(Debug)]
    pub fn new(node_address: &str) -> VdrResult<Web3Client> {
        let transport = Http::new(node_address).map_err(|_| VdrError::ClientNodeUnreachable)?;
        let web3 = Web3::new(transport);
        let web3_client = Web3Client { client: web3 };
        Ok(web3_client)
    }

    pub(crate) fn eth(&self) -> Eth<Http> {
        self.client.eth()
    }
}

#[cfg_attr(not(feature = "wasm"), async_trait)]
#[cfg_attr(feature = "wasm", async_trait(? Send))]
impl Client for Web3Client {
    async fn get_transaction_count(&self, address: &Address) -> VdrResult<u64> {
        trace!("Web3Client::get_transaction_count(address: {:?})", address);

        let account_address = EthAddress::from_str(address.as_ref()).map_err(|_| {
            VdrError::ClientInvalidTransaction(format!(
                "Invalid transaction sender address {:?}",
                address
            ))
        })?;

        let count = self
            .client
            .eth()
            .transaction_count(account_address, None)
            .await?
            .as_u64();

        trace!("Web3Client::get_transaction_count() -> {:?}", count);
        Ok(count)
    }

    async fn submit_transaction(&self, transaction: &[u8]) -> VdrResult<Vec<u8>> {
        trace!(
            "Web3Client::submit_transaction(transaction: {:?})",
            transaction
        );

        let receipt = self
            .client
            .send_raw_transaction_with_confirmation(
                Bytes::from(transaction),
                Duration::from_millis(POLL_INTERVAL),
                NUMBER_TX_CONFIRMATIONS,
            )
            .await?;

        // let tx_hash = self.client.eth().send_raw_transaction(Bytes::from(transaction)).await?;
        // let receipt = loop {
        //     if let Ok(Some(receipt)) = self.client.eth().transaction_receipt(tx_hash).await {
        //         break receipt;
        //     }
        //     task::sleep(Duration::from_secs(1)).await;
        // };

        if receipt.is_txn_reverted() {
            if let Some(revert_reason) = receipt.revert_reason {
                return Err(VdrError::ClientTransactionReverted(revert_reason));
            }

            return Err(VdrError::ClientTransactionReverted("".to_string()));
        }

        trace!("Web3Client::submit_transaction() -> {:?}", receipt);

        let transaction_hash = receipt.transaction_hash.0.to_vec();

        Ok(transaction_hash)
    }

    async fn call_transaction(&self, to: &str, transaction: &[u8]) -> VdrResult<Vec<u8>> {
        trace!(
            "Web3Client::call_transaction(to: {:?}, transaction: {:?})",
            to,
            transaction
        );

        let address = EthAddress::from_str(to).map_err(|_| {
            let vdr_error = VdrError::ClientInvalidTransaction(format!(
                "Invalid transaction target address {:?}",
                to
            ));

            warn!(
                "Error: {} during calling transaction: {:?}",
                vdr_error, transaction
            );

            vdr_error
        })?;
        let request = CallRequest::builder()
            .to(address)
            .data(Bytes(transaction.to_vec()))
            .build();
        let response = self.client.eth().call(request, None).await?.0.to_vec();

        trace!("Web3Client::call_transaction() -> {:?}", response);
        Ok(response)
    }

    async fn query_events(&self, query: &EventQuery) -> VdrResult<Vec<EventLog>> {
        trace!("Web3Client::query_events(query: {:?})", query);

        let address = H160::from_str(query.address.as_ref()).map_err(|_| {
            VdrError::ClientInvalidTransaction(format!(
                "Invalid transaction target address {:?}",
                query.address
            ))
        })?;

        let from_block = match query.from_block {
            Some(ref block) => BlockNumber::Number(U64::from(block.value())),
            None => BlockNumber::Earliest,
        };

        let to_block = match query.to_block {
            Some(ref block) => BlockNumber::Number(U64::from(block.value())),
            None => BlockNumber::Latest,
        };

        let event_signature = match query.event_signature {
            Some(ref event_signature) => Some(H256::from_str(event_signature).map_err(|_| {
                VdrError::ClientInvalidTransaction(format!(
                    "Unable to convert event signature into H256 {:?}",
                    event_signature
                ))
            })?),
            None => None,
        };

        let event_filter = match query.event_filter {
            Some(ref event_filter) => Some(H256::from_str(event_filter).map_err(|_| {
                VdrError::ClientInvalidTransaction(format!(
                    "Unable to convert event filter into H256 {:?}",
                    event_filter
                ))
            })?),
            None => None,
        };

        let filter = FilterBuilder::default()
            .address(vec![address])
            .topics(
                event_signature.map(|event_signature| vec![event_signature]),
                event_filter.map(|event_filter| vec![event_filter]),
                None,
                None,
            )
            .from_block(from_block)
            .to_block(to_block)
            .build();

        let logs = self
            .client
            .eth()
            .logs(filter)
            .await
            .map_err(|e| VdrError::GetTransactionError(e.to_string()))?;

        let events: Vec<EventLog> = logs
            .into_iter()
            .map(|log| EventLog {
                topics: log.topics,
                data: log.data.0,
                block: Block::from(log.block_number.unwrap_or_default().as_u64()),
            })
            .collect();

        trace!("Web3Client::query_events() -> {:?}", events);
        Ok(events)
    }

    async fn get_receipt(&self, hash: &[u8]) -> VdrResult<String> {
        trace!("Web3Client::get_receipt(hash: {:?})", hash);

        if hash.len() != 32 {
            let vdr_error =
                VdrError::CommonInvalidData("Transaction hash length != 32 bytes".to_string());

            warn!("Error: {} getting receipt", vdr_error,);

            return Err(vdr_error);
        }

        let receipt = self
            .client
            .eth()
            .transaction_receipt(H256::from_slice(hash))
            .await?
            .ok_or_else(|| {
                let vdr_error =
                    VdrError::ClientInvalidResponse("Missing transaction receipt".to_string());

                warn!("Error: {} getting receipt", vdr_error,);

                vdr_error
            })
            .map(|receipt| json!(receipt).to_string())?;

        trace!("Web3Client::get_receipt() -> {:?}", receipt);
        Ok(receipt)
    }

    async fn get_block(&self, block: Option<u64>) -> VdrResult<BlockDetails> {
        trace!("Web3Client::ping()");

        let block_id = match block {
            Some(block) => BlockId::Number(BlockNumber::Number(U64::from(block))),
            None => BlockId::Number(BlockNumber::Latest),
        };

        match self.client.eth().block(block_id).await {
            Ok(Some(block)) => Ok(BlockDetails {
                number: block.number.unwrap().as_u64(),
                timestamp: block.timestamp.as_u64(),
            }),
            _ => Err(VdrError::ClientInvalidState(
                "Could not get current network block".to_string(),
            )),
        }
    }

    async fn get_transaction(&self, transaction_hash: &[u8]) -> VdrResult<Option<Transaction>> {
        trace!(
            "Web3Client::get_transaction(transaction_hash: {:?})",
            transaction_hash
        );

        let transaction_id = TransactionId::Hash(H256::from_slice(transaction_hash));
        let transaction = self
            .client
            .eth()
            .transaction(transaction_id)
            .await
            .map_err(|_| {
                VdrError::GetTransactionError("Could not get transaction by hash".to_string())
            })?;

        let transaction = transaction.map(|transaction| Transaction {
            type_: Default::default(),
            from: transaction
                .from
                .map(|from| Address::from(from.to_string().as_str())),
            to: transaction
                .to
                .map(|from| Address::from(from.to_string().as_str()))
                .unwrap_or_default(),
            nonce: Some(transaction.nonce.as_u64()),
            chain_id: 0,
            data: transaction.input.0.to_vec(),
            signature: Default::default(),
            hash: Some(transaction.hash.as_bytes().to_vec()),
        });

        trace!("Web3Client::get_transaction() -> {:?}", transaction);
        Ok(transaction)
    }
}

impl Debug for Web3Client {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, r#"Web3Client {{ }}"#)
    }
}
