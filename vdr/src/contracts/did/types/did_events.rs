// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{
    types::{Block, ContractEvent},
    utils::parse_bytes32_string,
    Address, VdrError,
};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum DidEvents {
    AttributeChangedEvent(DidAttributeChanged),
    DelegateChanged(DidDelegateChanged),
    OwnerChanged(DidOwnerChanged),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DidAttributeChanged {
    pub identity: Address,
    pub name: String,
    pub value: Vec<u8>,
    pub valid_to: u64,
    pub previous_change: Block,
}

impl DidAttributeChanged {
    pub(crate) fn key(&self) -> String {
        format!("DidDocAttribute-{}-{:?}", self.name, self.value)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DidDelegateChanged {
    pub identity: Address,
    pub delegate: Address,
    pub delegate_type: Vec<u8>,
    pub valid_to: u64,
    pub previous_change: Block,
}

impl DidDelegateChanged {
    pub(crate) fn key(&self) -> String {
        format!(
            "DelegateChanged-{:?}-{}",
            self.delegate_type,
            self.delegate.as_ref()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DidOwnerChanged {
    pub identity: Address,
    pub owner: Address,
    pub previous_change: Block,
}

impl DidOwnerChanged {
    #[allow(unused)]
    pub(crate) fn key(&self) -> String {
        format!("DidOwnerChanged-{}", self.owner.as_ref())
    }
}

impl TryFrom<ContractEvent> for DidAttributeChanged {
    type Error = VdrError;

    fn try_from(log: ContractEvent) -> Result<Self, Self::Error> {
        let identity = log.get_address(0)?;
        let name = log.get_fixed_bytes(1)?;
        let value = log.get_bytes(2)?;
        let valid_to = log.get_uint(3)?;
        let previous_change = Block::from(log.get_uint(4)?);

        let name = parse_bytes32_string(name.as_slice())?.to_string();

        Ok(DidAttributeChanged {
            identity,
            name,
            value,
            valid_to,
            previous_change,
        })
    }
}

impl TryFrom<ContractEvent> for DidDelegateChanged {
    type Error = VdrError;

    fn try_from(value: ContractEvent) -> Result<Self, Self::Error> {
        let identity = value.get_address(0)?;
        let delegate = value.get_address(1)?;
        let delegate_type = value.get_fixed_bytes(2)?;
        let valid_to = value.get_uint(3)?;
        let previous_change = Block::from(value.get_uint(4)?);

        Ok(DidDelegateChanged {
            identity,
            delegate,
            delegate_type,
            valid_to,
            previous_change,
        })
    }
}

impl TryFrom<ContractEvent> for DidOwnerChanged {
    type Error = VdrError;

    fn try_from(value: ContractEvent) -> Result<Self, Self::Error> {
        let identity = value.get_address(0)?;
        let owner = value.get_address(1)?;
        let previous_change = Block::from(value.get_uint(2)?);

        Ok(DidOwnerChanged {
            identity,
            owner,
            previous_change,
        })
    }
}

impl DidEvents {
    pub fn previous_change(&self) -> Block {
        match self {
            DidEvents::AttributeChangedEvent(event) => event.previous_change.clone(),
            DidEvents::DelegateChanged(event) => event.previous_change.clone(),
            DidEvents::OwnerChanged(event) => event.previous_change.clone(),
        }
    }
}
