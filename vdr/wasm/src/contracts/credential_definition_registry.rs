// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use indy_besu_vdr::{
    credential_definition_registry, Address, CredentialDefinition, CredentialDefinitionId,
    SchemaId, SignatureType, DID,
};
use std::rc::Rc;
use wasm_bindgen::prelude::*;

use crate::{
    client::LedgerClientWrapper,
    error::{JsResult, Result},
    transaction::{TransactionEndorsingDataWrapper, TransactionWrapper},
};

#[wasm_bindgen(js_name = CredentialDefinitionRegistry)]
pub struct CredentialDefinitionRegistry;

#[wasm_bindgen(js_class = CredentialDefinitionRegistry)]
impl CredentialDefinitionRegistry {
    #[wasm_bindgen(js_name = buildCreateCredentialDefinitionTransaction)]
    pub async fn build_create_credential_definition_transaction(
        client: &LedgerClientWrapper,
        from: &str,
        cred_def: &CredentialDefinitionWrapper,
    ) -> Result<TransactionWrapper> {
        let client = client.0.clone();
        let address = Address::from(from);
        credential_definition_registry::build_create_credential_definition_transaction(
            &client,
            &address,
            &cred_def.0,
        )
        .await
        .as_js()
        .map(TransactionWrapper::from)
        .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = buildCreateCredentialDefinitionEndorsingData)]
    pub async fn build_create_credential_definition_endorsing_data(
        client: &LedgerClientWrapper,
        cred_def: &CredentialDefinitionWrapper,
    ) -> Result<TransactionEndorsingDataWrapper> {
        credential_definition_registry::build_create_credential_definition_endorsing_data(
            &client.0,
            &cred_def.0,
        )
        .await
        .as_js()
        .map(TransactionEndorsingDataWrapper::from)
        .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = buildResolveCredentialDefinitionTransaction)]
    pub async fn build_resolve_credential_definition_transaction(
        client: &LedgerClientWrapper,
        id: &str,
    ) -> Result<TransactionWrapper> {
        let id = CredentialDefinitionId::from(id);
        credential_definition_registry::build_resolve_credential_definition_transaction(
            &client.0, &id,
        )
        .await
        .as_js()
        .map(TransactionWrapper::from)
        .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = parseResolveCredentialDefinitionResult)]
    pub fn parse_resolve_credential_definition_result(
        client: &LedgerClientWrapper,
        bytes: Vec<u8>,
    ) -> Result<JsValue> {
        let cred_def = credential_definition_registry::parse_resolve_credential_definition_result(
            &client.0, &bytes,
        )
        .as_js()?;
        let result: JsValue = serde_wasm_bindgen::to_value(&cred_def)?;
        Ok(result)
    }

    #[wasm_bindgen(js_name = resolveCredentialDefinition)]
    pub async fn resolve_credential_definition(
        client: &LedgerClientWrapper,
        id: &str,
    ) -> Result<CredentialDefinitionWrapper> {
        let id = CredentialDefinitionId::from(id);
        credential_definition_registry::resolve_credential_definition(&client.0, &id)
            .await
            .as_js()
            .map(CredentialDefinitionWrapper::from)
            .map_err(JsValue::from)
    }
}

#[wasm_bindgen(js_name = CredentialDefinition)]
pub struct CredentialDefinitionWrapper(pub(crate) Rc<CredentialDefinition>);

#[wasm_bindgen(js_class = CredentialDefinition)]
impl CredentialDefinitionWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new(
        issuer_id: String,
        schema_id: String,
        tag: String,
        value: JsValue,
    ) -> CredentialDefinitionWrapper {
        let value: serde_json::Value = serde_wasm_bindgen::from_value(value).unwrap_or_default();
        CredentialDefinitionWrapper(Rc::new(CredentialDefinition {
            issuer_id: DID::from(issuer_id.as_str()),
            schema_id: SchemaId::from(schema_id.as_str()),
            cred_def_type: SignatureType::CL,
            tag,
            value,
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
    pub fn from_string(string: &str) -> Result<CredentialDefinitionWrapper> {
        CredentialDefinition::from_string(string)
            .as_js()
            .map(CredentialDefinitionWrapper::from)
            .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = asValue)]
    pub fn as_value(&self) -> Result<JsValue> {
        serde_wasm_bindgen::to_value(&*self.0).map_err(JsValue::from)
    }
}

impl From<CredentialDefinition> for CredentialDefinitionWrapper {
    fn from(data: CredentialDefinition) -> CredentialDefinitionWrapper {
        CredentialDefinitionWrapper(Rc::new(data))
    }
}
