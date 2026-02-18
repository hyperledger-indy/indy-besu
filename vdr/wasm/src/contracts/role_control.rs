// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use indy_besu_vdr::{role_control, Address, Role};
use wasm_bindgen::prelude::*;

use crate::{
    client::LedgerClientWrapper,
    error::{JsResult, Result},
    transaction::TransactionWrapper,
};

#[wasm_bindgen(js_name = RoleControl)]
pub struct RoleControl;

#[wasm_bindgen(js_class = RoleControl)]
impl RoleControl {
    #[wasm_bindgen(js_name = buildAssignRoleTransaction)]
    pub async fn build_assign_role_transaction(
        client: &LedgerClientWrapper,
        from: &str,
        role: u8,
        account: &str,
    ) -> Result<TransactionWrapper> {
        let role = Role::try_from(role).as_js()?;
        let from = Address::from(from);
        let account = Address::from(account);
        role_control::build_assign_role_transaction(&client.0, &from, &role, &account)
            .await
            .as_js()
            .map(TransactionWrapper::from)
            .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = buildRevokeRoleTransaction)]
    pub async fn build_revoke_role_transaction(
        client: &LedgerClientWrapper,
        from: &str,
        role: u8,
        account: &str,
    ) -> Result<TransactionWrapper> {
        let role = Role::try_from(role).as_js()?;
        let from = Address::from(from);
        let account = Address::from(account);
        role_control::build_revoke_role_transaction(&client.0, &from, &role, &account)
            .await
            .as_js()
            .map(TransactionWrapper::from)
            .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = buildHasRoleTransaction)]
    pub async fn build_has_role_transaction(
        client: &LedgerClientWrapper,
        role: u8,
        account: &str,
    ) -> Result<TransactionWrapper> {
        let role = Role::try_from(role).as_js()?;
        let account = Address::from(account);
        role_control::build_has_role_transaction(&client.0, &role, &account)
            .await
            .as_js()
            .map(TransactionWrapper::from)
            .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = buildGetRoleTransaction)]
    pub async fn build_get_role_transaction(
        client: &LedgerClientWrapper,
        account: &str,
    ) -> Result<TransactionWrapper> {
        let account = Address::from(account);
        role_control::build_get_role_transaction(&client.0, &account)
            .await
            .as_js()
            .map(TransactionWrapper::from)
            .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = parseHasRoleResult)]
    pub fn parse_has_role_result(client: &LedgerClientWrapper, bytes: Vec<u8>) -> Result<bool> {
        let has_role = role_control::parse_has_role_result(&client.0, &bytes).as_js()?;
        Ok(has_role)
    }

    #[wasm_bindgen(js_name = parseGetRoleResult)]
    pub fn parse_get_role_result(client: &LedgerClientWrapper, bytes: Vec<u8>) -> Result<u8> {
        let role = role_control::parse_get_role_result(&client.0, &bytes).as_js()?;
        Ok(role.into())
    }
}
