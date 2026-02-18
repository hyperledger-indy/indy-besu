// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
};

use ethabi::{AbiError, Param, ParamType};
use log::warn;
use log_derive::{logfn, logfn_inputs};

use crate::{
    client::{
        implementation::web3::{client::Web3Client, contract::Web3Contract},
        Client, Contract, QuorumHandler,
    },
    error::{VdrError, VdrResult},
    types::{
        Block, ContractConfig, ContractSpec, EventLog, EventQuery, PingStatus, Transaction,
        TransactionType,
    },
    Address, BlockDetails, QuorumConfig,
};

/// Client object for interaction with the network
pub struct LedgerClient {
    chain_id: u64,
    client: Box<dyn Client>,
    contracts: HashMap<String, Box<dyn Contract>>,
    errors: HashMap<[u8; 4], AbiError>,
    network: Option<String>,
    quorum_handler: Option<QuorumHandler>,
}

impl LedgerClient {
    /// Create client interacting with ledger
    ///
    /// # Params
    ///  - `chain_id`: [u64] - chain id of network (chain ID is part of the transaction signing process to protect against transaction replay attack)
    ///  - `rpc_node`: [String] - RPC node endpoint
    ///  - `network`: [String] - Name of the network
    ///  - `contract_configs`: [ContractSpec] - specifications for contracts  deployed on the network
    ///  - `quorum_config`: Option<[QuorumConfig]> - quorum configuration. Can be None if quorum check is not needed
    ///
    /// # Returns
    ///  client: [LedgerClient] - client to use for building and sending transactions
    #[logfn(Info)]
    #[logfn_inputs(Debug)]
    pub fn new(
        chain_id: u64,
        rpc_node: &str,
        contract_configs: &[ContractConfig],
        network: Option<&str>,
        quorum_config: Option<&QuorumConfig>,
    ) -> VdrResult<LedgerClient> {
        let client = Box::new(Web3Client::new(rpc_node)?);

        let contracts = Self::init_contracts(&client, contract_configs)?;
        let errors = Self::build_error_map(&contracts)?;

        let quorum_handler = match quorum_config {
            Some(quorum_config) => Some(QuorumHandler::new(quorum_config.clone())?),
            None => None,
        };

        let ledger_client = LedgerClient {
            chain_id,
            client,
            contracts,
            errors,
            network: network.map(String::from),
            quorum_handler,
        };
        Ok(ledger_client)
    }

    /// Ping Ledger.
    ///
    /// # Returns
    ///  status: [PingStatus] - ping status
    #[logfn(Info)]
    #[logfn_inputs(Debug)]
    pub async fn ping(&self) -> VdrResult<PingStatus> {
        match self.client.get_block(None).await {
            Ok(block) => Ok(PingStatus::ok(block.number, block.timestamp)),
            Err(err) => Ok(PingStatus::err(err.to_string().as_str())),
        }
    }

    /// Submit prepared transaction to the ledger
    ///     Depending on the transaction type Write/Read ethereum methods will be used
    ///
    /// #Params
    ///  `transaction`: [Transaction] - transaction to submit
    ///
    /// #Returns
    ///  response: [Vec] - transaction execution result:
    ///    depending on the type it will be either result bytes or block hash
    #[logfn(Info)]
    #[logfn_inputs(Debug)]
    pub async fn submit_transaction(&self, transaction: &Transaction) -> VdrResult<Vec<u8>> {
        let result = match transaction.type_ {
            TransactionType::Read => {
                self.client
                    .call_transaction(transaction.to.as_ref(), &transaction.data)
                    .await
            }
            TransactionType::Write => self.client.submit_transaction(&transaction.encode()?).await,
        };

        let data = match result {
            Ok(data) => data,
            Err(VdrError::ClientTransactionReverted(revert_reason)) => {
                let decoded_reason = self.decode_revert_reason(&revert_reason)?;

                return Err(VdrError::ClientTransactionReverted(decoded_reason));
            }
            Err(error) => return Err(error),
        };

        if let Some(quorum_handler) = &self.quorum_handler {
            quorum_handler.check(transaction, &data).await?;
        };

        Ok(data)
    }

    /// Submit prepared events query to the ledger
    ///
    /// #Params
    ///  `query`: [EventQuery] - events query to submit
    ///
    /// #Returns
    ///  events: [Vec] - list of log events received from the ledger
    #[logfn(Info)]
    #[logfn_inputs(Debug)]
    pub async fn query_events(&self, query: &EventQuery) -> VdrResult<Vec<EventLog>> {
        let events = self.client.query_events(query).await?;
        // TODO: Check quorum for events
        Ok(events)
    }

    /// Get receipt for the given block hash
    ///
    /// # Params
    ///  `transaction`: [Transaction] - transaction to submit
    ///
    /// # Returns
    ///  receipt: [String] - receipt for the given block
    #[logfn(Info)]
    #[logfn_inputs(Debug)]
    pub async fn get_receipt(&self, hash: &[u8]) -> VdrResult<String> {
        self.client.get_receipt(hash).await
    }

    /// Get a number of transactions sent by the given account address
    ///
    /// # Params
    ///  `address`: [Address] - target account address
    ///
    /// # Returns
    ///  count: [u64] - number of transaction sent by the given account
    #[logfn(Info)]
    #[logfn_inputs(Debug)]
    pub(crate) async fn get_transaction_count(&self, address: &Address) -> VdrResult<u64> {
        self.client.get_transaction_count(address).await
    }

    pub(crate) fn contract(&self, name: &str) -> VdrResult<&dyn Contract> {
        self.contracts
            .get(name)
            .map(|contract| contract.as_ref())
            .ok_or_else(|| {
                let vdr_error = VdrError::ContractInvalidName(name.to_string());

                warn!("Error during getting contract: {:?}", vdr_error);

                vdr_error
            })
    }

    pub(crate) fn chain_id(&self) -> u64 {
        self.chain_id
    }

    pub(crate) fn network(&self) -> Option<&String> {
        self.network.as_ref()
    }

    #[logfn(Info)]
    #[logfn_inputs(Debug)]
    fn init_contracts(
        client: &Web3Client,
        contract_configs: &[ContractConfig],
    ) -> VdrResult<HashMap<String, Box<dyn Contract>>> {
        let mut contracts: HashMap<String, Box<dyn Contract>> = HashMap::new();
        for contract_config in contract_configs {
            let spec = match (
                contract_config.spec_path.as_ref(),
                contract_config.spec.as_ref(),
            ) {
                (Some(spec_path), None) => ContractSpec::from_file(spec_path)?,
                (None, Some(spec)) => spec.clone(),
                (Some(_), Some(_)) => {
                    return Err(VdrError::ContractInvalidSpec(
                        "Either `spec_path` or `spec` must be provided".to_string(),
                    ));
                }
                (None, None) => {
                    return Err(VdrError::ContractInvalidSpec(
                        "Either `spec_path` or `spec` must be provided".to_string(),
                    ));
                }
            };

            let contract = Web3Contract::new(client, &contract_config.address, &spec)?;
            contracts.insert(spec.name.clone(), Box::new(contract));
        }

        Ok(contracts)
    }

    fn build_error_map(
        contracts: &HashMap<String, Box<dyn Contract>>,
    ) -> VdrResult<HashMap<[u8; 4], AbiError>> {
        let regular_error = AbiError {
            name: "Error".to_string(),
            inputs: vec![Param {
                name: "message".to_string(),
                kind: ParamType::String,
                internal_type: None,
            }],
        };

        let panic_error = AbiError {
            name: "Panic".to_string(),
            inputs: vec![Param {
                name: "code".to_string(),
                kind: ParamType::Uint(256),
                internal_type: None,
            }],
        };

        contracts
            .values()
            .map(|contract| contract.errors())
            .flatten()
            .chain([regular_error, panic_error].iter())
            .map(|error| {
                let short_signature: [u8; 4] =
                    error.signature().as_bytes()[0..4].try_into().map_err(|_| {
                        VdrError::ClientUnexpectedError(
                            "Cannot convert a slice into an array of 4 bytes".to_string(),
                        )
                    })?;

                Ok((short_signature, error.clone()))
            })
            .collect()
    }

    fn decode_revert_reason(&self, revert_reason: &str) -> VdrResult<String> {
        let error_data = hex::decode(revert_reason.trim_start_matches("0x")).map_err(|_| {
            VdrError::ContractInvalidResponseData(
                format!(
                    "Unable to parse the revert reason '{}': Incorrect hex string",
                    revert_reason
                )
                .to_string(),
            )
        })?;

        if error_data.len() < 4 {
            return Err(VdrError::ContractInvalidResponseData(
                format!(
                    "Unable to parse the revert reason '{}': Incorrect data",
                    revert_reason
                )
                .to_string(),
            ));
        }

        let signature: &[u8; 4] = error_data[0..4].try_into().map_err(|_| {
            VdrError::ClientUnexpectedError(
                "Cannot convert a slice into an array of 4 bytes".to_string(),
            )
        })?;
        let arguments: &[u8] = &error_data[4..];

        let error = self.errors.get(signature).ok_or_else( || {
            VdrError::ContractInvalidResponseData(
                format!(
                    "Unable to parse the revert reason '{}': Cannot match the error selector with registered errors",
                    revert_reason
                )
                .to_string()
            )
        })?;

        let decoded_args = error.decode(&arguments).map_err(|_| {
            VdrError::ContractInvalidResponseData(
                format!(
                    "Unable to parse the revert reason '{}': Failed to decode the arguments",
                    revert_reason
                )
                .to_string(),
            )
        })?;

        let inputs_str: Vec<String> = error
            .inputs
            .iter()
            .enumerate()
            .map(|(i, input)| format!("{}: {}", input.name, decoded_args[i].to_string()))
            .collect();

        let inputs_joined = inputs_str.join(", ");
        let decoded_str = format!("{}({})", error.name, inputs_joined);

        Ok(decoded_str)
    }

    #[logfn(Info)]
    #[logfn_inputs(Debug)]
    pub(crate) async fn get_block(&self, block: Option<&Block>) -> VdrResult<BlockDetails> {
        self.client
            .get_block(block.map(|block| block.value()))
            .await
    }
}

impl Debug for LedgerClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, r#"LedgerClient {{ chain_id: {} }}"#, self.chain_id)
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::{
        client::MockClient, types::transaction::test::read_transaction, utils::init_env_logger,
    };
    use once_cell::sync::Lazy;
    use serde::{Deserialize, Serialize};
    use std::{env, fs};

    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct TestConfig {
        pub chain_id: u64,
        pub node_address: String,
        pub contracts: TestContractsConfigs,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct TestContractsConfigs {
        pub indy_did_registry: TestContractConfig,
        pub ethereum_did_registry: TestContractConfig,
        pub cred_def_registry: TestContractConfig,
        pub schema_registry: TestContractConfig,
        pub role_control: TestContractConfig,
        pub validator_control: TestContractConfig,
        pub account_control: TestContractConfig,
        pub upgrade_control: TestContractConfig,
        pub legacy_mapping_registry: TestContractConfig,
        pub revocation_registry: TestContractConfig,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct TestContractConfig {
        pub address: Address,
        pub spec_path: String,
    }

    pub const CLIENT_NODE_ADDRESSES: [&str; 4] = [
        "http://127.0.0.1:21001",
        "http://127.0.0.1:21002",
        "http://127.0.0.1:21003",
        "http://127.0.0.1:21004",
    ];
    pub const DEFAULT_NONCE: u64 = 0;
    pub const INVALID_ADDRESS: &str = "123";
    pub const TEST_NETWORK: &str = "test";

    pub static CONFIG: Lazy<TestConfig> = Lazy::new(|| read_config());

    pub static TRUSTEE_ACCOUNT: Lazy<Address> =
        Lazy::new(|| Address::from("0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5"));

    pub static TEST_ACCOUNT: Lazy<Address> =
        Lazy::new(|| Address::from("0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5"));

    fn build_contract_path(contract_path: &str) -> String {
        let mut cur_dir = env::current_dir().unwrap();
        cur_dir.push(".."); // project root directory
        cur_dir.push(contract_path);
        fs::canonicalize(&cur_dir)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }

    fn read_config() -> TestConfig {
        let file =
            fs::File::open("../network/config.json").expect("Unable to open besu config file");
        serde_json::from_reader(file).expect("Unable to parse besu config file")
    }

    fn contracts() -> Vec<ContractConfig> {
        vec![
            ContractConfig {
                address: CONFIG.contracts.ethereum_did_registry.address.to_string(),
                spec_path: Some(build_contract_path(
                    CONFIG.contracts.ethereum_did_registry.spec_path.as_str(),
                )),
                spec: None,
            },
            ContractConfig {
                address: CONFIG.contracts.indy_did_registry.address.to_string(),
                spec_path: Some(build_contract_path(
                    CONFIG.contracts.indy_did_registry.spec_path.as_str(),
                )),
                spec: None,
            },
            ContractConfig {
                address: CONFIG.contracts.schema_registry.address.to_string(),
                spec_path: Some(build_contract_path(
                    CONFIG.contracts.schema_registry.spec_path.as_str(),
                )),
                spec: None,
            },
            ContractConfig {
                address: CONFIG.contracts.cred_def_registry.address.to_string(),
                spec_path: Some(build_contract_path(
                    CONFIG.contracts.cred_def_registry.spec_path.as_str(),
                )),
                spec: None,
            },
            ContractConfig {
                address: CONFIG.contracts.role_control.address.to_string(),
                spec_path: Some(build_contract_path(
                    CONFIG.contracts.role_control.spec_path.as_str(),
                )),
                spec: None,
            },
            ContractConfig {
                address: CONFIG.contracts.legacy_mapping_registry.address.to_string(),
                spec_path: Some(build_contract_path(
                    CONFIG.contracts.legacy_mapping_registry.spec_path.as_str(),
                )),
                spec: None,
            },
            ContractConfig {
                address: CONFIG.contracts.validator_control.address.to_string(),
                spec_path: Some(build_contract_path(
                    CONFIG.contracts.validator_control.spec_path.as_str(),
                )),
                spec: None,
            },
            ContractConfig {
                address: CONFIG.contracts.revocation_registry.address.to_string(),
                spec_path: Some(build_contract_path(
                    CONFIG.contracts.revocation_registry.spec_path.as_str(),
                )),
                spec: None,
            },
        ]
    }

    pub fn client() -> LedgerClient {
        LedgerClient::new(
            CONFIG.chain_id,
            &CONFIG.node_address,
            &contracts(),
            Some(TEST_NETWORK),
            None,
        )
        .unwrap()
    }

    pub fn mock_client() -> LedgerClient {
        init_env_logger();
        let mut ledger_client = LedgerClient::new(
            CONFIG.chain_id,
            &CONFIG.node_address,
            &contracts(),
            Some(TEST_NETWORK),
            Some(&QuorumConfig::default()),
        )
        .unwrap();

        let mut client = MockClient::new();
        client.expect_get_transaction_count().returning(|_| Ok(0));

        ledger_client.client = Box::new(client);
        ledger_client
    }

    pub fn mock_custom_client(client: Box<dyn Client>) -> LedgerClient {
        let mut ledger_client = LedgerClient::new(
            CONFIG.chain_id,
            &CONFIG.node_address,
            &contracts(),
            Some(TEST_NETWORK),
            None,
        )
        .unwrap();

        ledger_client.client = client;
        ledger_client
    }

    mod create {
        use crate::{
            transaction::test::write_transaction, validator_control::test::VALIDATOR_CONTROL_NAME,
            SignatureData,
        };
        use mockall::predicate::eq;
        use rstest::rstest;
        use serde_json::Value;

        use super::*;

        #[test]
        fn create_client_test() {
            client();
        }

        #[test]
        fn create_client_invalid_node_address() {
            let client_err = LedgerClient::new(
                CONFIG.chain_id,
                "..",
                &contracts(),
                Some(TEST_NETWORK),
                None,
            )
            .err()
            .unwrap();

            assert!(matches!(
                client_err,  | VdrError::ClientNodeUnreachable { .. }
            ));
        }

        #[rstest]
        #[case::invalid_contract_data(vec ! [ContractConfig {
                address: CONFIG.contracts.validator_control.address.to_string(),
                spec_path: None,
                spec: Some(ContractSpec {
                name: VALIDATOR_CONTROL_NAME.to_string(),
                abi: Value::String("".to_string()),
            }),
        }], VdrError::ContractInvalidInputData)]
        #[case::both_contract_path_and_spec_provided(vec ! [ContractConfig {
                address: CONFIG.contracts.validator_control.address.to_string(),
                spec_path: Some(build_contract_path(CONFIG.contracts.validator_control.spec_path.as_str())),
                spec: Some(ContractSpec {
                name: VALIDATOR_CONTROL_NAME.to_string(),
                abi: Value::Array(vec ! []),
            }),
        }], VdrError::ContractInvalidSpec("Either `spec_path` or `spec` must be provided".to_string()))]
        #[case::non_existent_spec_path(vec ! [ContractConfig {
                address: CONFIG.contracts.validator_control.address.to_string(),
                spec_path: Some(build_contract_path("")),
                spec: None,
            }], VdrError::ContractInvalidSpec("Unable to read contract spec file. Err: \"Is a directory (os error 21)\"".to_string())
        )]
        #[case::empty_contract_spec(vec ! [ContractConfig {
                address: CONFIG.contracts.validator_control.address.to_string(),
                spec_path: None,
                spec: None,
            }], VdrError::ContractInvalidSpec("Either `spec_path` or `spec` must be provided".to_string())
        )]
        fn test_create_client_errors(
            #[case] contract_config: Vec<ContractConfig>,
            #[case] expected_error: VdrError,
        ) {
            let client_err = LedgerClient::new(
                CONFIG.chain_id,
                &CONFIG.node_address,
                &contract_config,
                Some(TEST_NETWORK),
                None,
            )
            .err()
            .unwrap();

            assert_eq!(client_err, expected_error);
        }

        #[rstest]
        #[case::empty_recipient_address("", VdrError::ClientInvalidTransaction("Invalid transaction target address \"0x\"".to_string()))]
        #[case::invalid_recipient_address(INVALID_ADDRESS, VdrError::ClientInvalidTransaction("Invalid transaction target address \"0x123\"".to_string()))]
        async fn call_transaction_various_recipient_addresses(
            #[case] recipient_address: &str,
            #[case] expected_error: VdrError,
        ) {
            let transaction = Transaction {
                to: Address::from(recipient_address),
                ..read_transaction()
            };
            let client = client();

            let error = client.submit_transaction(&transaction).await.unwrap_err();

            assert_eq!(error, expected_error);
        }

        #[rstest]
        #[case::regular_error(
            "0x08c379a00000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000001a4e6f7420656e6f7567682045746865722070726f76696465642e000000000000", 
            VdrError::ClientTransactionReverted("Error(message: Not enough Ether provided.)".to_string()),
        )]
        #[case::panic_error(
            "0x4e487b710000000000000000000000000000000000000000000000000000000000000011",
            VdrError::ClientTransactionReverted("Panic(code: 11)".to_string()),
        )]
        #[case::custom_error(
            "0x863b93fe000000000000000000000000f0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5",
            VdrError::ClientTransactionReverted("DidNotFound(identity: f0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5)".to_string()),
        )]
        #[case::error_without_required_argument(
            "0x863b93fe",
            VdrError::ContractInvalidResponseData("Unable to parse the revert reason '0x863b93fe': Failed to decode the arguments".to_string()),
        )]
        #[case::error_with_extra_argument(
            "0x4e487b71000000000000000000000000000000000000000000000000000000000000001100000000000000000000000000000000000000000000000000000000000011",
            VdrError::ClientTransactionReverted("Panic(code: 11)".to_string()),
        )]
        #[case::incorrect_error_selector(
            "0x9999999e000000000000000000000000f0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5", 
            VdrError::ContractInvalidResponseData("Unable to parse the revert reason '0x9999999e000000000000000000000000f0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5': Cannot match the error selector with registered errors".to_string())
        )]
        #[case::incorrect_hex(
            "0xQQ123456e00000000000000000000000f0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5", 
            VdrError::ContractInvalidResponseData("Unable to parse the revert reason '0xQQ123456e00000000000000000000000f0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5': Incorrect hex string".to_string())
        )]
        #[case::empty_data(
            "", 
            VdrError::ContractInvalidResponseData("Unable to parse the revert reason '': Incorrect data".to_string())
        )]
        #[case::incorrect_data(
            "0x9999", 
            VdrError::ContractInvalidResponseData("Unable to parse the revert reason '0x9999': Incorrect data".to_string())
        )]
        async fn handle_transaction_reverts(
            #[case] encoded_error_message: &'static str,
            #[case] expected_error: VdrError,
        ) {
            let mut transaction = Transaction {
                to: CONFIG.contracts.ethereum_did_registry.address.clone(),
                ..write_transaction()
            };
            let fake_signature = SignatureData {
                recovery_id: 1,
                signature: vec![1; 64],
            };
            transaction.set_signature(fake_signature);

            let mut client_mock = MockClient::new();
            client_mock
                .expect_submit_transaction()
                .with(eq(transaction.encode().unwrap()))
                .returning(|_| {
                    Err(VdrError::ClientTransactionReverted(
                        encoded_error_message.to_string(),
                    ))
                });

            let client = mock_custom_client(Box::new(client_mock));

            let actual_error = client.submit_transaction(&transaction).await.unwrap_err();

            assert_eq!(actual_error, expected_error);
        }

        #[async_std::test]
        async fn get_receipt_invalid_transaction_hash() {
            let client = client();
            let txn_hash = vec![1; 4];

            let receipt_err = client.get_receipt(&txn_hash).await.unwrap_err();

            assert!(matches!(
                receipt_err,  | VdrError::CommonInvalidData { .. }
            ));
        }

        #[async_std::test]
        async fn get_receipt_transaction_does_not_exist() {
            let mut client_mock = MockClient::new();
            let txn_hash = vec![1; 32];
            client_mock
                .expect_get_receipt()
                .with(eq(txn_hash.clone()))
                .returning(|_| {
                    Err(VdrError::ClientInvalidResponse(
                        "Missing transaction receipt".to_string(),
                    ))
                });

            let client = mock_custom_client(Box::new(client_mock));

            let receipt_err = client.get_receipt(&txn_hash).await.unwrap_err();

            assert!(matches!(
                receipt_err,  | VdrError::ClientInvalidResponse { .. }
            ));
        }

        #[async_std::test]
        async fn get_receipt_positive() {
            let mut client_mock = MockClient::new();
            let txn_hash = vec![1; 32];
            client_mock
                .expect_get_receipt()
                .with(eq(txn_hash.clone()))
                .returning(|_| Ok("".to_string()));

            let client = mock_custom_client(Box::new(client_mock));

            client.get_receipt(&txn_hash).await.unwrap();
        }

        #[async_std::test]
        async fn get_transaction_count_invalid_address() {
            let client = client();

            let get_nonce_err = client
                .get_transaction_count(&Address::from(INVALID_ADDRESS))
                .await
                .unwrap_err();

            assert!(matches!(
                get_nonce_err,  | VdrError::ClientInvalidTransaction { .. }
            ));
        }

        #[async_std::test]
        async fn get_contract_does_not_exist() {
            let client = client();

            let contract_err = client.contract(INVALID_ADDRESS).err().unwrap();

            assert!(matches!(
                contract_err,  | VdrError::ContractInvalidName { .. }
            ));
        }
    }

    #[cfg(feature = "ledger_test")]
    mod ping {
        use super::*;
        use crate::types::Status;

        #[async_std::test]
        async fn client_ping_test() {
            let client = client();
            match client.ping().await.unwrap().status {
                Status::Ok { .. } => {}
                Status::Err { .. } => assert!(false, "Ping status expected to be `Ok`."),
            }
        }

        #[async_std::test]
        async fn client_ping_wrong_node_test() {
            let wrong_node_address = "http://127.0.0.1:1111";
            let client = LedgerClient::new(
                CONFIG.chain_id,
                wrong_node_address,
                &contracts(),
                None,
                Some(&QuorumConfig::default()),
            )
            .unwrap();
            match client.ping().await.unwrap().status {
                Status::Err { .. } => {}
                Status::Ok { .. } => assert!(false, "Ping status expected to be `Err`."),
            }
        }
    }
}
