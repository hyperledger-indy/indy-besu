use serde::{Deserialize, Serialize};

use crate::{RevocationRegistryDefinitionId, VdrError, VdrResult, DID};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevocationRegistryDelta {
    pub revoked: Vec<u32>,
    pub issued: Vec<u32>,
    pub accum: String,
    pub timestamp: u64,
}

impl RevocationRegistryDelta {
    pub fn validate(&self, limit_idx: u32) -> VdrResult<()> {
        let highest_index = self.issued.iter().chain(self.revoked.iter()).max();

        if highest_index.is_some_and(|&highest| highest > limit_idx) {
            return Err(VdrError::InvalidRevocationRegistryStatusList(format!(
                "Highest delta index {} is higher than maximum allowed limit {}",
                highest_index.unwrap(),
                limit_idx
            )));
        }

        return Ok(());
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevocationStatusList {
    pub issuer_id: DID,
    pub rev_reg_def_id: RevocationRegistryDefinitionId,
    pub revocation_list: Vec<u32>,
    pub current_accumulator: String,
    pub timestamp: u64,
}

impl RevocationStatusList {
    //TODO:
    pub fn to_string(&self) -> VdrResult<String> {
        serde_json::to_string(self).map_err(|err| {
            VdrError::InvalidRevocationRegistryEntry(format!(
                "Unable to serialize Revocation RegistryE Entry as JSON. Err: {:?}",
                err
            ))
        })
    }

    //TODO:
    pub fn from_string(value: &str) -> VdrResult<RevocationStatusList> {
        serde_json::from_str(value).map_err(|err| {
            VdrError::InvalidRevocationRegistryEntry(format!(
                "Unable to parse Revocation Registry Entry from JSON. Err: {:?}",
                err.to_string()
            ))
        })
    }
}

#[derive(PartialEq)]
pub enum RevocationState {
    Active = 0,
    Revoked = 1,
}

impl TryFrom<u8> for RevocationState {
    type Error = VdrError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(RevocationState::Active),
            1 => Ok(RevocationState::Revoked),
            _ => Err(VdrError::InvalidRevocationRegistryStatusList(
                "Invalid Revocation State: Values should be 0 or 1".to_string(),
            )),
        }
    }
}
