// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use indy_besu_vdr::VdrError as VdrError_;

#[derive(thiserror::Error, Debug, uniffi::Error)]
pub enum VdrError {
    #[error("Ledger Client: Node is unreachable")]
    ClientNodeUnreachable,

    #[error("Ledger Client: Invalid transaction: {}", msg)]
    ClientInvalidTransaction { msg: String },

    #[error("Ledger Client: Invalid endorsement data: {}", msg)]
    ClientInvalidEndorsementData { msg: String },

    #[error("Ledger Client: Got invalid response: {}", msg)]
    ClientInvalidResponse { msg: String },

    #[error("Ledger Client: Transaction reverted: {}", msg)]
    ClientTransactionReverted { msg: String },

    #[error("Ledger Client: Unexpected error occurred: {}", msg)]
    ClientUnexpectedError { msg: String },

    #[error("Ledger Client: Invalid state {}", msg)]
    ClientInvalidState { msg: String },

    #[error("Contract: Invalid name: {}", msg)]
    ContractInvalidName { msg: String },

    #[error("Contract: Invalid specification: {}", msg)]
    ContractInvalidSpec { msg: String },

    #[error("Contract: Invalid data")]
    ContractInvalidInputData,

    #[error("Contract: Invalid response data: {}", msg)]
    ContractInvalidResponseData { msg: String },

    #[error("Signer: Invalid private key")]
    SignerInvalidPrivateKey,

    #[error("Signer: Invalid message")]
    SignerInvalidMessage,

    #[error("Signer: Key is missing: {}", msg)]
    SignerMissingKey { msg: String },

    #[error("Signer: Unexpected error occurred: {}", msg)]
    SignerUnexpectedError { msg: String },

    #[error("Invalid data: {}", msg)]
    CommonInvalidData { msg: String },

    #[error("Ledger: Quorum not reached: {}", msg)]
    QuorumNotReached { msg: String },

    #[error("Could not get transaction: {}", msg)]
    GetTransactionError { msg: String },

    #[error("Invalid schema: {}", msg)]
    InvalidSchema { msg: String },

    #[error("Invalid credential definition: {}", msg)]
    InvalidCredentialDefinition { msg: String },

    #[error("Invalid DID document: {}", msg)]
    InvalidDidDocument { msg: String },

    #[error("Invalid Revocation Registry Definition: {}", msg)]
    InvalidRevocationRegistryDefinition { msg: String },

    #[error("Invalid Revocation Registry Entry: {}", msg)]
    InvalidRevocationRegistryEntry { msg: String },

    #[error("Invalid Revocation Registry Status List: {}", msg)]
    InvalidRevocationRegistryStatusList { msg: String },
}

pub type VdrResult<T> = Result<T, VdrError>;

impl From<VdrError_> for VdrError {
    fn from(error: VdrError_) -> Self {
        match error {
            VdrError_::ClientNodeUnreachable => VdrError::ClientNodeUnreachable,
            VdrError_::ClientInvalidTransaction(msg) => VdrError::ClientInvalidTransaction { msg },
            VdrError_::ClientInvalidEndorsementData(msg) => {
                VdrError::ClientInvalidEndorsementData { msg }
            }
            VdrError_::ClientInvalidResponse(msg) => VdrError::ClientInvalidResponse { msg },
            VdrError_::ClientTransactionReverted(msg) => {
                VdrError::ClientTransactionReverted { msg }
            }
            VdrError_::ClientUnexpectedError(msg) => VdrError::ClientUnexpectedError { msg },
            VdrError_::ClientInvalidState(msg) => VdrError::ClientInvalidState { msg },
            VdrError_::ContractInvalidName(msg) => VdrError::ContractInvalidName { msg },
            VdrError_::ContractInvalidSpec(msg) => VdrError::ContractInvalidSpec { msg },
            VdrError_::ContractInvalidInputData => VdrError::ContractInvalidInputData,
            VdrError_::ContractInvalidResponseData(msg) => {
                VdrError::ContractInvalidResponseData { msg }
            }
            VdrError_::SignerInvalidPrivateKey => VdrError::SignerInvalidPrivateKey,
            VdrError_::SignerInvalidMessage => VdrError::SignerInvalidMessage,
            VdrError_::SignerMissingKey(msg) => VdrError::SignerMissingKey { msg },
            VdrError_::SignerUnexpectedError(msg) => VdrError::SignerUnexpectedError { msg },
            VdrError_::CommonInvalidData(msg) => VdrError::CommonInvalidData { msg },
            VdrError_::QuorumNotReached(msg) => VdrError::QuorumNotReached { msg },
            VdrError_::GetTransactionError(msg) => VdrError::GetTransactionError { msg },
            VdrError_::InvalidSchema(msg) => VdrError::InvalidSchema { msg },
            VdrError_::InvalidCredentialDefinition(msg) => {
                VdrError::InvalidCredentialDefinition { msg }
            }
            VdrError_::InvalidDidDocument(msg) => VdrError::InvalidDidDocument { msg },
            VdrError_::InvalidRevocationRegistryDefinition(msg) => {
                VdrError::InvalidRevocationRegistryDefinition { msg }
            }
            VdrError_::InvalidRevocationRegistryEntry(msg) => {
                VdrError::InvalidRevocationRegistryEntry { msg }
            }
            VdrError_::InvalidRevocationRegistryStatusList(msg) => {
                VdrError::InvalidRevocationRegistryStatusList { msg }
            }
        }
    }
}
