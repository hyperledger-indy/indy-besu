// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::JsonValue;
use indy_besu_vdr::{
    ContractConfig as ContractConfig_, ContractSpec as ContractSpec_, PingStatus as PingStatus_,
    QuorumConfig as QuorumConfig_, SignatureData as SignatureData_, Status as Status_,
    TransactionType as TransactionType_,
};

#[derive(uniffi::Record)]
pub struct PingStatus {
    pub status: Status,
}

#[derive(uniffi::Enum)]
pub enum Status {
    Ok {
        block_number: u64,
        block_timestamp: u64,
    },
    Err {
        msg: String,
    },
}

#[derive(uniffi::Record)]
pub struct ContractConfig {
    pub address: String,
    pub spec_path: Option<String>,
    pub spec: Option<ContractSpec>,
}

#[derive(uniffi::Record)]
pub struct ContractSpec {
    pub name: String,
    pub abi: JsonValue,
}

#[derive(uniffi::Enum)]
pub enum TransactionType {
    Read,
    Write,
}

#[derive(uniffi::Record)]
pub struct SignatureData {
    pub recovery_id: u64,
    pub signature: Vec<u8>,
}

#[derive(uniffi::Record)]
pub struct QuorumConfig {
    pub nodes: Vec<String>,
    pub request_retries: Option<u8>,
    pub request_timeout: Option<u64>,
    pub retry_interval: Option<u64>,
}

impl From<PingStatus_> for PingStatus {
    fn from(status: PingStatus_) -> Self {
        PingStatus {
            status: Status::from(status.status),
        }
    }
}

impl From<Status_> for Status {
    fn from(status: Status_) -> Self {
        match status {
            Status_::Ok {
                block_number,
                block_timestamp,
            } => Status::Ok {
                block_number,
                block_timestamp,
            },
            Status_::Err { msg } => Status::Err { msg },
        }
    }
}

impl Into<ContractConfig_> for ContractConfig {
    fn into(self) -> ContractConfig_ {
        ContractConfig_ {
            address: self.address,
            spec_path: self.spec_path,
            spec: self.spec.map(ContractSpec::into),
        }
    }
}

impl Into<ContractSpec_> for ContractSpec {
    fn into(self) -> ContractSpec_ {
        ContractSpec_ {
            name: self.name,
            abi: self.abi.into_inner(),
        }
    }
}

impl Into<TransactionType_> for &TransactionType {
    fn into(self) -> TransactionType_ {
        match self {
            TransactionType::Read => TransactionType_::Read,
            TransactionType::Write => TransactionType_::Write,
        }
    }
}

impl From<&TransactionType_> for TransactionType {
    fn from(type_: &TransactionType_) -> TransactionType {
        match type_ {
            TransactionType_::Read => TransactionType::Read,
            TransactionType_::Write => TransactionType::Write,
        }
    }
}

impl Into<SignatureData_> for &SignatureData {
    fn into(self) -> SignatureData_ {
        SignatureData_ {
            recovery_id: self.recovery_id.to_owned(),
            signature: self.signature.to_owned(),
        }
    }
}

impl From<&SignatureData_> for SignatureData {
    fn from(signature: &SignatureData_) -> SignatureData {
        SignatureData {
            recovery_id: signature.recovery_id,
            signature: signature.signature.clone(),
        }
    }
}

impl Into<QuorumConfig_> for QuorumConfig {
    fn into(self) -> QuorumConfig_ {
        QuorumConfig_ {
            nodes: self.nodes,
            request_retries: self.request_retries,
            request_timeout: self.request_timeout,
            retry_interval: self.retry_interval,
        }
    }
}
