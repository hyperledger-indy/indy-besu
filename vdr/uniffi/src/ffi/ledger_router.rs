use std::{collections::HashMap, sync::Arc};

use crate::{
    ffi::{
        error::VdrResult,
        transaction::Transaction,
        types::{ContractConfig, PingStatus, QuorumConfig},
        LedgerClient,
    },
    VdrError,
};

use indy_besu_vdr::{
    ContractConfig as ContractConfig_, LedgerClientConfig, LedgerMode, LedgerResult,
    LedgerRouter as LedgerRouter_,
};

use uniffi_macros::export;

/// Represents the configuration for a single ledger network.
#[derive(uniffi::Record)]
pub struct LedgerConfiguration {
    /// The EVM chain ID of the target network.
    pub chain_id: u64,
    /// The RPC endpoint or node address of the ledger.
    pub node_address: String,
    /// List of contract configurations deployed in this ledger.
    pub contract_configs: Vec<ContractConfig>,
    /// Optional network name (e.g., "mainnet", "testnet").
    pub network: Option<String>,
    /// Optional quorum configuration, if applicable.
    pub quorum_config: Option<QuorumConfig>,
}

/// A wrapper around the internal `LedgerRouter` exposed through UniFFI for use in Python, Swift, etc.
#[derive(uniffi::Object)]
pub struct LedgerRouter {
    router: Arc<LedgerRouter_>,
}

#[export(async_runtime = "tokio")]
impl LedgerRouter {
    /// Creates a new LedgerRouter instance from a list of LedgerConfiguration records.
    ///
    /// # Parameters
    /// - `configs`: A vector of [`LedgerConfiguration`] representing network configurations.
    ///
    /// # Returns
    /// A [`LedgerRouter`] instance ready to interact with multiple ledgers.
    ///
    /// # Example
    /// ```python
    /// router = LedgerRouter.new([
    ///     LedgerConfiguration(
    ///         chain_id=1337,
    ///         node_address="http://localhost:8545",
    ///         contract_configs=[],
    ///         network="testnet",
    ///         quorum_config=None
    ///     )
    /// ])
    /// ```
    #[uniffi::constructor]
    pub fn new(configs: Vec<LedgerConfiguration>) -> VdrResult<LedgerRouter> {
        let mut ledgers: HashMap<String, LedgerClientConfig> = HashMap::new();
        let mut default_network_name: Option<String> = None;

        for config in configs {
            // Resolve the network name or default to "default"
            let network_name = config
                .network
                .as_ref()
                .cloned()
                .unwrap_or_else(|| "default".to_string());

            // Define the default network (first key or "default" if empty)
            if default_network_name.is_none() {
                default_network_name = Some(network_name.clone());
            }

            // Map Quorum and contract configurations to internal types
            let quorum_config: Option<indy_besu_vdr::QuorumConfig> =
                config.quorum_config.map(QuorumConfig::into);

            let contract_configs: Vec<ContractConfig_> = config
                .contract_configs
                .into_iter()
                .map(ContractConfig::into)
                .collect();

            // Build the LedgerClientConfig for this network
            let ledger_config = LedgerClientConfig::new(
                config.chain_id,
                &config.node_address,
                &contract_configs,
                config.network.as_deref(),
                quorum_config.as_ref(),
            );

            // Insert into the router configuration map
            ledgers.insert(network_name, ledger_config);
        }

        let default_network = default_network_name.unwrap_or_else(|| "default".to_string());

        // Initialize the underlying LedgerRouter (client mode)
        let router = LedgerRouter_::new(
            ledgers,
            Some(&default_network),
            Some(LedgerMode::LedgerClient),
        );

        Ok(LedgerRouter {
            router: Arc::new(router),
        })
    }

    /// Retrieves a `LedgerClient` instance for a specific identifier (e.g., DID).
    ///
    /// # Parameters
    /// - `identifier`: A DID or address that encodes the network name (e.g., `did:ethr:testnet:0x123`).
    ///
    /// # Returns
    /// A [`LedgerClient`] ready for transactions and queries.
    ///
    /// # Errors
    /// Returns [`VdrError::ClientInvalidResponse`] if the configuration mode is used instead of client mode.
    pub fn get_ledger_for_identifier(&self, identifier: &str) -> VdrResult<LedgerClient> {
        let ledger_result = self.router.get_ledger_for_identifier(identifier)?;

        let client = match ledger_result {
            LedgerResult::Client(c) => c,
            LedgerResult::Config(_) => {
                return Err(VdrError::ClientInvalidResponse {
                    msg: "Expected LedgerClient but got config".to_string(),
                });
            }
        };
        Ok(LedgerClient { client })
    }

    /// Pings all configured ledgers and returns their current status.
    ///
    /// # Returns
    /// A HashMap mapping network names to their [`PingStatus`] results.
    ///
    /// # Example
    /// ```python
    /// status = await router.ping_all()
    /// print(status["testnet"].status)
    /// ```
    pub async fn ping_all(&self) -> VdrResult<HashMap<String, PingStatus>> {
        let internal_result = self.router.ping_all().await?;
        let mapped = internal_result
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect();
        Ok(mapped)
    }

    /// Submits a transaction to the ledger corresponding to a given identifier.
    ///
    /// # Parameters
    /// - `identifier`: The target ledger identifier (e.g., DID or address).
    /// - `transaction`: The transaction object to be submitted.
    ///
    /// # Returns
    /// A raw Vec<u8> representing the transaction result or receipt.
    ///
    /// # Example
    /// ```python
    /// result = await router.submit_transaction_for_identifier("did:ethr:testnet:0x123", tx)
    /// ```
    pub async fn submit_transaction_for_identifier(
        &self,
        identifier: String,
        transaction: &Transaction,
    ) -> VdrResult<Vec<u8>> {
        self.router
            .submit_transaction_for_identifier(identifier, &transaction.into())
            .await
            .map_err(VdrError::from)
    }

    /// Lists all configured network names in this router instance.
    ///
    /// # Returns
    /// A vector of network identifiers such as `["mainnet", "testnet"]`.
    ///
    /// # Example
    /// ```python
    /// networks = router.list_networks()
    /// print(networks)
    /// ```
    pub fn list_networks(&self) -> Vec<String> {
        self.router.list_networks()
    }
}
