// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{
    ffi::{
        client::LedgerClient,
        endorsing_data::TransactionEndorsingData,
        error::{VdrError, VdrResult},
        transaction::Transaction,
    },
    EventLog, EventQuery,
};
use indy_besu_vdr::{
    did_ethr_registry, Address, Block, DelegateType, DidAttributeChanged as DidAttributeChanged_,
    DidDelegateChanged as DidDelegateChanged_, DidDocAttribute, DidEvents as DidEvents_,
    DidOwnerChanged as DidOwnerChanged_, Validity, DID,
};

#[uniffi::export(async_runtime = "tokio")]
pub async fn build_did_change_owner_transaction(
    client: &LedgerClient,
    from: &str,
    did: &str,
    new_owner: &str,
) -> VdrResult<Transaction> {
    did_ethr_registry::build_did_change_owner_transaction(
        &client.client,
        &Address::from(from),
        &DID::from(did),
        &Address::from(new_owner),
    )
    .await
    .map(Transaction::from)
    .map_err(VdrError::from)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn build_did_change_owner_endorsing_data(
    client: &LedgerClient,
    did: &str,
    new_owner: &str,
) -> VdrResult<TransactionEndorsingData> {
    did_ethr_registry::build_did_change_owner_endorsing_data(
        &client.client,
        &DID::from(did),
        &Address::from(new_owner),
    )
    .await
    .map(TransactionEndorsingData::from)
    .map_err(VdrError::from)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn build_did_add_delegate_transaction(
    client: &LedgerClient,
    from: &str,
    did: &str,
    delegate_type: &str,
    delegate: &str,
    validity: u64,
) -> VdrResult<Transaction> {
    did_ethr_registry::build_did_add_delegate_transaction(
        &client.client,
        &Address::from(from),
        &DID::from(did),
        &DelegateType::try_from(delegate_type)?,
        &Address::from(delegate),
        &Validity::from(validity),
    )
    .await
    .map(Transaction::from)
    .map_err(VdrError::from)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn build_did_add_delegate_endorsing_data(
    client: &LedgerClient,
    did: &str,
    delegate_type: &str,
    delegate: &str,
    validity: u64,
) -> VdrResult<TransactionEndorsingData> {
    did_ethr_registry::build_did_add_delegate_endorsing_data(
        &client.client,
        &DID::from(did),
        &DelegateType::try_from(delegate_type)?,
        &Address::from(delegate),
        &Validity::from(validity),
    )
    .await
    .map(TransactionEndorsingData::from)
    .map_err(VdrError::from)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn build_did_revoke_delegate_transaction(
    client: &LedgerClient,
    from: &str,
    did: &str,
    delegate_type: &str,
    delegate: &str,
) -> VdrResult<Transaction> {
    did_ethr_registry::build_did_revoke_delegate_transaction(
        &client.client,
        &Address::from(from),
        &DID::from(did),
        &DelegateType::try_from(delegate_type)?,
        &Address::from(delegate),
    )
    .await
    .map(Transaction::from)
    .map_err(VdrError::from)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn build_did_revoke_delegate_endorsing_data(
    client: &LedgerClient,
    did: &str,
    delegate_type: &str,
    delegate: &str,
) -> VdrResult<TransactionEndorsingData> {
    did_ethr_registry::build_did_revoke_delegate_endorsing_data(
        &client.client,
        &DID::from(did),
        &DelegateType::try_from(delegate_type)?,
        &Address::from(delegate),
    )
    .await
    .map(TransactionEndorsingData::from)
    .map_err(VdrError::from)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn build_did_set_attribute_transaction(
    client: &LedgerClient,
    from: &str,
    did: &str,
    attribute: &str,
    validity: u64,
) -> VdrResult<Transaction> {
    let attribute: DidDocAttribute =
        serde_json::from_str(attribute).map_err(|err| VdrError::CommonInvalidData {
            msg: format!("Unable to parse DID Attribute. Err: {:?}", err),
        })?;

    did_ethr_registry::build_did_set_attribute_transaction(
        &client.client,
        &Address::from(from),
        &DID::from(did),
        &attribute,
        &Validity::from(validity),
    )
    .await
    .map(Transaction::from)
    .map_err(VdrError::from)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn build_did_set_attribute_endorsing_data(
    client: &LedgerClient,
    did: &str,
    attribute: &str,
    validity: u64,
) -> VdrResult<TransactionEndorsingData> {
    let attribute: DidDocAttribute =
        serde_json::from_str(attribute).map_err(|err| VdrError::CommonInvalidData {
            msg: format!("Unable to parse DID Attribute. Err: {:?}", err),
        })?;

    did_ethr_registry::build_did_set_attribute_endorsing_data(
        &client.client,
        &DID::from(did),
        &attribute,
        &Validity::from(validity),
    )
    .await
    .map(TransactionEndorsingData::from)
    .map_err(VdrError::from)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn build_did_revoke_attribute_transaction(
    client: &LedgerClient,
    from: &str,
    did: &str,
    attribute: &str,
) -> VdrResult<Transaction> {
    let attribute: DidDocAttribute =
        serde_json::from_str(attribute).map_err(|err| VdrError::CommonInvalidData {
            msg: format!("Unable to parse DID Attribute. Err: {:?}", err),
        })?;

    did_ethr_registry::build_did_revoke_attribute_transaction(
        &client.client,
        &Address::from(from),
        &DID::from(did),
        &attribute,
    )
    .await
    .map(Transaction::from)
    .map_err(VdrError::from)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn build_did_revoke_attribute_endorsing_data(
    client: &LedgerClient,
    did: &str,
    attribute: &str,
) -> VdrResult<TransactionEndorsingData> {
    let attribute: DidDocAttribute =
        serde_json::from_str(attribute).map_err(|err| VdrError::CommonInvalidData {
            msg: format!("Unable to parse DID Attribute. Err: {:?}", err),
        })?;

    did_ethr_registry::build_did_revoke_attribute_endorsing_data(
        &client.client,
        &DID::from(did),
        &attribute,
    )
    .await
    .map(TransactionEndorsingData::from)
    .map_err(VdrError::from)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn build_get_did_owner_transaction(
    client: &LedgerClient,
    did: &str,
) -> VdrResult<Transaction> {
    did_ethr_registry::build_get_did_owner_transaction(&client.client, &DID::from(did))
        .await
        .map(Transaction::from)
        .map_err(VdrError::from)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn build_get_did_changed_transaction(
    client: &LedgerClient,
    did: &str,
) -> VdrResult<Transaction> {
    did_ethr_registry::build_get_did_changed_transaction(&client.client, &DID::from(did))
        .await
        .map(Transaction::from)
        .map_err(VdrError::from)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn build_get_identity_nonce_transaction(
    client: &LedgerClient,
    identity: &str,
) -> VdrResult<Transaction> {
    did_ethr_registry::build_get_identity_nonce_transaction(
        &client.client,
        &Address::from(identity),
    )
    .await
    .map(Transaction::from)
    .map_err(VdrError::from)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn build_get_did_events_query(
    client: &LedgerClient,
    did: &str,
    from_block: Option<u64>,
    to_block: Option<u64>,
) -> VdrResult<EventQuery> {
    did_ethr_registry::build_get_did_events_query(
        &client.client,
        &DID::from(did),
        from_block.map(Block::from).as_ref(),
        to_block.map(Block::from).as_ref(),
    )
    .await
    .map(EventQuery::from)
    .map_err(VdrError::from)
}

#[uniffi::export]
pub fn parse_did_changed_result(client: &LedgerClient, bytes: Vec<u8>) -> VdrResult<u64> {
    let block = did_ethr_registry::parse_did_changed_result(&client.client, &bytes)?;
    Ok(block.value())
}

#[uniffi::export]
pub fn parse_did_nonce_result(client: &LedgerClient, bytes: Vec<u8>) -> VdrResult<u64> {
    let block = did_ethr_registry::parse_did_nonce_result(&client.client, &bytes)?;
    Ok(block.value())
}

#[uniffi::export]
pub fn parse_did_owner_result(client: &LedgerClient, bytes: Vec<u8>) -> VdrResult<String> {
    let address = did_ethr_registry::parse_did_owner_result(&client.client, &bytes)?;
    Ok(address.to_string())
}

#[uniffi::export]
pub fn parse_did_attribute_changed_event_response(
    client: &LedgerClient,
    log: EventLog,
) -> VdrResult<DidAttributeChanged> {
    did_ethr_registry::parse_did_attribute_changed_event_response(&client.client, &log.into())
        .map(DidAttributeChanged::from)
        .map_err(VdrError::from)
}

#[uniffi::export]
pub fn parse_did_delegate_changed_event_response(
    client: &LedgerClient,
    log: EventLog,
) -> VdrResult<DidDelegateChanged> {
    did_ethr_registry::parse_did_delegate_changed_event_response(&client.client, &log.into())
        .map(DidDelegateChanged::from)
        .map_err(VdrError::from)
}

#[uniffi::export]
pub fn parse_did_owner_changed_event_response(
    client: &LedgerClient,
    log: EventLog,
) -> VdrResult<DidOwnerChanged> {
    did_ethr_registry::parse_did_owner_changed_event_response(&client.client, &log.into())
        .map(DidOwnerChanged::from)
        .map_err(VdrError::from)
}

#[uniffi::export]
pub fn parse_did_event_response(client: &LedgerClient, log: EventLog) -> VdrResult<DidEvents> {
    did_ethr_registry::parse_did_event_response(&client.client, &log.into())
        .map(DidEvents::from)
        .map_err(VdrError::from)
}

#[derive(uniffi::Record)]
pub struct DidAttributeChanged {
    pub identity: String,
    pub name: String,
    pub value: Vec<u8>,
    pub valid_to: u64,
    pub previous_change: u64,
}

impl From<DidAttributeChanged_> for DidAttributeChanged {
    fn from(event: DidAttributeChanged_) -> Self {
        DidAttributeChanged {
            identity: event.identity.to_string(),
            name: event.name,
            value: event.value,
            valid_to: event.valid_to,
            previous_change: event.previous_change.value(),
        }
    }
}

#[derive(uniffi::Record)]
pub struct DidDelegateChanged {
    pub identity: String,
    pub delegate: String,
    pub delegate_type: Vec<u8>,
    pub valid_to: u64,
    pub previous_change: u64,
}

impl From<DidDelegateChanged_> for DidDelegateChanged {
    fn from(event: DidDelegateChanged_) -> Self {
        DidDelegateChanged {
            identity: event.identity.to_string(),
            delegate: event.delegate.to_string(),
            delegate_type: event.delegate_type,
            valid_to: event.valid_to,
            previous_change: event.previous_change.value(),
        }
    }
}

#[derive(uniffi::Record)]
pub struct DidOwnerChanged {
    pub identity: String,
    pub owner: String,
    pub previous_change: u64,
}

impl From<DidOwnerChanged_> for DidOwnerChanged {
    fn from(event: DidOwnerChanged_) -> Self {
        DidOwnerChanged {
            identity: event.identity.to_string(),
            owner: event.owner.to_string(),
            previous_change: event.previous_change.value(),
        }
    }
}

#[derive(uniffi::Enum)]
pub enum DidEvents {
    AttributeChangedEvent { event: DidAttributeChanged },
    DelegateChanged { event: DidDelegateChanged },
    OwnerChanged { event: DidOwnerChanged },
}

impl From<DidEvents_> for DidEvents {
    fn from(event: DidEvents_) -> Self {
        match event {
            DidEvents_::AttributeChangedEvent(event) => DidEvents::AttributeChangedEvent {
                event: event.into(),
            },
            DidEvents_::DelegateChanged(event) => DidEvents::DelegateChanged {
                event: event.into(),
            },
            DidEvents_::OwnerChanged(event) => DidEvents::OwnerChanged {
                event: event.into(),
            },
        }
    }
}
