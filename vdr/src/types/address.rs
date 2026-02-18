// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::VdrError,
    types::{ContractOutput, ContractParam},
    DID,
};

use crate::contracts::types::did::ParsedDid;
use ethereum_types::Address as Address_;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

const PREFIX: &str = "0x";
const NULL_ADDRESS: &str = "0x0000000000000000000000000000000000000000";

/// Wrapper structure for Ethereum account address
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Address(String);

impl Address {
    pub(crate) fn to_filter(&self) -> String {
        format!("000000000000000000000000{}", &self.0[2..])
    }

    pub fn as_blockchain_id(&self, chain_id: u64) -> String {
        format!("eip155:{}:{}", chain_id, self.as_ref())
    }

    pub fn null() -> Address {
        Address::from(NULL_ADDRESS)
    }

    pub fn is_null(&self) -> bool {
        self.as_ref() == NULL_ADDRESS
    }
}

impl From<&str> for Address {
    fn from(address: &str) -> Self {
        if address.starts_with(PREFIX) {
            Address(address.to_string())
        } else {
            Address(format!("{}{}", PREFIX, address))
        }
    }
}

impl AsRef<str> for Address {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl ToString for Address {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl TryInto<ContractParam> for &Address {
    type Error = VdrError;

    fn try_into(self) -> Result<ContractParam, Self::Error> {
        let acc_address = Address_::from_str(self.as_ref()).map_err(|err| {
            VdrError::CommonInvalidData(format!(
                "Unable to parse account address. Err: {:?}",
                err.to_string()
            ))
        })?;
        Ok(ContractParam::Address(acc_address))
    }
}

impl TryFrom<ContractOutput> for Address {
    type Error = VdrError;

    fn try_from(value: ContractOutput) -> Result<Self, Self::Error> {
        value.get_address(0)
    }
}

impl TryFrom<&DID> for Address {
    type Error = VdrError;

    fn try_from(did: &DID) -> Result<Self, Self::Error> {
        let parsed_did = ParsedDid::try_from(did)?;
        Ok(Address::from(parsed_did.identifier.as_str()))
    }
}
