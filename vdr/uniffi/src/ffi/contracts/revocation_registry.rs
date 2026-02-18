use crate::{
    ffi::{
        client::LedgerClient,
        endorsing_data::TransactionEndorsingData,
        error::{VdrError, VdrResult},
        transaction::Transaction,
    },
    JsonValue,
};
use indy_besu_vdr::{
    revocation_registry, Address, CredentialDefinitionId, RegistryType,
    RevocationRegistryDefinition as RevocationRegistryDefinition_, RevocationRegistryDefinitionId,
    RevocationRegistryDefinitionValue, RevocationRegistryEntry as RevocationRegistryEntry_,
    RevocationStatusList as RevocationStatusList_, DID,
};
use serde::Deserialize;
use serde_json::json;
use std::str::FromStr;
use uniffi::export;

// Revocation Registry Definition functions

#[uniffi::export(async_runtime = "tokio")]
pub async fn build_create_revocation_registry_definition_transaction(
    client: &LedgerClient,
    from: &str,
    rev_reg_def: &RevocationRegistryDefinition,
) -> VdrResult<Transaction> {
    revocation_registry::build_create_revocation_registry_definition_transaction(
        &client.client,
        &Address::from(from),
        &RevocationRegistryDefinition_::from(rev_reg_def),
    )
    .await
    .map(Transaction::from)
    .map_err(VdrError::from)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn build_create_revocation_registry_definition_endorsing_data(
    client: &LedgerClient,
    rev_reg_def: &RevocationRegistryDefinition,
) -> VdrResult<TransactionEndorsingData> {
    revocation_registry::build_create_revocation_registry_definition_endorsing_data(
        &client.client,
        &RevocationRegistryDefinition_::from(rev_reg_def),
    )
    .await
    .map(TransactionEndorsingData::from)
    .map_err(VdrError::from)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn resolve_revocation_registry_definition(
    client: &LedgerClient,
    rev_reg_def_id: &str,
) -> VdrResult<RevocationRegistryDefinition> {
    revocation_registry::resolve_revocation_registry_definition(
        &client.client,
        &RevocationRegistryDefinitionId::from(rev_reg_def_id),
    )
    .await
    .map(RevocationRegistryDefinition::from)
    .map_err(VdrError::from)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn build_resolve_revocation_registry_definition_transaction(
    client: &LedgerClient,
    rev_reg_def_id: &str,
) -> VdrResult<Transaction> {
    revocation_registry::build_resolve_revocation_registry_definition_transaction(
        &client.client,
        &RevocationRegistryDefinitionId::from(rev_reg_def_id),
    )
    .await
    .map(Transaction::from)
    .map_err(VdrError::from)
}

#[uniffi::export]
pub fn parse_revocation_registry_definition(
    client: &LedgerClient,
    bytes: Vec<u8>,
) -> VdrResult<JsonValue> {
    let rev_reg_def = revocation_registry::parse_resolve_revocation_registry_definition_result(
        &client.client,
        &bytes,
    )?;
    Ok(JsonValue::from(json!(rev_reg_def)))
}

#[derive(uniffi::Record)]
pub struct RevocationRegistryDefinition {
    pub issuer_id: String,
    pub revoc_def_type: String,
    pub cred_def_id: String,
    pub tag: String,
    pub value: JsonValue,
}

impl From<RevocationRegistryDefinition_> for RevocationRegistryDefinition {
    fn from(rev_reg_def: RevocationRegistryDefinition_) -> Self {
        RevocationRegistryDefinition {
            issuer_id: rev_reg_def.issuer_id.as_ref().to_string(),
            revoc_def_type: rev_reg_def.revoc_def_type.to_str().to_string(),
            cred_def_id: rev_reg_def.cred_def_id.as_ref().to_string(),
            tag: rev_reg_def.tag,
            value: JsonValue::from(serde_json::json!(rev_reg_def.value)),
        }
    }
}

impl From<&RevocationRegistryDefinition> for RevocationRegistryDefinition_ {
    fn from(rev_reg_def: &RevocationRegistryDefinition) -> Self {
        RevocationRegistryDefinition_ {
            issuer_id: DID::from(rev_reg_def.issuer_id.as_str()),
            revoc_def_type: RegistryType::from_str(rev_reg_def.revoc_def_type.as_str()).unwrap(),
            cred_def_id: CredentialDefinitionId::from(rev_reg_def.cred_def_id.as_str()),
            tag: rev_reg_def.tag.to_string(),
            value: RevocationRegistryDefinitionValue::deserialize(
                rev_reg_def.value.clone().into_inner(),
            )
            .unwrap(),
        }
    }
}

#[uniffi::export]
pub fn revocation_registry_definition_get_id(rev_reg_def: &RevocationRegistryDefinition) -> String {
    RevocationRegistryDefinition_::from(rev_reg_def)
        .id()
        .as_ref()
        .to_string()
}

#[uniffi::export]
pub fn revocation_registry_definition_to_string(
    rev_reg_def: &RevocationRegistryDefinition,
) -> VdrResult<String> {
    let rev_reg = RevocationRegistryDefinition_::from(rev_reg_def);
    rev_reg.to_string().map_err(VdrError::from)
}

#[uniffi::export]
pub fn revocation_registry_definition_from_string(
    rev_reg_def_str: &str,
) -> VdrResult<RevocationRegistryDefinition> {
    RevocationRegistryDefinition_::from_string(rev_reg_def_str)
        .map(RevocationRegistryDefinition::from)
        .map_err(VdrError::from)
}

// Revocation Registry Entry functions

#[uniffi::export(async_runtime = "tokio")]
pub async fn build_create_revocation_registry_entry_transaction(
    client: &LedgerClient,
    from: &str,
    rev_reg_entry: &RevocationRegistryEntry,
) -> VdrResult<Transaction> {
    revocation_registry::build_create_revocation_registry_entry_transaction(
        &client.client,
        &Address::from(from),
        &RevocationRegistryEntry_::from(rev_reg_entry),
    )
    .await
    .map(Transaction::from)
    .map_err(VdrError::from)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn build_create_revocation_registry_entry_endorsing_data(
    client: &LedgerClient,
    rev_reg_entry: &RevocationRegistryEntry,
) -> VdrResult<TransactionEndorsingData> {
    revocation_registry::build_create_revocation_registry_entry_endorsing_data(
        &client.client,
        &RevocationRegistryEntry_::from(rev_reg_entry),
    )
    .await
    .map(TransactionEndorsingData::from)
    .map_err(VdrError::from)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn resolve_revocation_registry_status_list(
    client: &LedgerClient,
    rev_reg_def_id: &str,
    timestamp: u64,
) -> VdrResult<JsonValue> {
    revocation_registry::resolve_revocation_registry_status_list(
        &client.client,
        &RevocationRegistryDefinitionId::from(rev_reg_def_id),
        timestamp,
    )
    .await
    .map(|status_list| JsonValue::from(json!(status_list)))
    .map_err(VdrError::from)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn resolve_revocation_registry_status_list_full(
    client: &LedgerClient,
    rev_reg_def_id: &str,
    timestamp: u64,
) -> VdrResult<RevocationStatusList> {
    revocation_registry::resolve_revocation_registry_status_list(
        &client.client,
        &RevocationRegistryDefinitionId::from(rev_reg_def_id),
        timestamp,
    )
    .await
    .map(RevocationStatusList::from)
    .map_err(VdrError::from)
}

#[derive(uniffi::Record)]
pub struct RevocationRegistryEntry {
    issuer_id: String,
    rev_reg_def_id: String,
    rev_reg_entry_data: JsonValue,
}

impl From<&RevocationRegistryEntry> for RevocationRegistryEntry_ {
    fn from(entry: &RevocationRegistryEntry) -> Self {
        RevocationRegistryEntry_ {
            issuer_id: DID::from(entry.issuer_id.as_str()),
            rev_reg_def_id: RevocationRegistryDefinitionId::from(entry.rev_reg_def_id.as_str()),
            rev_reg_entry_data: serde_json::from_value(
                entry.rev_reg_entry_data.clone().into_inner(),
            )
            .unwrap(),
        }
    }
}

impl From<RevocationRegistryEntry_> for RevocationRegistryEntry {
    fn from(entry: RevocationRegistryEntry_) -> Self {
        RevocationRegistryEntry {
            issuer_id: entry.issuer_id.as_ref().to_string(),
            rev_reg_def_id: entry.rev_reg_def_id.as_ref().to_string(),
            rev_reg_entry_data: JsonValue::from(
                serde_json::to_value(entry.rev_reg_entry_data).unwrap(),
            ),
        }
    }
}

#[uniffi::export]
pub fn revocation_registry_entry_to_string(
    rev_reg_entry: &RevocationRegistryEntry,
) -> VdrResult<String> {
    RevocationRegistryEntry_::from(rev_reg_entry)
        .to_string()
        .map_err(VdrError::from)
}

#[uniffi::export]
pub fn revocation_registry_entry_from_string(
    rev_reg_entry_str: &str,
) -> VdrResult<RevocationRegistryEntry> {
    RevocationRegistryEntry_::from_string(rev_reg_entry_str)
        .map(RevocationRegistryEntry::from)
        .map_err(VdrError::from)
}

#[derive(uniffi::Record)]
pub struct RevocationStatusList {
    issuer_id: String,
    rev_reg_def_id: String,
    timestamp: u64,
    revocation_list: Vec<u32>,
    current_accumulator: String,
}

impl From<&RevocationStatusList> for RevocationStatusList_ {
    fn from(status_list: &RevocationStatusList) -> Self {
        RevocationStatusList_ {
            issuer_id: DID::from(status_list.issuer_id.as_str()),
            rev_reg_def_id: RevocationRegistryDefinitionId::from(
                status_list.rev_reg_def_id.as_str(),
            ),
            timestamp: status_list.timestamp,
            revocation_list: status_list.revocation_list.clone(),
            current_accumulator: status_list.current_accumulator.clone(),
        }
    }
}

impl From<RevocationStatusList_> for RevocationStatusList {
    fn from(status_list: RevocationStatusList_) -> Self {
        RevocationStatusList {
            issuer_id: status_list.issuer_id.as_ref().to_string(),
            rev_reg_def_id: status_list.rev_reg_def_id.as_ref().to_string(),
            timestamp: status_list.timestamp,
            revocation_list: status_list.revocation_list,
            current_accumulator: status_list.current_accumulator,
        }
    }
}

#[uniffi::export]
pub fn revocation_status_list_to_string(status_list: &RevocationStatusList) -> VdrResult<String> {
    RevocationStatusList_::from(status_list)
        .to_string()
        .map_err(VdrError::from)
}

#[uniffi::export]
pub fn revocation_status_list_from_string(
    status_list_str: &str,
) -> VdrResult<RevocationStatusList> {
    RevocationStatusList_::from_string(status_list_str)
        .map(RevocationStatusList::from)
        .map_err(VdrError::from)
}
