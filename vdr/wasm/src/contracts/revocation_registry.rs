// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use indy_besu_vdr::{
    revocation_registry, AccumKey, Accumulator, Address, CredentialDefinitionId, PublicKeys,
    RegistryType, RevocationRegistryDefinition, RevocationRegistryDefinitionId,
    RevocationRegistryDefinitionValue, RevocationRegistryEntry,
    RevocationRegistryEntryData, RevocationState, RevocationStatusList, VdrResult, DID,
};
use serde_json::value;
use std::borrow::Borrow;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

use crate::transaction::{TransactionEndorsingDataWrapper, TransactionWrapper};
use crate::{
    client::LedgerClientWrapper,
    error::{JsResult, Result},
};

#[wasm_bindgen(js_name = RevocationRegistry)]
pub struct RevocationRegistry;

#[wasm_bindgen(js_class = RevocationRegistry)]
impl RevocationRegistry {
    #[wasm_bindgen(js_name = buildCreateRevocationRegistryDefinitionTransaction)]
    pub async fn build_create_revocation_registry_definition_transaction(
        client: &LedgerClientWrapper,
        from: &str,
        rev_reg_def: &RevocationRegistryDefinitionWrapper,
    ) -> Result<TransactionWrapper> {
        let client = client.0.clone();
        let address = Address::from(from);
        revocation_registry::build_create_revocation_registry_definition_transaction(
            &client,
            &address,
            &rev_reg_def.0,
        )
        .await
        .as_js()
        .map(TransactionWrapper::from)
        .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = buildCreateRevocationRegistryDefinitionEndorsingData)]
    pub async fn build_create_revocation_registry_definition_endorsing_data(
        client: &LedgerClientWrapper,
        rev_reg_def: &RevocationRegistryDefinitionWrapper,
    ) -> Result<TransactionEndorsingDataWrapper> {
        revocation_registry::build_create_revocation_registry_definition_endorsing_data(
            &client.0,
            &rev_reg_def.0,
        )
        .await
        .as_js()
        .map(TransactionEndorsingDataWrapper::from)
        .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = buildCreateRevocationRegistryEntryTransaction)]
    pub async fn build_create_revocation_registry_entry_transaction(
        client: &LedgerClientWrapper,
        from: &str,
        rev_reg_entry: &RevocationRegistryEntryWrapper,
    ) -> Result<TransactionWrapper> {
        let client = client.0.clone();
        let address = Address::from(from);
        revocation_registry::build_create_revocation_registry_entry_transaction(
            &client,
            &address,
            &rev_reg_entry.0,
        )
        .await
        .as_js()
        .map(TransactionWrapper::from)
        .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = buildCreateRevocationRegistryEntryEndorsingData)]
    pub async fn build_create_revocation_registry_entry_endorsing_data(
        client: &LedgerClientWrapper,
        rev_reg_entry: &RevocationRegistryEntryWrapper,
    ) -> Result<TransactionEndorsingDataWrapper> {
        revocation_registry::build_create_revocation_registry_entry_endorsing_data(
            &client.0,
            &rev_reg_entry.0,
        )
        .await
        .as_js()
        .map(TransactionEndorsingDataWrapper::from)
        .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = buildResolveRevocationRegistryDefinitionTransaction)]
    pub async fn build_resolve_revocation_registry_definition_transaction(
        client: &LedgerClientWrapper,
        id: &str,
    ) -> Result<TransactionWrapper> {
        let id = RevocationRegistryDefinitionId::from(id);
        revocation_registry::build_resolve_revocation_registry_definition_transaction(
            &client.0, &id,
        )
        .await
        .as_js()
        .map(TransactionWrapper::from)
        .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = parseResolveRevocationRegistryDefinitionResult)]
    pub fn parse_resolve_revocation_registry_definition_result(
        client: &LedgerClientWrapper,
        bytes: Vec<u8>,
    ) -> Result<JsValue> {
        let rev_reg_def = revocation_registry::parse_resolve_revocation_registry_definition_result(
            &client.0, &bytes,
        )
        .as_js()?;
        let result: JsValue = serde_wasm_bindgen::to_value(&rev_reg_def)?;
        Ok(result)
    }

    #[wasm_bindgen(js_name = resolveRevocationRegistryDefinition)]
    pub async fn resolve_revocation_registry_definition(
        client: &LedgerClientWrapper,
        id: &str,
    ) -> Result<RevocationRegistryDefinitionWrapper> {
        let id = RevocationRegistryDefinitionId::from(id);
        revocation_registry::resolve_revocation_registry_definition(&client.0, &id)
            .await
            .as_js()
            .map(RevocationRegistryDefinitionWrapper::from)
            .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = resolveRevocationRegistryStatusList)]
    pub async fn resolve_revocation_registry_status_list(
        client: &LedgerClientWrapper,
        id: &str,
        to_timestamp: u64,
    ) -> Result<RevocationRegistryStatusListWrapper> {
        let id = RevocationRegistryDefinitionId::from(id);
        revocation_registry::resolve_revocation_registry_status_list(&client.0, &id, to_timestamp)
            .await
            .as_js()
            .map(RevocationRegistryStatusListWrapper::from)
            .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = buildLatestRevocationRegistryEntryFromStatusList)]
    pub async fn build_latest_revocation_registry_entry_from_status_list(
        client: &LedgerClientWrapper,
        id: &str,
        revocation_registry_status_list: Vec<u8>,
        accumulator: &str,
    ) -> Result<RevocationRegistryEntryWrapper> {
        let revocation_registry_status_list = revocation_registry_status_list
            .into_iter()
            .map(RevocationState::try_from)
            .collect::<VdrResult<Vec<RevocationState>>>()
            .as_js()?;

        let id = RevocationRegistryDefinitionId::from(id);

        revocation_registry::build_latest_revocation_registry_entry_from_status_list(
            &client.0,
            &id,
            &revocation_registry_status_list,
            accumulator.to_string(),
        )
        .await
        .as_js()
        .map(RevocationRegistryEntryWrapper::from)
        .map_err(JsValue::from)
    }
}

#[wasm_bindgen(js_name = RevocationRegistryDefinition)]
pub struct RevocationRegistryDefinitionWrapper(pub(crate) Rc<RevocationRegistryDefinition>);

#[wasm_bindgen(js_class = RevocationRegistryDefinition)]
impl RevocationRegistryDefinitionWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new(
        issuer_id: String,
        cred_def_id: String,
        tag: String,
        value: JsValue,
    ) -> RevocationRegistryDefinitionWrapper {
        //TODO: probably a better way to do this
        // let accum_key = AccumKey { z };
        // let public_keys = PublicKeys { accum_key };
        RevocationRegistryDefinitionWrapper(Rc::new(RevocationRegistryDefinition {
            issuer_id: DID::from(issuer_id.as_str()),
            cred_def_id: CredentialDefinitionId::from(cred_def_id.as_str()),
            revoc_def_type: RegistryType::CL_ACCUM,
            tag,
            value: serde_wasm_bindgen::from_value(value).unwrap(),
        }))
    }

    #[wasm_bindgen(js_name = getId)]
    pub fn get_id(&self) -> String {
        self.0.id().as_ref().to_string()
    }

    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> Result<String> {
        self.0.to_string().as_js().map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = fromString)]
    pub fn from_string(string: &str) -> Result<RevocationRegistryDefinitionWrapper> {
        RevocationRegistryDefinition::from_string(string)
            .as_js()
            .map(RevocationRegistryDefinitionWrapper::from)
            .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = asValue)]
    pub fn as_value(&self) -> Result<JsValue> {
        serde_wasm_bindgen::to_value(&*self.0).map_err(JsValue::from)
    }
}

impl From<RevocationRegistryDefinition> for RevocationRegistryDefinitionWrapper {
    fn from(data: RevocationRegistryDefinition) -> RevocationRegistryDefinitionWrapper {
        RevocationRegistryDefinitionWrapper(Rc::new(data))
    }
}

#[wasm_bindgen(js_name = RevocationRegistryEntry)]
pub struct RevocationRegistryEntryWrapper(pub(crate) Rc<RevocationRegistryEntry>);

#[wasm_bindgen(js_class = RevocationRegistryEntry)]
impl RevocationRegistryEntryWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new(
        issuer_id: String,
        rev_reg_def_id: String,
        current_accumulator: String,
        prev_accumulator: Option<String>,
        issued: Option<Vec<u32>>,
        revoked: Option<Vec<u32>>,
    ) -> RevocationRegistryEntryWrapper {
        RevocationRegistryEntryWrapper(Rc::new(RevocationRegistryEntry {
            rev_reg_def_id: RevocationRegistryDefinitionId::from(rev_reg_def_id.as_str()),
            issuer_id: DID::from(issuer_id.as_str()),
            rev_reg_entry_data: RevocationRegistryEntryData {
                current_accumulator: Accumulator::from(current_accumulator.as_str()),
                prev_accumulator: match prev_accumulator {
                    Some(prev_accumulator) => Some(Accumulator::from(prev_accumulator.as_str())),
                    None => None,
                },
                issued,
                revoked,
            },
        }))
    }

    #[wasm_bindgen(js_name = asValue)]
    pub fn as_value(&self) -> Result<JsValue> {
        serde_wasm_bindgen::to_value(&*self.0).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> Result<String> {
        self.0.to_string().as_js().map_err(JsValue::from)
    }
}

impl From<RevocationRegistryEntry> for RevocationRegistryEntryWrapper {
    fn from(data: RevocationRegistryEntry) -> RevocationRegistryEntryWrapper {
        RevocationRegistryEntryWrapper(Rc::new(data))
    }
}

#[wasm_bindgen(js_name = RevocationRegistryStatusList)]
pub struct RevocationRegistryStatusListWrapper(pub(crate) Rc<RevocationStatusList>);

#[wasm_bindgen(js_class = RevocationRegistryStatusList)]
impl RevocationRegistryStatusListWrapper {
    #[wasm_bindgen(js_name = asValue)]
    pub fn as_value(&self) -> Result<JsValue> {
        serde_wasm_bindgen::to_value(&*self.0).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> Result<String> {
        self.0.to_string().as_js().map_err(JsValue::from)
    }
}

impl From<RevocationStatusList> for RevocationRegistryStatusListWrapper {
    fn from(data: RevocationStatusList) -> RevocationRegistryStatusListWrapper {
        RevocationRegistryStatusListWrapper(Rc::new(data))
    }
}
