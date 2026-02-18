// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{
    types::{ContractOutput, ContractParam},
    SchemaId, VdrError,
};
use serde::{Deserialize, Serialize};

/// Wrapper structure for resource identifier (Schema Id / Credential Definition Id)
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceIdentifier(String);

impl ResourceIdentifier {
    pub fn value(&self) -> &str {
        self.0.as_str()
    }
}

impl AsRef<str> for ResourceIdentifier {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl ToString for ResourceIdentifier {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl From<&str> for ResourceIdentifier {
    fn from(value: &str) -> Self {
        ResourceIdentifier(value.to_string())
    }
}

impl From<&SchemaId> for ResourceIdentifier {
    fn from(value: &SchemaId) -> Self {
        ResourceIdentifier::from(value.as_ref())
    }
}

impl From<&indy_data_types::SchemaId> for ResourceIdentifier {
    fn from(value: &indy_data_types::SchemaId) -> Self {
        ResourceIdentifier::from(value.0.as_str())
    }
}

impl TryFrom<&ResourceIdentifier> for ContractParam {
    type Error = VdrError;

    fn try_from(value: &ResourceIdentifier) -> Result<Self, Self::Error> {
        Ok(ContractParam::String(value.0.to_string()))
    }
}

impl TryFrom<ContractOutput> for ResourceIdentifier {
    type Error = VdrError;

    fn try_from(value: ContractOutput) -> Result<Self, Self::Error> {
        let string = value.get_string(0)?;
        Ok(ResourceIdentifier(string))
    }
}
