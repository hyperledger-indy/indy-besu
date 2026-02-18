// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{
    client::LedgerClient,
    contracts::auth::{HasRole, Role},
    error::VdrResult,
    types::{Address, Transaction, TransactionBuilder, TransactionParser, TransactionType},
};
use log_derive::{logfn, logfn_inputs};

const CONTRACT_NAME: &str = "RoleControl";
const METHOD_ASSIGN_ROLE: &str = "assignRole";
const METHOD_REVOKE_ROLE: &str = "revokeRole";
const METHOD_HAS_ROLE: &str = "hasRole";
const METHOD_GET_ROLE: &str = "getRole";

/// Build transaction to execute RoleControl.assignRole contract method to assign a role to an account
///
/// # Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `from`: [Address] - transaction sender account address
/// - `role`: [Role] - role to assign
/// - `account`: [Address] - assignee account
///
/// # Returns
///   transaction: [Transaction] - prepared write transaction object to sign and submit
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_assign_role_transaction(
    client: &LedgerClient,
    from: &Address,
    role: &Role,
    account: &Address,
) -> VdrResult<Transaction> {
    TransactionBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_ASSIGN_ROLE)
        .add_param(role)?
        .add_param(account)?
        .set_type(TransactionType::Write)
        .set_from(from)
        .build(client)
        .await
}

/// Build transaction to execute RoleControl.revokeRole contract method to revoke a role from an account
///
/// # Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `from`: [Address] - transaction sender account address
/// - `role`: [Role] - role to assign
/// - `account`: [Address] - revoke account
///
/// # Returns
///   transaction: [Transaction] - prepared write transaction object to sign and submit
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_revoke_role_transaction(
    client: &LedgerClient,
    from: &Address,
    role: &Role,
    account: &Address,
) -> VdrResult<Transaction> {
    TransactionBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_REVOKE_ROLE)
        .add_param(role)?
        .add_param(account)?
        .set_type(TransactionType::Write)
        .set_from(from)
        .build(client)
        .await
}

/// Build transaction to execute RoleControl.hasRole contract method to check an account has a role
///
/// # Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `role`: [Role] - role to check
/// - `account`: [Address] - account to check
///
/// # Returns
///   transaction: [Transaction] - prepared read transaction object to submit
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_has_role_transaction(
    client: &LedgerClient,
    role: &Role,
    account: &Address,
) -> VdrResult<Transaction> {
    TransactionBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_HAS_ROLE)
        .add_param(role)?
        .add_param(account)?
        .set_type(TransactionType::Read)
        .build(client)
        .await
}

/// Build transaction to execute RoleControl.getRole contract method to get account's role
///
/// # Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `account`: [Address] - account address
///
/// # Returns
///   transaction: [Transaction] - prepared read transaction object to submit
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_get_role_transaction(
    client: &LedgerClient,
    account: &Address,
) -> VdrResult<Transaction> {
    TransactionBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_GET_ROLE)
        .add_param(account)?
        .set_type(TransactionType::Read)
        .build(client)
        .await
}

/// Parse the result of execution RoleControl.HasRole contract method to check an account has a role
///
/// # Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `bytes`: [Vec] - result bytes returned from the ledger
///
/// # Returns
/// Account has role result
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub fn parse_has_role_result(client: &LedgerClient, bytes: &[u8]) -> VdrResult<bool> {
    TransactionParser::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_HAS_ROLE)
        .parse::<HasRole>(client, bytes)
}

/// Parse the result of execution RoleControl.GetRole contract method to get account's role
///
/// # Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `bytes`: [Vec] - result bytes returned from the ledger
///
/// # Returns
/// [Role] Account's role
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub fn parse_get_role_result(client: &LedgerClient, bytes: &[u8]) -> VdrResult<Role> {
    TransactionParser::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_GET_ROLE)
        .parse::<Role>(client, bytes)
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::client::client::test::{
        mock_client, CONFIG, DEFAULT_NONCE, TEST_ACCOUNT, TRUSTEE_ACCOUNT,
    };

    pub static ACCOUNT_ROLES: [Role; 4] =
        [Role::Empty, Role::Trustee, Role::Steward, Role::Endorser];

    mod build_assign_role_transaction {
        use super::*;

        #[async_std::test]
        async fn build_assign_role_transaction_test() {
            let client = mock_client();
            let expected_data = vec![
                136, 165, 191, 110, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 240, 226, 219,
                108, 141, 198, 198, 129, 187, 93, 106, 209, 33, 161, 7, 243, 0, 233, 178, 181,
            ];

            let transaction = build_assign_role_transaction(
                &client,
                &TRUSTEE_ACCOUNT,
                &Role::Trustee,
                &TEST_ACCOUNT,
            )
            .await
            .unwrap();

            let expected_transaction = Transaction {
                type_: TransactionType::Write,
                from: Some(TRUSTEE_ACCOUNT.clone()),
                to: CONFIG.contracts.role_control.address.clone(),
                nonce: Some(DEFAULT_NONCE.clone()),
                chain_id: CONFIG.chain_id,
                data: expected_data,
                signature: None,
                hash: None,
            };

            assert_eq!(expected_transaction, transaction);
        }
    }

    mod build_revoke_role_transaction {
        use super::*;

        #[async_std::test]
        async fn build_revoke_role_transaction_test() {
            let client = mock_client();
            let expected_data = vec![
                76, 187, 135, 211, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 240, 226, 219,
                108, 141, 198, 198, 129, 187, 93, 106, 209, 33, 161, 7, 243, 0, 233, 178, 181,
            ];

            let transaction = build_revoke_role_transaction(
                &client,
                &TRUSTEE_ACCOUNT,
                &Role::Trustee,
                &TEST_ACCOUNT,
            )
            .await
            .unwrap();

            let expected_transaction = Transaction {
                type_: TransactionType::Write,
                from: Some(TRUSTEE_ACCOUNT.clone()),
                to: CONFIG.contracts.role_control.address.clone(),
                nonce: Some(DEFAULT_NONCE.clone()),
                chain_id: CONFIG.chain_id,
                data: expected_data,
                signature: None,
                hash: None,
            };

            assert_eq!(expected_transaction, transaction);
        }
    }

    mod build_get_role_transaction {
        use super::*;

        #[async_std::test]
        async fn build_get_role_transaction_test() {
            let client = mock_client();
            let expected_data = vec![
                68, 39, 103, 51, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 240, 226, 219, 108, 141, 198,
                198, 129, 187, 93, 106, 209, 33, 161, 7, 243, 0, 233, 178, 181,
            ];

            let transaction = build_get_role_transaction(&client, &TEST_ACCOUNT)
                .await
                .unwrap();

            let expected_transaction = Transaction {
                type_: TransactionType::Read,
                from: None,
                to: CONFIG.contracts.role_control.address.clone(),
                nonce: None,
                chain_id: CONFIG.chain_id,
                data: expected_data,
                signature: None,
                hash: None,
            };

            assert_eq!(expected_transaction, transaction);
        }
    }

    mod parse_get_role_result {
        use super::*;

        #[test]
        fn parse_get_role_result_test() {
            let client = mock_client();
            let result = vec![0; 32];
            let expected_role = Role::Empty;

            let role = parse_get_role_result(&client, &result).unwrap();

            assert_eq!(expected_role, role);
        }
    }

    mod build_has_role_transaction {
        use super::*;

        #[async_std::test]
        async fn build_has_role_transaction_test() {
            let client = mock_client();
            let expected_data = vec![
                158, 151, 184, 246, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 240, 226, 219,
                108, 141, 198, 198, 129, 187, 93, 106, 209, 33, 161, 7, 243, 0, 233, 178, 181,
            ];

            let transaction = build_has_role_transaction(&client, &Role::Trustee, &TEST_ACCOUNT)
                .await
                .unwrap();

            let expected_transaction = Transaction {
                type_: TransactionType::Read,
                from: None,
                to: CONFIG.contracts.role_control.address.clone(),
                nonce: None,
                chain_id: CONFIG.chain_id,
                data: expected_data,
                signature: None,
                hash: None,
            };

            assert_eq!(expected_transaction, transaction);
        }
    }

    mod parse_has_role_result {
        use super::*;

        #[test]
        fn parse_has_role_result_test() {
            let client = mock_client();
            let result = vec![0; 32];
            let expected_has_role = false;

            let has_role = parse_has_role_result(&client, &result).unwrap();

            assert_eq!(expected_has_role, has_role);
        }
    }
}
