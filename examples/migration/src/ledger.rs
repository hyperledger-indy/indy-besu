// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::wallet::{BesuWallet, IndyWallet};
use indy_besu_vdr::{
    credential_definition_registry::{
        build_create_credential_definition_transaction, resolve_credential_definition,
    },
    did_ethr_registry::build_did_set_attribute_transaction,
    legacy_mapping_registry::{
        build_create_did_mapping_transaction, build_create_resource_mapping_transaction,
        build_get_resource_mapping_transaction,
        parse_resource_mapping_result,
    },
    role_control::build_assign_role_transaction,
    schema_registry::{build_create_schema_transaction, resolve_schema},
    Address, ContractConfig, DidDocAttribute, Ed25519Signature, LedgerClient, LegacyDid,
    LegacyVerkey, ResourceIdentifier, Role, Status, Transaction, Validity,
};
use indy_credx::types::{AttributeNames, CredentialDefinition, Schema};
use indy_data_types::{CredentialDefinitionId, SchemaId};
use indy_vdr::{
    config::PoolConfig,
    ledger::constants::UpdateRole,
    pool::{
        helpers::perform_ledger_request, LocalPool, Pool, PoolBuilder, PoolTransactions,
        PreparedRequest, RequestResult,
    },
    utils::did::DidValue,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{env, fs, str::FromStr, time::Duration};

pub enum Ledgers {
    Indy,
    Besu,
}

pub struct IndyLedger {
    pool: LocalPool,
}

impl IndyLedger {
    pub fn new() -> IndyLedger {
        let mut cur_dir = env::current_dir().unwrap();
        cur_dir.push("indy-genesis.txn");
        let pool_transactions = PoolTransactions::from_json_file(cur_dir.as_path()).unwrap();

        let pool = PoolBuilder::new(PoolConfig::default(), pool_transactions)
            .into_local()
            .unwrap();
        IndyLedger { pool }
    }

    pub async fn publish_nym(
        &self,
        wallet: &IndyWallet,
        submitter_did: &str,
        target_did: &str,
        verkey: &str,
        role: Option<&str>,
    ) {
        let mut request = self
            .pool
            .get_request_builder()
            .build_nym_request(
                &DidValue(submitter_did.to_string()),
                &DidValue(target_did.to_string()),
                Some(verkey.to_string()),
                None,
                role.map(|role| UpdateRole::from_str(role).unwrap()),
                None,
                None,
            )
            .unwrap();

        self._sign_and_submit_request(wallet, &mut request).await;
    }

    pub async fn publish_attrib(
        &self,
        wallet: &IndyWallet,
        submitter_did: &str,
        target_did: &str,
        attrib: &serde_json::Value,
    ) {
        let mut request = self
            .pool
            .get_request_builder()
            .build_attrib_request(
                &DidValue(submitter_did.to_string()),
                &DidValue(target_did.to_string()),
                None,
                Some(attrib),
                None,
            )
            .unwrap();

        self._sign_and_submit_request(wallet, &mut request).await;
    }

    pub async fn publish_schema(&self, wallet: &IndyWallet, submitter_did: &str, schema: &Schema) {
        let mut request = self
            .pool
            .get_request_builder()
            .build_schema_request(&DidValue(submitter_did.to_string()), schema.clone())
            .unwrap();

        self._sign_and_submit_request(wallet, &mut request).await;
    }

    pub async fn publish_cred_def(
        &self,
        wallet: &IndyWallet,
        submitter_did: &str,
        cred_def: &CredentialDefinition,
    ) {
        // hack to clone cred def
        let cred_def_json = json!(cred_def).to_string();
        let cred_def = serde_json::from_str(&cred_def_json).unwrap();

        let mut request = self
            .pool
            .get_request_builder()
            .build_cred_def_request(&DidValue(submitter_did.to_string()), cred_def)
            .unwrap();

        self._sign_and_submit_request(wallet, &mut request).await;
    }

    pub async fn get_schema(&self, id: &str) -> Schema {
        let request = self
            .pool
            .get_request_builder()
            .build_get_schema_request(
                None,
                &indy_vdr::ledger::identifiers::SchemaId(id.to_string()),
            )
            .expect("Unable to build get schema request");

        let response = self._submit_request(&request).await;
        serde_json::from_value(json!({
            "name": response["result"]["data"]["name"].as_str().unwrap().to_string(),
            "version": response["result"]["data"]["version"].as_str().unwrap().to_string(),
            "attrNames": serde_json::from_value::<AttributeNames>(response["result"]["data"]["attr_names"].clone()).unwrap(),
            "id": SchemaId(id.to_string()),
            "seqNo": Some(response["result"]["seqNo"].as_u64().unwrap() as u32),
            "ver": "1.0"
        })).unwrap()
    }

    pub async fn get_cred_def(&self, id: &str) -> CredentialDefinition {
        let request = self
            .pool
            .get_request_builder()
            .build_get_cred_def_request(
                None,
                &indy_vdr::ledger::identifiers::CredentialDefinitionId(id.to_string()),
            )
            .expect("Unable to build get cred def request");

        let response = self._submit_request(&request).await;
        let seq_no = response["result"]["ref"].as_u64().unwrap().to_string();
        serde_json::from_value(json!({
            "id": CredentialDefinitionId(id.to_string()),
            "schemaId": seq_no,
            "type": response["result"]["signature_type"],
            "tag": response["result"]["tag"],
            "value": response["result"]["data"],
            "ver": "1.0"
        }))
        .unwrap()
    }

    async fn _sign_and_submit_request(
        &self,
        wallet: &IndyWallet,
        request: &mut PreparedRequest,
    ) -> serde_json::Value {
        let sig_bytes = request.get_signature_input().unwrap();
        let signature = wallet.sign(sig_bytes.as_bytes()).await;
        request.set_signature(&signature).unwrap();
        self._submit_request(request).await
    }

    async fn _submit_request(&self, request: &PreparedRequest) -> serde_json::Value {
        let (request_result, _) = perform_ledger_request(&self.pool, request).await.unwrap();
        std::thread::sleep(Duration::from_millis(500));
        match request_result {
            RequestResult::Reply(message) => serde_json::from_str(&message).unwrap(),
            RequestResult::Failed(error) => {
                panic!("Unable to send request. Err: {:?}", error);
            }
        }
    }
}

pub struct BesuLedger {
    pub client: LedgerClient,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BesuConfig {
    chain_id: u64,
    node_address: String,
    contracts: BesuContracts,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BesuContracts {
    ethereum_did_registry: BesuContractConfig,
    indy_did_registry: BesuContractConfig,
    cred_def_registry: BesuContractConfig,
    schema_registry: BesuContractConfig,
    role_control: BesuContractConfig,
    validator_control: BesuContractConfig,
    account_control: BesuContractConfig,
    upgrade_control: BesuContractConfig,
    legacy_mapping_registry: BesuContractConfig,
    universal_did_resolver: BesuContractConfig,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BesuContractConfig {
    address: String,
    spec_path: String,
}

fn build_contract_path(contract_path: &str) -> String {
    let mut cur_dir = env::current_dir().unwrap();
    cur_dir.push("../../"); // project root directory
    cur_dir.push(contract_path);
    fs::canonicalize(&cur_dir)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

impl BesuLedger {

    fn contracts(contracts: &BesuContracts) -> Vec<ContractConfig> {
        vec![
            ContractConfig {
                address: contracts.ethereum_did_registry.address.to_string(),
                spec_path: Some(build_contract_path(contracts.ethereum_did_registry.spec_path.as_str())),
                spec: None,
            },
            ContractConfig {
                address: contracts.schema_registry.address.to_string(),
                spec_path: Some(build_contract_path(contracts.schema_registry.spec_path.as_str())),
                spec: None,
            },
            ContractConfig {
                address: contracts.cred_def_registry.address.to_string(),
                spec_path: Some(build_contract_path(contracts.cred_def_registry.spec_path.as_str())),
                spec: None,
            },
            ContractConfig {
                address: contracts.role_control.address.to_string(),
                spec_path: Some(build_contract_path(contracts.role_control.spec_path.as_str())),
                spec: None,
            },
            ContractConfig {
                address: contracts.legacy_mapping_registry.address.to_string(),
                spec_path: Some(build_contract_path(contracts.legacy_mapping_registry.spec_path.as_str())),
                spec: None,
            },
        ]
    }

    pub async fn new() -> BesuLedger {
        let file = fs::File::open("../../network/config.json").expect("Unable to open besu config file");
        let config: BesuConfig =
            serde_json::from_reader(file).expect("Unable to parse besu config file");

        let client = LedgerClient::new(
            config.chain_id,
            config.node_address.as_str(),
            &Self::contracts(&config.contracts),
            None,
        )
        .unwrap();
        let status = client.ping().await.unwrap();
        match status.status {
            Status::Ok { .. } => {
                // ok
            }
            Status::Err { .. } => {
                panic!("Besu network is not reachable")
            }
        };

        BesuLedger { client }
    }

    pub async fn assign_role(
        &self,
        account: &Address,
        role: &Role,
        to: &Address,
        wallet: &BesuWallet,
    ) {
        let transaction = build_assign_role_transaction(&self.client, account, role, to)
            .await
            .unwrap();
        self.sign_and_submit_transaction(&transaction, wallet, account)
            .await;
    }

    pub async fn publish_did_attribute(
        &self,
        account: &Address,
        did: &str,
        attribute: &DidDocAttribute,
        wallet: &BesuWallet,
    ) {
        let did = indy_besu_vdr::DID::from(did);
        let transaction = build_did_set_attribute_transaction(
            &self.client,
            account,
            &did,
            attribute,
            &Validity::from(10000),
        )
        .await
        .unwrap();
        self.sign_and_submit_transaction(&transaction, wallet, account)
            .await;
    }

    pub async fn publish_did_mapping(
        &self,
        account: &Address,
        did: &str,
        legacy_did: &str,
        legacy_verkey: &str,
        legacy_signature: &[u8],
        wallet: &BesuWallet,
    ) {
        let transaction = build_create_did_mapping_transaction(
            &self.client,
            account,
            &indy_besu_vdr::DID::from(did),
            &LegacyDid::from(legacy_did),
            &LegacyVerkey::from(legacy_verkey),
            &Ed25519Signature::from(legacy_signature),
        )
        .await
        .unwrap();
        self.sign_and_submit_transaction(&transaction, wallet, account)
            .await;
    }

    pub async fn publish_schema(
        &self,
        account: &Address,
        schema: &indy_besu_vdr::Schema,
        wallet: &BesuWallet,
    ) -> indy_besu_vdr::SchemaId {
        let transaction =
            build_create_schema_transaction(&self.client, account, schema)
                .await
                .unwrap();
        self.sign_and_submit_transaction(&transaction, wallet, account)
            .await;
        schema.id()
    }

    pub async fn publish_resource_mapping(
        &self,
        account: &Address,
        did: &indy_besu_vdr::DID,
        legacy_did: &indy_data_types::did::DidValue,
        legacy_id: &ResourceIdentifier,
        new_id: &ResourceIdentifier,
        wallet: &BesuWallet,
    ) {
        let transaction = build_create_resource_mapping_transaction(
            &self.client,
            account,
            did,
            &LegacyDid::from(legacy_did.0.as_str()),
            legacy_id,
            new_id,
        )
        .await
        .unwrap();
        self.sign_and_submit_transaction(&transaction, wallet, account)
            .await;
    }

    pub async fn publish_cred_def(
        &self,
        account: &Address,
        cred_def: &indy_besu_vdr::CredentialDefinition,
        wallet: &BesuWallet,
    ) -> indy_besu_vdr::CredentialDefinitionId {
        let transaction = build_create_credential_definition_transaction(
            &self.client,
            account,
            cred_def,
        )
        .await
        .unwrap();
        self.sign_and_submit_transaction(&transaction, wallet, account)
            .await;
        cred_def.id()
    }

    async fn sign_and_submit_transaction(
        &self,
        transaction: &Transaction,
        wallet: &BesuWallet,
        account: &Address,
    ) {
        let signature = wallet
            .signer
            .sign(&transaction.get_signing_bytes().unwrap(), account.as_ref())
            .unwrap();
        transaction.set_signature(signature);
        let hash = self.client.submit_transaction(transaction).await.unwrap();
        let _receipt = self.client.get_receipt(&hash).await.unwrap();
    }

    pub async fn get_schema(&self, id: &str) -> Schema {
        let schema_id = self.get_resource_mapping(id).await;
        let schema = resolve_schema(
            &self.client,
            &indy_besu_vdr::SchemaId::from(schema_id.as_ref()),
        )
        .await
        .unwrap();
        (&schema).into()
    }

    pub async fn get_cred_def(&self, id: &str) -> CredentialDefinition {
        let cred_def_id = self.get_resource_mapping(id).await;
        let cred_def = resolve_credential_definition(
            &self.client,
            &&indy_besu_vdr::CredentialDefinitionId::from(cred_def_id.as_ref()),
        )
        .await
        .unwrap();
        (&cred_def).into()
    }

    pub async fn get_resource_mapping(&self, id: &str) -> indy_besu_vdr::ResourceIdentifier {
        let transaction =
            build_get_resource_mapping_transaction(&self.client, &ResourceIdentifier::from(id))
                .await
                .unwrap();
        let response = self.client.submit_transaction(&transaction).await.unwrap();
        parse_resource_mapping_result(&self.client, &response).unwrap()
    }
}
