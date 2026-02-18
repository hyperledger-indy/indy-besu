// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use std::ops::RangeInclusive;

use serde_json::json;

use jsonrpc_core::types::error::{Error as RpcError, ErrorCode};
#[cfg(not(feature = "wasm"))]
use web3::{ethabi::Error as Web3EthabiError, Error as Web3Error};
#[cfg(feature = "wasm")]
use web3_wasm::{ethabi::Error as Web3EthabiError, Error as Web3Error};

const RPC_SERVER_ERROR_RANGE: RangeInclusive<i64> = -32099..=-32000;

#[derive(thiserror::Error, Debug, PartialEq, Clone)]
pub enum VdrError {
    #[error("Ledger: Quorum not reached: {}", _0)]
    QuorumNotReached(String),

    #[error("Ledger Client: Node is unreachable")]
    ClientNodeUnreachable,

    #[error("Ledger Client: Invalid transaction: {}", _0)]
    ClientInvalidTransaction(String),

    #[error("Ledger Client: Invalid endorsement data: {}", _0)]
    ClientInvalidEndorsementData(String),

    #[error("Ledger Client: Got invalid response: {}", _0)]
    ClientInvalidResponse(String),

    #[error("Ledger Client: Transaction reverted: {}", _0)]
    ClientTransactionReverted(String),

    #[error("Ledger Client: Unexpected error occurred: {}", _0)]
    ClientUnexpectedError(String),

    #[error("Ledger Client: Invalid state {}", _0)]
    ClientInvalidState(String),

    #[error("Contract: Invalid name: {}", _0)]
    ContractInvalidName(String),

    #[error("Contract: Invalid specification: {}", _0)]
    ContractInvalidSpec(String),

    #[error("Contract: Invalid data")]
    ContractInvalidInputData,

    #[error("Contract: Invalid response data: {}", _0)]
    ContractInvalidResponseData(String),

    #[error("Signer: Invalid private key")]
    SignerInvalidPrivateKey,

    #[error("Signer: Invalid message")]
    SignerInvalidMessage,

    #[error("Signer: Key is missing: {}", _0)]
    SignerMissingKey(String),

    #[error("Signer: Unexpected error occurred: {}", _0)]
    SignerUnexpectedError(String),

    #[error("Invalid data: {}", _0)]
    CommonInvalidData(String),

    #[error("Could not get transaction: {}", _0)]
    GetTransactionError(String),

    #[error("Invalid DID document: {}", _0)]
    InvalidDidDocument(String),

    #[error("Invalid schema: {}", _0)]
    InvalidSchema(String),

    #[error("Invalid credential definition: {}", _0)]
    InvalidCredentialDefinition(String),

    #[error("Invalid revocation registry definition: {}", _0)]
    InvalidRevocationRegistryDefinition(String),

    #[error("Invalid revocation registry entry: {}", _0)]
    InvalidRevocationRegistryEntry(String),

    #[error("Invalid revocation status list: {}", _0)]
    InvalidRevocationRegistryStatusList(String),
}

pub type VdrResult<T> = Result<T, VdrError>;

impl From<Web3Error> for VdrError {
    fn from(value: Web3Error) -> Self {
        match value {
            Web3Error::Unreachable => VdrError::ClientNodeUnreachable,
            Web3Error::InvalidResponse(err) => VdrError::ClientInvalidResponse(err),
            Web3Error::Rpc(err) => err.into(),
            _ => VdrError::ClientUnexpectedError(value.to_string()),
        }
    }
}

impl From<RpcError> for VdrError {
    fn from(value: RpcError) -> Self {
        let create_unexpected_error = || VdrError::ClientUnexpectedError(json!(value).to_string());

        match value.code {
            ErrorCode::ServerError(code) if RPC_SERVER_ERROR_RANGE.contains(&code) => value
                .data
                .as_ref()
                .and_then(|data| data.as_str())
                .map_or_else(create_unexpected_error, |data| {
                    if data.starts_with("0x") {
                        VdrError::ClientTransactionReverted(data.to_string())
                    } else {
                        create_unexpected_error()
                    }
                }),
            _ => create_unexpected_error(),
        }
    }
}

impl From<Web3EthabiError> for VdrError {
    fn from(value: Web3EthabiError) -> Self {
        match value {
            Web3EthabiError::InvalidName(name) => VdrError::ContractInvalidName(name),
            _ => VdrError::ContractInvalidInputData,
        }
    }
}

#[cfg(feature = "basic_signer")]
impl From<secp256k1::Error> for VdrError {
    fn from(value: secp256k1::Error) -> Self {
        match value {
            secp256k1::Error::InvalidSecretKey => VdrError::SignerInvalidPrivateKey,
            secp256k1::Error::InvalidMessage => VdrError::SignerInvalidMessage,
            err => VdrError::SignerUnexpectedError(err.to_string()),
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use jsonrpc_core::{
        types::error::{Error as RpcError, ErrorCode},
        Value,
    };
    use rstest::rstest;

    #[rstest]
    #[case::rpc_error_with_hex_string_data(
        RpcError { code:  ErrorCode::ServerError(-32000), message: "transaction reverted".to_string(), data: Option::Some(Value::String("0x4e487b710000000000000000000000000000000000000000000000000000000000000011".to_string()))}, 
        VdrError::ClientTransactionReverted("0x4e487b710000000000000000000000000000000000000000000000000000000000000011".to_string()),
    )]
    #[case::rpc_error_without_data(
        RpcError { code:  ErrorCode::ServerError(-32000), message: "transaction reverted".to_string(), data: Option::None}, 
        VdrError::ClientUnexpectedError("{\"code\":-32000,\"message\":\"transaction reverted\"}".to_string()),
    )]
    #[case::rpc_error_with_text_data(
        RpcError { code:  ErrorCode::ServerError(-32000), message: "transaction reverted".to_string(), data: Option::Some(Value::String("Error(message: Not enough Ether provided.)".to_string()))}, 
        VdrError::ClientUnexpectedError("{\"code\":-32000,\"data\":\"Error(message: Not enough Ether provided.)\",\"message\":\"transaction reverted\"}".to_string()),
    )]
    #[case::rpc_error_with_boolean_data(
        RpcError { code:  ErrorCode::ServerError(-32000), message: "Invalid request".to_string(), data: Option::Some(Value::Bool(true))}, 
        VdrError::ClientUnexpectedError("{\"code\":-32000,\"data\":true,\"message\":\"Invalid request\"}".to_string()),
    )]
    #[case::rpc_error_with_invalid_request_code(
        RpcError { code:  ErrorCode::InvalidRequest, message: "Invalid request".to_string(), data: Option::None}, 
        VdrError::ClientUnexpectedError("{\"code\":-32600,\"message\":\"Invalid request\"}".to_string()),
    )]
    fn convert_rpc_error_to_vdr_error_test(
        #[case] rpc_error: RpcError,
        #[case] expected_vdr_error: VdrError,
    ) {
        let actual_vdr_error: VdrError = rpc_error.into();

        assert_eq!(actual_vdr_error, expected_vdr_error);
    }
}
