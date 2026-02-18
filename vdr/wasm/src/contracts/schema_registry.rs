// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use indy_besu_vdr::{schema_registry, Address, Schema, SchemaId, DID};
use std::rc::Rc;
use wasm_bindgen::prelude::*;

use crate::{
    client::LedgerClientWrapper,
    error::{JsResult, Result},
    transaction::{TransactionEndorsingDataWrapper, TransactionWrapper},
};

#[wasm_bindgen(js_name = SchemaRegistry)]
pub struct SchemaRegistry;

#[wasm_bindgen(js_class = SchemaRegistry)]
impl SchemaRegistry {
    #[wasm_bindgen(js_name = buildCreateSchemaTransaction)]
    pub async fn build_create_schema_transaction(
        client: &LedgerClientWrapper,
        from: &str,
        schema: &SchemaWrapper,
    ) -> Result<TransactionWrapper> {
        let address = Address::from(from);
        schema_registry::build_create_schema_transaction(&client.0, &address, &schema.0)
            .await
            .as_js()
            .map(TransactionWrapper::from)
            .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = buildCreateSchemaEndorsingData)]
    pub async fn build_create_schema_endorsing_data(
        client: &LedgerClientWrapper,
        schema: &SchemaWrapper,
    ) -> Result<TransactionEndorsingDataWrapper> {
        schema_registry::build_create_schema_endorsing_data(&client.0, &schema.0)
            .await
            .as_js()
            .map(TransactionEndorsingDataWrapper::from)
            .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = buildResolveSchemaTransaction)]
    pub async fn build_resolve_schema_transaction(
        client: &LedgerClientWrapper,
        id: &str,
    ) -> Result<TransactionWrapper> {
        let id = SchemaId::from(id);
        schema_registry::build_resolve_schema_transaction(&client.0, &id)
            .await
            .as_js()
            .map(TransactionWrapper::from)
            .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = parseResolveSchemaResult)]
    pub fn parse_resolve_schema_result(
        client: &LedgerClientWrapper,
        bytes: Vec<u8>,
    ) -> Result<JsValue> {
        let schema = schema_registry::parse_resolve_schema_result(&client.0, &bytes).as_js()?;
        let result: JsValue = serde_wasm_bindgen::to_value(&schema)?;
        Ok(result)
    }

    #[wasm_bindgen(js_name = resolveSchema)]
    pub async fn resolve_schema(client: &LedgerClientWrapper, id: &str) -> Result<SchemaWrapper> {
        let id = SchemaId::from(id);
        schema_registry::resolve_schema(&client.0, &id)
            .await
            .as_js()
            .map(SchemaWrapper::from)
            .map_err(JsValue::from)
    }
}

#[wasm_bindgen(js_name = Schema)]
pub struct SchemaWrapper(pub(crate) Rc<Schema>);

#[wasm_bindgen(js_class = Schema)]
impl SchemaWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new(
        issuer_id: String,
        name: String,
        version: String,
        attr_names: Vec<String>,
    ) -> SchemaWrapper {
        SchemaWrapper(Rc::new(Schema {
            issuer_id: DID::from(issuer_id.as_str()),
            name,
            version,
            attr_names: attr_names.iter().cloned().collect(),
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
    pub fn from_string(string: &str) -> Result<SchemaWrapper> {
        Schema::from_string(string)
            .as_js()
            .map(SchemaWrapper::from)
            .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = asValue)]
    pub fn as_value(&self) -> Result<JsValue> {
        serde_wasm_bindgen::to_value(&*self.0).map_err(JsValue::from)
    }
}

impl From<Schema> for SchemaWrapper {
    fn from(data: Schema) -> SchemaWrapper {
        SchemaWrapper(Rc::new(data))
    }
}
