use std::collections::HashMap;

use crate::{
    client::LedgerClient,
    error::{VdrError, VdrResult},
    types::{PingStatus, Transaction},
    QuorumConfig,
};

const DEFAULT_NETWORK: &str = "default";

/// Defines the operating mode for the LedgerRouter
#[derive(Debug, Clone)]
pub enum LedgerMode {
    /// Return a fully initialized LedgerClient (cached mode)
    LedgerClient,
    /// Return only configuration data without instantiating clients
    ConfigOnly,
}

impl Default for LedgerMode {
    fn default() -> Self {
        LedgerMode::ConfigOnly
    }
}

/// Result returned by get_ledger_for_identifier
#[derive(Debug)]
pub enum LedgerResult {
    /// A fully initialized LedgerClient
    Client(LedgerClient),
    /// A configuration-only LedgerClientConfig
    Config(LedgerClientConfig),
}

/// Configuration for a specific ledger network
#[derive(Debug, Clone)]
pub struct LedgerClientConfig {
    pub chain_id: u64,
    pub rpc_node: String,
    pub contract_configs: Vec<crate::types::ContractConfig>,
    pub network: Option<String>,
    pub quorum_config: Option<QuorumConfig>,
}

impl LedgerClientConfig {
    /// Creates a new LedgerClientConfig instance
    pub fn new(
        chain_id: u64,
        rpc_node: &str,
        contract_configs: &[crate::types::ContractConfig],
        network: Option<&str>,
        quorum_config: Option<&QuorumConfig>,
    ) -> Self {
        Self {
            chain_id,
            rpc_node: rpc_node.to_string(),
            contract_configs: contract_configs.to_vec(),
            network: network.map(str::to_string),
            quorum_config: quorum_config.cloned(),
        }
    }
}

/// Router responsible for managing multiple ledgers
pub struct LedgerRouter {
    configs: HashMap<String, LedgerClientConfig>,
    default_network: String,
    mode: LedgerMode,
}

impl LedgerRouter {
    /// Creates a new LedgerRouter instance
    ///
    /// # Parameters
    /// - `configs`: HashMap<String, LedgerClientConfig> — ledger configurations by network name
    /// - `default_network`: Option<&str> — default network name if none is provided
    /// - `mode`: LedgerMode — determines whether the router returns configs or instantiated clients
    ///
    /// # Returns
    /// A new instance of LedgerRouter
    pub fn new(
        configs: HashMap<String, LedgerClientConfig>,
        default_network: Option<&str>,
        mode: Option<LedgerMode>,
    ) -> Self {
        LedgerRouter {
            configs,
            default_network: default_network.unwrap_or(DEFAULT_NETWORK).to_string(),
            mode: mode.unwrap_or_default(),
        }
    }

    /// Returns a ledger configuration or client based on a given identifier
    ///
    /// # Parameters
    /// - `identifier`: &str — for example, `"did:ethr:testnet:0xabc..."`
    ///
    /// # Returns
    /// A LedgerResult, which may contain either a client or a configuration
    pub fn get_ledger_for_identifier(&self, identifier: &str) -> VdrResult<LedgerResult> {
        let network = Self::extract_network(identifier)?;

        let config = self
            .configs
            .get(&network)
            .or_else(|| self.configs.get(&self.default_network))
            .ok_or_else(|| {
                VdrError::RouterConfigError(format!(
                    "Ledger for network '{}' not configured and no default network defined",
                    network
                ))
            })?;

        match self.mode {
            LedgerMode::LedgerClient => {
                let client = LedgerClient::new(
                    config.chain_id,
                    &config.rpc_node,
                    &config.contract_configs,
                    config.network.as_deref(),
                    config.quorum_config.as_ref(),
                )?;
                Ok(LedgerResult::Client(client))
            }
            LedgerMode::ConfigOnly => Ok(LedgerResult::Config(config.clone())),
        }
    }

    /// Extracts the network name from an identifier string
    ///
    /// # Parameters
    /// - `identifier`: &str — e.g., `"did:ethr:testnet:0xabc"`
    ///
    /// # Returns
    /// The extracted network name as String
    pub fn extract_network(identifier: &str) -> VdrResult<String> {
        let parts: Vec<&str> = identifier.split(':').collect();

        if parts.len() < 3 {
            return Err(VdrError::RouterConfigError(format!(
                "Invalid DID identifier: {}",
                identifier
            )));
        }

        if parts[0] != "did" || parts[1] != "ethr" {
            return Err(VdrError::RouterConfigError(format!(
                "Unsupported DID method: {}",
                identifier
            )));
        }

        match parts.len() {
            3 => Ok(DEFAULT_NETWORK.to_string()),
            n if n >= 4 => Ok(parts[2].to_string()),
            _ => Err(VdrError::RouterConfigError(format!(
                "Invalid did:ethr format: {}",
                identifier
            ))),
        }
    }

    /// Submits a transaction to a ledger based on its identifier
    ///
    /// # Parameters
    /// - `identifier`: String — ledger identifier
    /// - `transaction`: Transaction — transaction data
    ///
    /// # Returns
    /// A Vec<u8> response from the ledger
    pub async fn submit_transaction_for_identifier(
        &self,
        identifier: String,
        transaction: &Transaction,
    ) -> VdrResult<Vec<u8>> {
        match self.get_ledger_for_identifier(&identifier)? {
            LedgerResult::Client(client) => client.submit_transaction(transaction).await,
            LedgerResult::Config(_) => Err(VdrError::RouterConfigError(
                "Cannot submit transaction in ConfigOnly mode".to_string(),
            )),
        }
    }

    /// Pings all configured ledgers (only works in `LedgerClient` mode)
    ///
    /// # Returns
    /// A map of network names and their ping statuses
    pub async fn ping_all(&self) -> VdrResult<HashMap<String, PingStatus>> {
        if let LedgerMode::ConfigOnly = self.mode {
            return Err(VdrError::RouterConfigError(
                "Cannot ping ledgers in ConfigOnly mode".to_string(),
            ));
        }

        let mut results = HashMap::new();
        for (network, config) in self.configs.iter() {
            let client = LedgerClient::new(
                config.chain_id,
                &config.rpc_node,
                &config.contract_configs,
                config.network.as_deref(),
                config.quorum_config.as_ref(),
            )?;
            let ping_status = client.ping().await?;
            results.insert(network.clone(), ping_status);
        }
        Ok(results)
    }

    /// Returns a list of all configured networks
    pub fn list_networks(&self) -> Vec<String> {
        self.configs.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::HashMap, sync::Arc};

    fn mock_client_config(network: &str) -> LedgerClientConfig {
        LedgerClientConfig {
            chain_id: 1337,
            rpc_node: "http://localhost:8545".to_string(),
            contract_configs: vec![],
            network: Some(network.to_string()),
            quorum_config: None,
        }
    }

    fn setup_router_config_mode() -> LedgerRouter {
        let mut configs = HashMap::new();
        configs.insert("mainnet".to_string(), mock_client_config("mainnet"));
        configs.insert("testnet".to_string(), mock_client_config("testnet"));

        LedgerRouter::new(configs, Some("mainnet"), Some(LedgerMode::ConfigOnly))
    }

    fn setup_router_client_mode() -> LedgerRouter {
        let mut configs = HashMap::new();
        configs.insert("mainnet".to_string(), mock_client_config("mainnet"));
        configs.insert("testnet".to_string(), mock_client_config("testnet"));

        LedgerRouter::new(configs, Some("mainnet"), Some(LedgerMode::LedgerClient))
    }

    #[test]
    fn test_get_ledger_existing_network_config_mode() {
        let router = setup_router_config_mode();
        let id = "did:ethr:testnet:0x123";

        let result = router.get_ledger_for_identifier(id).unwrap();

        match result {
            LedgerResult::Config(config) => {
                assert_eq!(config.network, Some("testnet".to_string()));
            }
            LedgerResult::Client(_) => panic!("Expected LedgerResult::Config, got Client"),
        }
    }

    #[test]
    fn test_get_ledger_uses_default_when_missing_network_config_mode() {
        let router = setup_router_config_mode();
        let id = "did:ethr:unknownnet:0x123";

        let result = router.get_ledger_for_identifier(id).unwrap();

        match result {
            LedgerResult::Config(config) => {
                assert_eq!(config.network, Some("mainnet".to_string()));
            }
            LedgerResult::Client(_) => panic!("Expected LedgerResult::Config, got Client"),
        }
    }

    #[test]
    fn test_get_ledger_existing_network_client_mode() {
        let router = setup_router_client_mode();
        let id = "did:ethr:testnet:0x123";

        let result = router.get_ledger_for_identifier(id).unwrap();

        match result {
            LedgerResult::Client(client) => {
                assert_eq!(client.network(), Some(&"testnet".to_string()));
            }
            LedgerResult::Config(_) => panic!("Expected LedgerResult::Client, got Config"),
        }
    }

    #[test]
    fn test_get_ledger_uses_default_when_missing_network_client_mode() {
        let router = setup_router_client_mode();
        let id = "did:ethr:unknownnet:0x123";

        let result = router.get_ledger_for_identifier(id).unwrap();

        match result {
            LedgerResult::Client(client) => {
                assert_eq!(client.network(), Some(&"mainnet".to_string()));
            }
            LedgerResult::Config(_) => panic!("Expected LedgerResult::Client, got Config"),
        }
    }
}
