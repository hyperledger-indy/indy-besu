// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use indy_besu_vdr::{
    did_ethr_registry, Address, Block, DelegateType, DidDocAttribute, EventLog, Validity, DID,
};
use std::rc::Rc;
use wasm_bindgen::prelude::*;

use crate::{
    client::LedgerClientWrapper,
    error::{JsResult, Result},
    event_query::EventQueryWrapper,
    transaction::{TransactionEndorsingDataWrapper, TransactionWrapper},
};

#[wasm_bindgen(js_name = EthrDidRegistry)]
pub struct EthrDidRegistry;

#[wasm_bindgen(js_class = EthrDidRegistry)]
impl EthrDidRegistry {
    #[wasm_bindgen(js_name = buildDidChangeOwnerTransaction)]
    pub async fn build_did_change_owner_transaction(
        client: &LedgerClientWrapper,
        sender: &str,
        did: &str,
        new_owner: &str,
    ) -> Result<TransactionWrapper> {
        let sender = Address::from(sender);
        let did = DID::from(did);
        let new_owner = Address::from(new_owner);
        did_ethr_registry::build_did_change_owner_transaction(&client.0, &sender, &did, &new_owner)
            .await
            .as_js()
            .map(TransactionWrapper::from)
            .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = buildDidChangeOwnerEndorsingData)]
    pub async fn build_did_change_owner_endorsing_data(
        client: &LedgerClientWrapper,
        did: &str,
        new_owner: &str,
    ) -> Result<TransactionEndorsingDataWrapper> {
        let did = DID::from(did);
        let new_owner = Address::from(new_owner);
        did_ethr_registry::build_did_change_owner_endorsing_data(&client.0, &did, &new_owner)
            .await
            .as_js()
            .map(TransactionEndorsingDataWrapper::from)
            .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = buildDidAddDelegateTransaction)]
    pub async fn build_did_add_delegate_transaction(
        client: &LedgerClientWrapper,
        sender: &str,
        did: &str,
        delegate_type: &str,
        delegate: &str,
        validity: u64,
    ) -> Result<TransactionWrapper> {
        let sender = Address::from(sender);
        let did = DID::from(did);
        let delegate_type = DelegateType::try_from(delegate_type).as_js()?;
        let delegate = Address::from(delegate);
        let validity = Validity::from(validity);
        did_ethr_registry::build_did_add_delegate_transaction(
            &client.0,
            &sender,
            &did,
            &delegate_type,
            &delegate,
            &validity,
        )
        .await
        .as_js()
        .map(TransactionWrapper::from)
        .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = buildDidAddDelegateEndorsingData)]
    pub async fn build_did_add_delegate_endorsing_data(
        client: &LedgerClientWrapper,
        did: &str,
        delegate_type: &str,
        delegate: &str,
        validity: u64,
    ) -> Result<TransactionEndorsingDataWrapper> {
        let did = DID::from(did);
        let delegate_type = DelegateType::try_from(delegate_type).as_js()?;
        let delegate = Address::from(delegate);
        let validity = Validity::from(validity);
        did_ethr_registry::build_did_add_delegate_endorsing_data(
            &client.0,
            &did,
            &delegate_type,
            &delegate,
            &validity,
        )
        .await
        .as_js()
        .map(TransactionEndorsingDataWrapper::from)
        .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = buildDidRevokeDelegateTransaction)]
    pub async fn build_did_revoke_delegate_transaction(
        client: &LedgerClientWrapper,
        sender: &str,
        did: &str,
        delegate_type: &str,
        delegate: &str,
    ) -> Result<TransactionWrapper> {
        let sender = Address::from(sender);
        let did = DID::from(did);
        let delegate_type = DelegateType::try_from(delegate_type).as_js()?;
        let delegate = Address::from(delegate);
        did_ethr_registry::build_did_revoke_delegate_transaction(
            &client.0,
            &sender,
            &did,
            &delegate_type,
            &delegate,
        )
        .await
        .as_js()
        .map(TransactionWrapper::from)
        .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = buildDidRevokeDelegateEndorsingData)]
    pub async fn build_did_revoke_delegate_endorsing_data(
        client: &LedgerClientWrapper,
        did: &str,
        delegate_type: &str,
        delegate: &str,
    ) -> Result<TransactionEndorsingDataWrapper> {
        let did = DID::from(did);
        let delegate_type = DelegateType::try_from(delegate_type).as_js()?;
        let delegate = Address::from(delegate);
        did_ethr_registry::build_did_revoke_delegate_endorsing_data(
            &client.0,
            &did,
            &delegate_type,
            &delegate,
        )
        .await
        .as_js()
        .map(TransactionEndorsingDataWrapper::from)
        .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = buildDidSetAttributeTransaction)]
    pub async fn build_did_set_attribute_transaction(
        client: &LedgerClientWrapper,
        sender: &str,
        did: &str,
        attribute: JsValue,
        validity: u64,
    ) -> Result<TransactionWrapper> {
        let sender = Address::from(sender);
        let did = DID::from(did);
        let did_attribute: DidDocAttribute = serde_wasm_bindgen::from_value(attribute)?;
        let validity = Validity::from(validity);
        did_ethr_registry::build_did_set_attribute_transaction(
            &client.0,
            &sender,
            &did,
            &did_attribute,
            &validity,
        )
        .await
        .as_js()
        .map(TransactionWrapper::from)
        .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = buildDidSetAttributeEndorsingData)]
    pub async fn build_did_set_attribute_endorsing_data(
        client: &LedgerClientWrapper,
        did: &str,
        attribute: JsValue,
        validity: u64,
    ) -> Result<TransactionEndorsingDataWrapper> {
        let did = DID::from(did);
        let did_attribute: DidDocAttribute = serde_wasm_bindgen::from_value(attribute)?;
        let validity = Validity::from(validity);
        did_ethr_registry::build_did_set_attribute_endorsing_data(
            &client.0,
            &did,
            &did_attribute,
            &validity,
        )
        .await
        .as_js()
        .map(TransactionEndorsingDataWrapper::from)
        .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = buildDidRevokeAttributeTransaction)]
    pub async fn build_did_revoke_attribute_transaction(
        client: &LedgerClientWrapper,
        sender: &str,
        did: &str,
        attribute: JsValue,
    ) -> Result<TransactionWrapper> {
        let sender = Address::from(sender);
        let did = DID::from(did);
        let did_attribute: DidDocAttribute = serde_wasm_bindgen::from_value(attribute)?;
        did_ethr_registry::build_did_revoke_attribute_transaction(
            &client.0,
            &sender,
            &did,
            &did_attribute,
        )
        .await
        .as_js()
        .map(TransactionWrapper::from)
        .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = buildDidRevokeAttributeEndorsingData)]
    pub async fn build_did_revoke_attribute_endorsing_data(
        client: &LedgerClientWrapper,
        did: &str,
        attribute: JsValue,
    ) -> Result<TransactionEndorsingDataWrapper> {
        let did = DID::from(did);
        let did_attribute: DidDocAttribute = serde_wasm_bindgen::from_value(attribute)?;
        did_ethr_registry::build_did_revoke_attribute_endorsing_data(
            &client.0,
            &did,
            &did_attribute,
        )
        .await
        .as_js()
        .map(TransactionEndorsingDataWrapper::from)
        .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = buildGetDidOwnerTransaction)]
    pub async fn build_get_did_owner_transaction(
        client: &LedgerClientWrapper,
        did: &str,
    ) -> Result<TransactionWrapper> {
        let did = DID::from(did);
        did_ethr_registry::build_get_did_owner_transaction(&client.0, &did)
            .await
            .as_js()
            .map(TransactionWrapper::from)
            .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = buildGetDidChangedTransaction)]
    pub async fn build_get_did_changed_transaction(
        client: &LedgerClientWrapper,
        did: &str,
    ) -> Result<TransactionWrapper> {
        let did = DID::from(did);
        did_ethr_registry::build_get_did_changed_transaction(&client.0, &did)
            .await
            .as_js()
            .map(TransactionWrapper::from)
            .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = buildGetIdentityNonceTransaction)]
    pub async fn build_get_identity_nonce_transaction(
        client: &LedgerClientWrapper,
        identity: &str,
    ) -> Result<TransactionWrapper> {
        let identity = Address::from(identity);
        did_ethr_registry::build_get_identity_nonce_transaction(&client.0, &identity)
            .await
            .as_js()
            .map(TransactionWrapper::from)
            .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = buildGetDidEventsQuery)]
    pub async fn build_get_did_events_query(
        client: &LedgerClientWrapper,
        did: &str,
        from_block: Option<u64>,
        to_block: Option<u64>,
    ) -> Result<EventQueryWrapper> {
        let did = DID::from(did);
        let from_block = from_block.map(Block::from);
        let to_block = to_block.map(Block::from);
        let query = did_ethr_registry::build_get_did_events_query(
            &client.0,
            &did,
            from_block.as_ref(),
            to_block.as_ref(),
        )
        .await
        .as_js()?;
        Ok(EventQueryWrapper(Rc::new(query)))
    }

    #[wasm_bindgen(js_name = parseDidChangedResult)]
    pub fn parse_did_changed_result(client: &LedgerClientWrapper, bytes: Vec<u8>) -> Result<u64> {
        let block = did_ethr_registry::parse_did_changed_result(&client.0, &bytes).as_js()?;
        Ok(block.value())
    }

    #[wasm_bindgen(js_name = parseDidOwnerResult)]
    pub fn parse_did_owner_result(client: &LedgerClientWrapper, bytes: Vec<u8>) -> Result<String> {
        let owner = did_ethr_registry::parse_did_owner_result(&client.0, &bytes).as_js()?;
        Ok(owner.to_string())
    }

    #[wasm_bindgen(js_name = parseDidAttributeChangedEventResponse)]
    pub fn parse_did_attribute_changed_event_response(
        client: &LedgerClientWrapper,
        log: JsValue,
    ) -> Result<JsValue> {
        let log: EventLog = serde_wasm_bindgen::from_value(log)?;
        let event = did_ethr_registry::parse_did_attribute_changed_event_response(&client.0, &log)
            .as_js()?;
        let result: JsValue = serde_wasm_bindgen::to_value(&event)?;
        Ok(result)
    }

    #[wasm_bindgen(js_name = parseDidDelegateChangedEventResponse)]
    pub fn parse_did_delegate_changed_event_response(
        client: &LedgerClientWrapper,
        log: JsValue,
    ) -> Result<JsValue> {
        let log: EventLog = serde_wasm_bindgen::from_value(log)?;
        let event = did_ethr_registry::parse_did_delegate_changed_event_response(&client.0, &log)
            .as_js()?;
        let result: JsValue = serde_wasm_bindgen::to_value(&event)?;
        Ok(result)
    }

    #[wasm_bindgen(js_name = parseDidOwnerChangedEventResponse)]
    pub fn parse_did_owner_changed_event_response(
        client: &LedgerClientWrapper,
        log: JsValue,
    ) -> Result<JsValue> {
        let log: EventLog = serde_wasm_bindgen::from_value(log)?;
        let event =
            did_ethr_registry::parse_did_owner_changed_event_response(&client.0, &log).as_js()?;
        let result: JsValue = serde_wasm_bindgen::to_value(&event)?;
        Ok(result)
    }

    #[wasm_bindgen(js_name = parseDidEventResponse)]
    pub fn parse_did_event_response(client: &LedgerClientWrapper, log: JsValue) -> Result<JsValue> {
        let log: EventLog = serde_wasm_bindgen::from_value(log)?;
        let event = did_ethr_registry::parse_did_event_response(&client.0, &log).as_js()?;
        let result: JsValue = serde_wasm_bindgen::to_value(&event)?;
        Ok(result)
    }
}
