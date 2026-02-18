// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use futures::{
    channel::{
        mpsc,
        mpsc::{Receiver, Sender},
    },
    StreamExt,
};
use log::trace;
use log_derive::{logfn, logfn_inputs};
use serde_derive::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Formatter},
    sync::Arc,
    time::Duration,
};

use crate::{
    client::implementation::web3::client::Web3Client, Client, Transaction, TransactionType,
    VdrError, VdrResult,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuorumConfig {
    pub nodes: Vec<String>,
    pub request_retries: Option<u8>,
    pub request_timeout: Option<u64>,
    pub retry_interval: Option<u64>,
}

const DEFAULT_REQUEST_RETRIES: u8 = 4;
const DEFAULT_REQUEST_TIMEOUT: u64 = 2000;
const DEFAULT_RETRY_INTERVAL: u64 = 500;

pub struct QuorumHandler {
    clients: Vec<Arc<Box<dyn Client>>>,
    request_retries: u8,
    request_timeout: Duration,
    retry_interval: Duration,
}

impl QuorumHandler {
    #[logfn(Info)]
    #[logfn_inputs(Debug)]
    pub fn new(config: QuorumConfig) -> VdrResult<QuorumHandler> {
        let clients = config
            .nodes
            .iter()
            .map(|node_address| {
                let client: Box<dyn Client> = Box::new(Web3Client::new(node_address)?);
                Ok(Arc::new(client))
            })
            .collect::<Result<Vec<_>, VdrError>>()?;

        let handler = QuorumHandler {
            clients,
            request_retries: config.request_retries.unwrap_or(DEFAULT_REQUEST_RETRIES),
            request_timeout: Duration::from_millis(
                config.request_timeout.unwrap_or(DEFAULT_REQUEST_TIMEOUT),
            ),
            retry_interval: Duration::from_millis(
                config.retry_interval.unwrap_or(DEFAULT_RETRY_INTERVAL),
            ),
        };
        Ok(handler)
    }

    #[logfn(Info)]
    #[logfn_inputs(Debug)]
    #[allow(clippy::too_many_arguments)]
    async fn send_transaction_with_retries(
        mut sender: Sender<Vec<u8>>,
        client: Arc<Box<dyn Client>>,
        type_: TransactionType,
        to: String,
        data: Vec<u8>,
        request_retries: u8,
        request_timeout: Duration,
        retry_interval: Duration,
    ) {
        for _ in 1..request_retries {
            match type_ {
                TransactionType::Write => {
                    let future = client.get_transaction(&data);
                    match async_std::future::timeout(request_timeout, future).await {
                        Ok(Ok(Some(transaction))) if transaction.hash.is_some() => {
                            let hash = transaction.hash.unwrap();
                            if sender.try_send(hash).is_err() {
                                trace!("Receiver is closed for sender: {:?}", sender);
                            }
                            break;
                        }
                        _ => {
                            trace!(
                                "eth_getTransaction not succeed for transaction_hash: {:?}. retry",
                                data
                            );
                            async_std::task::sleep(retry_interval).await;
                        }
                    }
                }
                TransactionType::Read => {
                    let future = client.call_transaction(&to, &data);
                    match async_std::future::timeout(request_timeout, future).await {
                        Ok(Ok(transaction)) => {
                            if sender.try_send(transaction).is_err() {
                                trace!("Receiver is closed for sender: {:?}", sender);
                            }
                            break;
                        }
                        _ => {
                            trace!(
                                "call_transaction not succeed for transaction_hash: {:?}. retry",
                                data
                            );
                            async_std::task::sleep(retry_interval).await;
                        }
                    }
                }
            };
        }
    }

    #[logfn(Info)]
    #[logfn_inputs(Debug)]
    async fn wait_for_quorum(
        &self,
        mut receiver: Receiver<Vec<u8>>,
        expected_result: &[u8],
    ) -> bool {
        let approvals_needed = self.clients.len() / 3 + 1;
        let mut approvals_counter = 0;
        let mut quorum_reached = false;

        while let Some(result) = receiver.next().await {
            if result == expected_result {
                approvals_counter += 1;

                quorum_reached = approvals_counter >= approvals_needed;
                if quorum_reached {
                    break;
                }
            }
        }

        quorum_reached
    }

    #[logfn(Info)]
    #[logfn_inputs(Debug)]
    pub async fn check(
        &self,
        transaction: &Transaction,
        expected_result: &[u8],
    ) -> VdrResult<bool> {
        let clients_count = self.clients.len();
        let (sender, receiver) = mpsc::channel::<Vec<u8>>(clients_count);

        for client in self.clients.iter() {
            let type_ = transaction.type_.clone();
            let to = transaction.to.clone();
            let transaction_data = match transaction.type_ {
                TransactionType::Write => expected_result.to_vec(),
                TransactionType::Read => transaction.data.to_vec(),
            };

            #[cfg(feature = "wasm")]
            {
                let sender = sender.clone();
                let client = client.clone();
                let request_retries = self.request_retries;
                let request_timeout = self.request_timeout;
                let retry_interval = self.retry_interval;
                async_std::task::block_on(QuorumHandler::send_transaction_with_retries(
                    sender,
                    client,
                    type_,
                    to.to_string(),
                    transaction_data,
                    request_retries,
                    request_timeout,
                    retry_interval,
                ));
            }

            #[cfg(not(feature = "wasm"))]
            {
                async_std::task::spawn(QuorumHandler::send_transaction_with_retries(
                    sender.clone(),
                    client.clone(),
                    type_,
                    to.to_string(),
                    transaction_data,
                    self.request_retries,
                    self.request_timeout,
                    self.retry_interval,
                ));
            }
        }

        drop(sender);

        let quorum_reached = self.wait_for_quorum(receiver, expected_result).await;
        if quorum_reached {
            Ok(quorum_reached)
        } else {
            Err(VdrError::QuorumNotReached(format!(
                "Quorum not reached for transaction: {:?}",
                transaction
            )))
        }
    }
}

impl Debug for QuorumHandler {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"QuorumHandler {{
            request_retries: {},
            request_timeout: {:?},
            retry_interval: {:?}
        }}"#,
            self.request_retries, self.request_timeout, self.retry_interval
        )
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::client::{client::test::CLIENT_NODE_ADDRESSES, MockClient};
    use mockall::predicate::eq;
    use once_cell::sync::Lazy;
    use std::{thread, time};

    impl Default for QuorumConfig {
        fn default() -> Self {
            QuorumConfig {
                nodes: CLIENT_NODE_ADDRESSES
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect(),
                request_retries: Some(DEFAULT_REQUEST_RETRIES),
                request_timeout: Some(DEFAULT_REQUEST_TIMEOUT),
                retry_interval: Some(DEFAULT_RETRY_INTERVAL),
            }
        }
    }

    impl Default for QuorumHandler {
        fn default() -> Self {
            QuorumHandler {
                clients: vec![],
                request_retries: DEFAULT_REQUEST_RETRIES,
                request_timeout: Duration::from_millis(DEFAULT_REQUEST_TIMEOUT),
                retry_interval: Duration::from_millis(DEFAULT_RETRY_INTERVAL),
            }
        }
    }

    const TIMEOUT_TIME: u64 = 1000;
    const RETRIES: u8 = 5;

    #[cfg(test)]
    mod write_quorum_test {
        use super::*;

        static TXN_HASH: Lazy<Vec<u8>> = Lazy::new(|| vec![1; 32]);

        static WRITE_TRANSACTION: Lazy<Transaction> = Lazy::new(|| Transaction {
            type_: TransactionType::Write,
            hash: Some(TXN_HASH.clone()),
            ..Transaction::default()
        });

        fn mock_client(
            txn_hash: &[u8],
            expected_output: Option<Transaction>,
        ) -> Arc<Box<dyn Client>> {
            let mut mock_client = MockClient::new();
            mock_client
                .expect_get_transaction()
                .with(eq(txn_hash.to_vec()))
                .returning(move |_| Ok(expected_output.clone()));

            Arc::new(Box::new(mock_client))
        }

        fn mock_client_sleep_before_return(
            txn_hash: &[u8],
            expected_output: Option<Transaction>,
            sleep_time_sec: u64,
        ) -> Arc<Box<dyn Client>> {
            let mut mock_client = MockClient::new();
            mock_client
                .expect_get_transaction()
                .with(eq(txn_hash.to_vec()))
                .returning(move |_| {
                    thread::sleep(time::Duration::from_millis(sleep_time_sec.into()));
                    Ok(expected_output.clone())
                });

            Arc::new(Box::new(mock_client))
        }

        fn mock_client_retries(
            txn_hash: &[u8],
            expected_output: Option<Transaction>,
            retries_num: u8,
        ) -> Arc<Box<dyn Client>> {
            let mut mock_client = MockClient::new();
            mock_client
                .expect_get_transaction()
                .with(eq(txn_hash.to_vec()))
                .times(retries_num as usize - 1)
                .returning(move |_| Ok(None));

            mock_client
                .expect_get_transaction()
                .with(eq(txn_hash.to_vec()))
                .returning(move |_| Ok(expected_output.clone()));

            Arc::new(Box::new(mock_client))
        }

        #[async_std::test]
        async fn test_quorum_check_positive_case() {
            let client1 = mock_client(&TXN_HASH, Some(WRITE_TRANSACTION.clone()));
            let client2 = mock_client(&TXN_HASH, Some(WRITE_TRANSACTION.clone()));
            let quorum = QuorumHandler {
                clients: vec![client1, client2],
                ..QuorumHandler::default()
            };
            assert!(quorum.check(&WRITE_TRANSACTION, &TXN_HASH).await.unwrap());
        }

        #[async_std::test]
        async fn test_quorum_check_failed_with_timeout() {
            let client1 = mock_client(&TXN_HASH, None);
            let client2 = mock_client_sleep_before_return(&TXN_HASH, None, TIMEOUT_TIME + 3000);
            let quorum = QuorumHandler {
                clients: vec![client1, client2],
                request_timeout: Duration::from_millis(TIMEOUT_TIME),
                ..QuorumHandler::default()
            };
            let _err = quorum
                .check(&WRITE_TRANSACTION, &TXN_HASH)
                .await
                .unwrap_err();
        }

        #[async_std::test]
        async fn test_quorum_check_not_reached() {
            let client1: Arc<Box<dyn Client>> =
                mock_client(&TXN_HASH, Some(WRITE_TRANSACTION.clone()));
            let client2 = mock_client(
                &TXN_HASH,
                Some(Transaction {
                    hash: Some([2; 32].to_vec()),
                    ..Default::default()
                }),
            );
            let client3 = mock_client(
                &TXN_HASH,
                Some(Transaction {
                    hash: Some([3; 32].to_vec()),
                    ..Default::default()
                }),
            );
            let quorum = QuorumHandler {
                clients: vec![client1, client2, client3],
                ..QuorumHandler::default()
            };

            let _err = quorum
                .check(&WRITE_TRANSACTION, &TXN_HASH)
                .await
                .unwrap_err();
        }

        #[async_std::test]
        async fn test_quorum_check_got_transaction_after_retries() {
            let client1 = mock_client_retries(&TXN_HASH, Some(WRITE_TRANSACTION.clone()), RETRIES);
            let client2 = mock_client(&TXN_HASH, Some(WRITE_TRANSACTION.clone()));
            let quorum = QuorumHandler {
                clients: vec![client1, client2],
                request_retries: RETRIES,
                ..QuorumHandler::default()
            };
            assert!(quorum.check(&WRITE_TRANSACTION, &TXN_HASH).await.unwrap());
        }
    }

    #[cfg(test)]
    mod read_quorum_test {
        use super::*;

        static READ_TRANSACTION: Lazy<Transaction> = Lazy::new(|| Transaction {
            type_: TransactionType::Read,
            ..Transaction::default()
        });

        static RESPONSE: Lazy<Vec<u8>> = Lazy::new(|| vec![1, 1, 1, 1]);

        fn mock_client(
            transaction: Transaction,
            expected_output: VdrResult<Vec<u8>>,
        ) -> Arc<Box<dyn Client>> {
            let mut mock_client = MockClient::new();
            mock_client
                .expect_call_transaction()
                .with(
                    eq(transaction.to.to_string()),
                    eq(transaction.data.to_vec()),
                )
                .returning(move |_, _| expected_output.clone());

            Arc::new(Box::new(mock_client))
        }

        fn mock_client_sleep_before_return(
            transaction: Transaction,
            expected_output: VdrResult<Vec<u8>>,
            sleep_time_sec: u64,
        ) -> Arc<Box<dyn Client>> {
            let mut mock_client = MockClient::new();
            mock_client
                .expect_call_transaction()
                .with(
                    eq(transaction.to.to_string()),
                    eq(transaction.data.to_vec()),
                )
                .returning(move |_, _| {
                    thread::sleep(time::Duration::from_millis(sleep_time_sec.into()));
                    expected_output.clone()
                });

            Arc::new(Box::new(mock_client))
        }

        fn mock_client_retries(
            transaction: Transaction,
            expected_output: VdrResult<Vec<u8>>,
            retries_num: u8,
        ) -> Arc<Box<dyn Client>> {
            let mut mock_client = MockClient::new();

            mock_client
                .expect_call_transaction()
                .with(
                    eq(transaction.to.to_string()),
                    eq(transaction.data.to_vec()),
                )
                .times(retries_num as usize - 1)
                .returning(move |_, _| Err(VdrError::ContractInvalidResponseData("".to_string())));

            mock_client
                .expect_call_transaction()
                .with(
                    eq(transaction.to.to_string()),
                    eq(transaction.data.to_vec()),
                )
                .returning(move |_, _| expected_output.clone());

            Arc::new(Box::new(mock_client))
        }

        #[async_std::test]
        async fn test_quorum_check_positive_case() {
            let client1 = mock_client(READ_TRANSACTION.clone(), Ok(RESPONSE.clone()));
            let client2 = mock_client(READ_TRANSACTION.clone(), Ok(RESPONSE.clone()));
            let quorum = QuorumHandler {
                clients: vec![client1, client2],
                ..QuorumHandler::default()
            };
            assert!(quorum.check(&READ_TRANSACTION, &RESPONSE).await.unwrap());
        }

        #[async_std::test]
        async fn test_quorum_check_failed_with_timeout() {
            let err = Err(VdrError::ClientTransactionReverted(
                "Transaction reverted".to_string(),
            ));
            let client1 = mock_client(READ_TRANSACTION.clone(), err.clone());
            let client2 = mock_client_sleep_before_return(
                READ_TRANSACTION.clone(),
                err.clone(),
                TIMEOUT_TIME + 3000,
            );
            let quorum = QuorumHandler {
                clients: vec![client1, client2],
                request_timeout: Duration::from_millis(TIMEOUT_TIME),
                ..QuorumHandler::default()
            };

            let _err = quorum
                .check(&READ_TRANSACTION, &RESPONSE)
                .await
                .unwrap_err();
        }

        #[async_std::test]
        async fn test_quorum_check_not_reached() {
            let client1 = mock_client(READ_TRANSACTION.clone(), Ok(RESPONSE.clone()));
            let client2 = mock_client(READ_TRANSACTION.clone(), Ok(vec![1, 1, 1, 2]));
            let client3 = mock_client(READ_TRANSACTION.clone(), Ok(vec![1, 1, 1, 3]));
            let quorum = QuorumHandler {
                clients: vec![client1, client2, client3],
                ..QuorumHandler::default()
            };
            let _err = quorum
                .check(&READ_TRANSACTION, &RESPONSE)
                .await
                .unwrap_err();
        }

        #[async_std::test]
        async fn test_quorum_check_got_transaction_after_retries() {
            let client1 =
                mock_client_retries(READ_TRANSACTION.clone(), Ok(RESPONSE.clone()), RETRIES);
            let client2 = mock_client(READ_TRANSACTION.clone(), Ok(RESPONSE.clone()));
            let quorum = QuorumHandler {
                clients: vec![client1, client2],
                request_retries: RETRIES,
                ..QuorumHandler::default()
            };
            assert!(quorum.check(&READ_TRANSACTION, &RESPONSE).await.unwrap());
        }
    }
}
