// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::VdrError,
    types::{ContractOutput, ContractParam},
};

/// Enum listing roles defined on the ledger
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Role {
    Empty = 0,
    Trustee = 1,
    Endorser = 2,
    Steward = 3,
}

pub type HasRole = bool;
pub type RoleIndex = u8;

impl TryFrom<&Role> for ContractParam {
    type Error = VdrError;

    fn try_from(value: &Role) -> Result<Self, Self::Error> {
        let role_index: RoleIndex = (*value).into();
        Ok(ContractParam::Uint(role_index.into()))
    }
}

impl TryFrom<ContractOutput> for Role {
    type Error = VdrError;

    fn try_from(value: ContractOutput) -> Result<Self, Self::Error> {
        let role_index = value.get_u8(0)?;
        Role::try_from(role_index)
    }
}

impl From<Role> for RoleIndex {
    fn from(value: Role) -> Self {
        value as u8
    }
}

impl TryFrom<RoleIndex> for Role {
    type Error = VdrError;

    fn try_from(index: RoleIndex) -> Result<Self, Self::Error> {
        match index {
            0 => Ok(Role::Empty),
            1 => Ok(Role::Trustee),
            2 => Ok(Role::Endorser),
            3 => Ok(Role::Steward),
            _ => Err(VdrError::ContractInvalidResponseData(
                "Invalid role provided".to_string(),
            )),
        }
    }
}

impl TryFrom<ContractOutput> for HasRole {
    type Error = VdrError;

    fn try_from(value: ContractOutput) -> Result<Self, Self::Error> {
        value.get_bool(0)
    }
}
