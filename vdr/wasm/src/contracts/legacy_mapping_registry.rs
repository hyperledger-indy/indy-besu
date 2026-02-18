// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use indy_besu_vdr::{
    legacy_mapping_registry, Address, Ed25519Signature, LegacyDid, LegacyVerkey,
    ResourceIdentifier, DID,
};
use wasm_bindgen::prelude::*;

use crate::{
    client::LedgerClientWrapper,
    error::{JsResult, Result},
    transaction::{TransactionEndorsingDataWrapper, TransactionWrapper},
};

#[wasm_bindgen(js_name = LegacyMappingRegistry)]
pub struct LegacyMappingRegistry;

#[wasm_bindgen(js_class = LegacyMappingRegistry)]
impl LegacyMappingRegistry {
    #[wasm_bindgen(js_name = buildCreateDidMappingTransaction)]
    pub async fn build_create_did_mapping_transaction(
        client: &LedgerClientWrapper,
        from: &str,
        did: &str,
        legacy_did: &str,
        legacy_verkey: &str,
        ed25519_signature: &[u8],
    ) -> Result<TransactionWrapper> {
        let from = Address::from(from);
        let did = DID::from(did);
        let legacy_did = LegacyDid::from(legacy_did);
        let legacy_verkey = LegacyVerkey::from(legacy_verkey);
        let ed25519_signature = Ed25519Signature::from(ed25519_signature);
        legacy_mapping_registry::build_create_did_mapping_transaction(
            &client.0,
            &from,
            &did,
            &legacy_did,
            &legacy_verkey,
            &ed25519_signature,
        )
        .await
        .as_js()
        .map(TransactionWrapper::from)
        .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = buildCreateDidMappingEndorsingData)]
    pub async fn build_create_did_mapping_endorsing_data(
        client: &LedgerClientWrapper,
        did: &str,
        legacy_did: &str,
        legacy_verkey: &str,
        ed25519_signature: &[u8],
    ) -> Result<TransactionEndorsingDataWrapper> {
        let did = DID::from(did);
        let legacy_did = LegacyDid::from(legacy_did);
        let legacy_verkey = LegacyVerkey::from(legacy_verkey);
        let ed25519_signature = Ed25519Signature::from(ed25519_signature);
        legacy_mapping_registry::build_create_did_mapping_endorsing_data(
            &client.0,
            &did,
            &legacy_did,
            &legacy_verkey,
            &ed25519_signature,
        )
        .await
        .as_js()
        .map(TransactionEndorsingDataWrapper::from)
        .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = buildGetDidMappingTransaction)]
    pub async fn build_get_did_mapping_transaction(
        client: &LedgerClientWrapper,
        legacy_identifier: &str,
    ) -> Result<TransactionWrapper> {
        let legacy_identifier = LegacyDid::from(legacy_identifier);
        legacy_mapping_registry::build_get_did_mapping_transaction(&client.0, &legacy_identifier)
            .await
            .as_js()
            .map(TransactionWrapper::from)
            .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = parseDidMappingResult)]
    pub fn parse_did_mapping_result(
        client: &LedgerClientWrapper,
        bytes: Vec<u8>,
    ) -> Result<String> {
        let did = legacy_mapping_registry::parse_did_mapping_result(&client.0, &bytes).as_js()?;
        Ok(did.to_string())
    }

    #[wasm_bindgen(js_name = buildCreateResourceMappingTransaction)]
    pub async fn build_create_resource_mapping_transaction(
        client: &LedgerClientWrapper,
        from: &str,
        did: &str,
        legacy_issuer_identifier: &str,
        legacy_identifier: &str,
        new_identifier: &str,
    ) -> Result<TransactionWrapper> {
        let from = Address::from(from);
        let did = DID::from(did);
        let legacy_issuer_identifier = LegacyDid::from(legacy_issuer_identifier);
        let legacy_identifier = ResourceIdentifier::from(legacy_identifier);
        let new_identifier = ResourceIdentifier::from(new_identifier);
        legacy_mapping_registry::build_create_resource_mapping_transaction(
            &client.0,
            &from,
            &did,
            &legacy_issuer_identifier,
            &legacy_identifier,
            &new_identifier,
        )
        .await
        .as_js()
        .map(TransactionWrapper::from)
        .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = buildCreateResourceMappingEndorsingData)]
    pub async fn build_create_resource_mapping_endorsing_data(
        client: &LedgerClientWrapper,
        did: &str,
        legacy_issuer_identifier: &str,
        legacy_identifier: &str,
        new_identifier: &str,
    ) -> Result<TransactionEndorsingDataWrapper> {
        let did = DID::from(did);
        let legacy_issuer_identifier = LegacyDid::from(legacy_issuer_identifier);
        let legacy_identifier = ResourceIdentifier::from(legacy_identifier);
        let new_identifier = ResourceIdentifier::from(new_identifier);
        legacy_mapping_registry::build_create_resource_mapping_endorsing_data(
            &client.0,
            &did,
            &legacy_issuer_identifier,
            &legacy_identifier,
            &new_identifier,
        )
        .await
        .as_js()
        .map(TransactionEndorsingDataWrapper::from)
        .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = buildGetResourceMappingTransaction)]
    pub async fn build_get_resource_mapping_transaction(
        client: &LedgerClientWrapper,
        legacy_identifier: &str,
    ) -> Result<TransactionWrapper> {
        let legacy_identifier = ResourceIdentifier::from(legacy_identifier);
        legacy_mapping_registry::build_get_resource_mapping_transaction(
            &client.0,
            &legacy_identifier,
        )
        .await
        .as_js()
        .map(TransactionWrapper::from)
        .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = parseResourceMappingResult)]
    pub fn parse_resource_mapping_result(
        client: &LedgerClientWrapper,
        bytes: Vec<u8>,
    ) -> Result<String> {
        let resource =
            legacy_mapping_registry::parse_resource_mapping_result(&client.0, &bytes).as_js()?;
        Ok(resource.to_string())
    }
}
