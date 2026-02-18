use ethabi::Bytes;
use serde::{Deserialize, Serialize};

use crate::{
    types::transaction::Block, ContractEvent, ContractOutput, RevocationRegistryEntry, VdrError,
};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum RevocationRegistryEvents {
    RevocationRegistryEntryCreatedEvent(RevRegEntryCreated),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevRegEntryCreated {
    pub revocation_registry_definition_id: Vec<u8>,
    pub timestamp: u64,
    pub parent_block_number: Block,
    pub rev_reg_entry: RevocationRegistryEntry,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevRegEntry {
    pub current_accumulator: String,
    pub issued: Vec<u32>,
    pub revoked: Vec<u32>,
}

impl TryFrom<&Bytes> for RevRegEntry {
    type Error = VdrError;

    fn try_from(value: &Bytes) -> Result<Self, Self::Error> {
        serde_json::from_slice(&value).map_err(|err| {
            VdrError::ContractInvalidResponseData(format!(
                "Unable to parse RevRegEntry from the response. Err: {:?}",
                err
            ))
        })
    }
}

impl TryFrom<ContractEvent> for RevRegEntryCreated {
    type Error = VdrError;

    fn try_from(log: ContractEvent) -> Result<Self, Self::Error> {
        let revocation_registry_definition_id = log.get_fixed_bytes(0)?;
        let timestamp = log.get_uint(1)?;
        let parent_block_number = Block::try_from(log.get_uint(2).unwrap()).unwrap();
        let rev_reg_entry_tuple = log.get_bytes(3)?;
        let rev_reg_entry = RevocationRegistryEntry::try_from(&rev_reg_entry_tuple)?;

        Ok(RevRegEntryCreated {
            revocation_registry_definition_id,
            timestamp,
            parent_block_number,
            rev_reg_entry,
        })
    }
}
