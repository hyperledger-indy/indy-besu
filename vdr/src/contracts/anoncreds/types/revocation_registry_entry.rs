// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{error::VdrError, types::ContractParam, RevocationRegistryDefinitionId, VdrResult};

use crate::contracts::did::types::did::DID;

use crate::contracts::anoncreds::types::revocation_registry_delta::RevocationStatusList;

use ethabi::{Bytes, Uint};
use serde_derive::{Deserialize, Serialize};

/// Wrapper structure for DID
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct Accumulator(String);

impl Accumulator {
    pub(crate) fn validate(&self) -> VdrResult<()> {
        if self.0.is_empty() {
            return Err(VdrError::InvalidRevocationRegistryEntry(format!(
                "Incorrect Accumulator: {}",
                &self.0
            )));
        }

        Ok(())
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.0.as_bytes()
    }
}

impl From<&str> for Accumulator {
    fn from(acc: &str) -> Self {
        Accumulator(acc.to_string())
    }
}

impl AsRef<str> for Accumulator {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&Accumulator> for ContractParam {
    type Error = VdrError;

    fn try_from(value: &Accumulator) -> Result<Self, Self::Error> {
        Ok(ContractParam::Bytes(Bytes::from(value.as_bytes())))
    }
}

/// Definition of AnonCreds Revocation Registry Definition object matching to the specification - `<https://hyperledger.github.io/anoncreds-spec/#term:revocation-registry-entry>`
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevocationRegistryEntry {
    #[serde(rename = "revRegDefId")]
    pub rev_reg_def_id: RevocationRegistryDefinitionId,
    #[serde(rename = "issuerId")]
    pub issuer_id: DID,
    pub rev_reg_entry_data: RevocationRegistryEntryData,
}

/// Revocation Registry Entry Data stored in the Registry
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevocationRegistryEntryData {
    #[serde(rename = "currentAccumulator")]
    pub current_accumulator: Accumulator,
    #[serde(rename = "prevAccumulator")]
    pub prev_accumulator: Option<Accumulator>,
    pub issued: Option<Vec<u32>>,
    pub revoked: Option<Vec<u32>>,
}

impl RevocationRegistryEntry {
    pub fn new(
        rev_reg_def_id: RevocationRegistryDefinitionId,
        issuer_id: DID,
        current_accumulator: Accumulator,
        prev_accumulator: Option<Accumulator>,
        issued: Option<Vec<u32>>,
        revoked: Option<Vec<u32>>,
    ) -> RevocationRegistryEntry {
        RevocationRegistryEntry {
            rev_reg_def_id,
            issuer_id,
            rev_reg_entry_data: RevocationRegistryEntryData {
                prev_accumulator,
                current_accumulator,
                issued,
                revoked,
            },
        }
    }

    pub(crate) fn validate(&self) -> VdrResult<()> {
        self.rev_reg_entry_data.current_accumulator.validate()?;
        match self.rev_reg_entry_data.prev_accumulator {
            Some(ref prev_acc) => prev_acc.validate()?,
            None => {}
        };
        Ok(())
    }

    pub fn validate_with_ledger(
        &self,
        prev_accum_from_ledger: Option<Accumulator>,
    ) -> VdrResult<()> {
        match (
            &self.rev_reg_entry_data.prev_accumulator,
            prev_accum_from_ledger,
        ) {
            (Some(local), Some(ledger)) => {
                if local.as_ref() != ledger.as_ref() {
                    return Err(VdrError::InvalidRevocationRegistryEntry(format!(
                        "prev_accum mismatch: expected {}, found {}",
                        ledger.as_ref(),
                        local.as_ref()
                    )));
                }
            }
            (None, Some(_)) => {
                return Err(VdrError::InvalidRevocationRegistryEntry(
                    "prev_accum not provided locally, but exists on the ledger".to_string(),
                ));
            }
            (Some(_), None) => {
                return Err(VdrError::InvalidRevocationRegistryEntry(
                    "prev_accum provided locally, but does not exist on the ledger".to_string(),
                ));
            }
            (None, None) => {} // ok, both absent
        }

        Ok(())
    }

    pub fn validate_with_status_list(
        &self,
        status_list: &Option<RevocationStatusList>,
    ) -> VdrResult<()> {
        // 1. Local validation
        self.validate()?;

        // 2. Check issuer consistency
        if let Some(sl) = status_list {
            if self.issuer_id != sl.issuer_id {
                return Err(VdrError::InvalidRevocationRegistryEntry(format!(
                    "issuer mismatch: entry issuer {} != status list issuer {}",
                    self.issuer_id.to_string(),
                    sl.issuer_id.to_string()
                )));
            }
        }

        // 3. Transform accumulator
        let ledger_accum: Option<Accumulator> = match status_list {
            Some(sl) => {
                let s = sl.current_accumulator.as_str();
                if s.is_empty() {
                    None
                } else {
                    Some(Accumulator::from(s))
                }
            }
            None => None,
        };

        // 4. Validate with ledger accumulator
        self.validate_with_ledger(ledger_accum)?;
        Ok(())
    }

    pub fn to_string(&self) -> VdrResult<String> {
        serde_json::to_string(self).map_err(|err| {
            VdrError::InvalidRevocationRegistryEntry(format!(
                "Unable to serialize Revocation RegistryE Entry as JSON. Err: {:?}",
                err
            ))
        })
    }

    pub fn from_string(value: &str) -> VdrResult<RevocationRegistryEntry> {
        serde_json::from_str(value).map_err(|err| {
            VdrError::InvalidRevocationRegistryEntry(format!(
                "Unable to parse Revocation Registry Entry from JSON. Err: {:?}",
                err.to_string()
            ))
        })
    }
}

impl TryFrom<&Bytes> for RevocationRegistryEntry {
    type Error = VdrError;

    fn try_from(value: &Bytes) -> Result<Self, Self::Error> {
        serde_json::from_slice(&value).map_err(|err| {
            VdrError::InvalidRevocationRegistryEntry(format!(
                "Unable to parse Revocation Registry Entry from the response. Err: {:?}",
                err
            ))
        })
    }
}

impl TryFrom<&RevocationRegistryEntry> for ContractParam {
    type Error = VdrError;

    fn try_from(value: &RevocationRegistryEntry) -> Result<Self, Self::Error> {
        serde_json::to_vec(value)
            .map(ContractParam::Bytes)
            .map_err(|_| VdrError::ContractInvalidInputData)
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    // === Helper functions for tests ===

    /// Creates a RevocationRegistryEntryData with optional current and previous accumulator
    pub fn revocation_registry_entry_data(
        revoked_indices: Option<Vec<u32>>,
        accum: Option<&str>,
        prev_accum: Option<&str>,
    ) -> RevocationRegistryEntryData {
        RevocationRegistryEntryData {
            current_accumulator: match accum {
                Some(acc) => Accumulator::from(acc),
                None => Accumulator::from("currentAccum"),
            },
            prev_accumulator: prev_accum.map(|acc| Accumulator::from(acc)),
            issued: None,
            revoked: revoked_indices,
        }
    }

    /// Creates a RevocationRegistryEntry with optional accumulators
    pub fn revocation_registry_entry(
        issuer_id: &DID,
        rev_reg_def_id: &RevocationRegistryDefinitionId,
        revoked_indices: Option<Vec<u32>>,
        accum: Option<&str>,
        prev_accum: Option<&str>,
    ) -> RevocationRegistryEntry {
        RevocationRegistryEntry {
            issuer_id: issuer_id.clone(),
            rev_reg_def_id: rev_reg_def_id.clone(),
            rev_reg_entry_data: revocation_registry_entry_data(revoked_indices, accum, prev_accum),
        }
    }

    fn fake_status_list_with_issuer(issuer: &str) -> RevocationStatusList {
        RevocationStatusList {
            current_accumulator: "123".to_string(),
            revocation_list: vec![0u32; 32], // must be non-empty for tests
            rev_reg_def_id: RevocationRegistryDefinitionId::from("rev_reg_def"),
            issuer_id: DID::from(issuer),
            timestamp: u64::MAX,
        }
    }

    /// Generates a fake RevocationStatusList for testing
    pub fn fake_status_list(accum: &str) -> RevocationStatusList {
        RevocationStatusList {
            current_accumulator: accum.to_string(),
            revocation_list: vec![0u32; 32], // ensures a vec32
            rev_reg_def_id: RevocationRegistryDefinitionId::build(
                &DID::default(),
                "cred_def_id",
                "tag",
            ),
            issuer_id: DID::default(),
            timestamp: u64::MAX,
        }
    }

    // === Basic accumulator tests ===

    #[test]
    pub fn accumulator_validate_empty_fails() {
        let res = Accumulator::from("").validate();
        assert!(res.is_err());
        assert!(format!("{:?}", res.unwrap_err()).contains("Incorrect Accumulator"));
    }

    #[test]
    pub fn accumulator_validate_ok() {
        let acc = Accumulator::from("valid_acc");
        assert!(acc.validate().is_ok());
    }

    // === Local validation tests ===

    #[test]
    pub fn rev_reg_entry_validate_local_ok() {
        let issuer = DID::default();
        let rev_reg_def_id = RevocationRegistryDefinitionId::build(&issuer, "cred_def_id", "tag");
        let entry =
            revocation_registry_entry(&issuer, &rev_reg_def_id, None, None, Some("prev123"));
        assert!(entry.validate().is_ok());
    }

    // === validate_with_status_list tests ===

    // 1️⃣ prev_acc == current_accumulator -> should pass
    #[test]
    pub fn validate_status_list_match_ok() {
        let issuer = DID::default();
        let rev_reg_def_id = RevocationRegistryDefinitionId::build(&issuer, "cred_def_id", "tag");
        let entry = revocation_registry_entry(&issuer, &rev_reg_def_id, None, None, Some("123"));
        let status_list = fake_status_list("123");

        let res = entry.validate_with_status_list(&Some(status_list));
        assert!(res.is_ok());
    }

    // 2️⃣ prev_acc != current_accumulator -> should fail
    #[test]
    pub fn validate_status_list_mismatch_fails() {
        let issuer = DID::default();
        let rev_reg_def_id = RevocationRegistryDefinitionId::build(&issuer, "cred_def_id", "tag");
        let entry =
            revocation_registry_entry(&issuer, &rev_reg_def_id, None, None, Some("localPrev"));
        let status_list = fake_status_list("ledgerPrev");

        let res = entry.validate_with_status_list(&Some(status_list));
        assert!(res.is_err());
        assert!(format!("{}", res.unwrap_err()).contains("prev_accum mismatch"));
    }

    // 3️⃣ prev_acc Some, ledger None -> should fail
    #[test]
    pub fn validate_status_list_none_ledger_fails() {
        let issuer = DID::default();
        let rev_reg_def_id = RevocationRegistryDefinitionId::build(&issuer, "cred_def_id", "tag");
        let entry =
            revocation_registry_entry(&issuer, &rev_reg_def_id, None, None, Some("localPrev"));

        let res = entry.validate_with_status_list(&None);
        assert!(res.is_err());
        assert!(format!("{}", res.unwrap_err())
            .contains("prev_accum provided locally, but does not exist on the ledger"));
    }

    // 4️⃣ prev_acc None, ledger Some -> should fail
    #[test]
    pub fn validate_status_list_none_local_fails() {
        let issuer = DID::default();
        let rev_reg_def_id = RevocationRegistryDefinitionId::build(&issuer, "cred_def_id", "tag");
        let entry = revocation_registry_entry(&issuer, &rev_reg_def_id, None, None, None);
        let status_list = fake_status_list("ledgerPrev");

        let res = entry.validate_with_status_list(&Some(status_list));
        assert!(res.is_err());
        assert!(format!("{}", res.unwrap_err())
            .contains("prev_accum not provided locally, but exists on the ledger"));
    }

    // 5️⃣ prev_acc None, ledger None -> should pass (first creation)
    #[test]
    pub fn validate_status_list_none_both_ok() {
        let issuer = DID::default();
        let rev_reg_def_id = RevocationRegistryDefinitionId::build(&issuer, "cred_def_id", "tag");
        let entry = revocation_registry_entry(&issuer, &rev_reg_def_id, None, None, None);
        let status_list = fake_status_list(""); // simulates empty ledger

        let res = entry.validate_with_status_list(&Some(status_list));
        assert!(res.is_ok());
    }

    // 6️⃣ prev_acc 0, ledger empty -> should pass (first creation)
    #[test]
    pub fn validate_status_list_prev_none_ledger_empty_ok() {
        let issuer = DID::default();
        let rev_reg_def_id = RevocationRegistryDefinitionId::build(&issuer, "cred_def_id", "tag");
        let entry =
            revocation_registry_entry(&issuer, &rev_reg_def_id, None, Some("prev123"), None);

        let res = entry.validate_with_status_list(&None);
        assert!(res.is_ok());
    }

    #[test]
    fn validate_with_status_list_correct_issuer_ok() {
        let issuer = "did:ethr:0x123";
        let rev_reg_def_id = RevocationRegistryDefinitionId::from("rev_reg_def");
        let entry = revocation_registry_entry(
            &DID::from(issuer),
            &rev_reg_def_id,
            None,
            Some("123"),
            Some("123"),
        );

        let status_list = Some(fake_status_list_with_issuer(issuer));

        let res = entry.validate_with_status_list(&status_list);
        assert!(res.is_ok(), "Validation should pass when issuer matches");
    }

    #[test]
    fn validate_with_status_list_incorrect_issuer_fails() {
        let issuer = "did:ethr:0x123";
        let wrong_issuer = "did:ethr:0x456";
        let rev_reg_def_id = RevocationRegistryDefinitionId::from("rev_reg_def");
        let entry = revocation_registry_entry(
            &DID::from(issuer),
            &rev_reg_def_id,
            None,
            Some("123"),
            Some("123"),
        );

        let status_list = Some(fake_status_list_with_issuer(wrong_issuer));

        let res = entry.validate_with_status_list(&status_list);
        assert!(
            res.is_err(),
            "Validation should fail when issuer does not match"
        );
        assert!(format!("{}", res.unwrap_err()).contains("issuer mismatch"));
    }

    #[test]
    fn validate_with_status_list_none_status_list_ok() {
        let issuer = "did:ethr:0x123";
        let rev_reg_def_id = RevocationRegistryDefinitionId::from("rev_reg_def");
        let entry =
            revocation_registry_entry(&DID::from(issuer), &rev_reg_def_id, None, Some("123"), None);

        // None status_list: should only perform local validation
        let res = entry.validate_with_status_list(&None);
        assert!(
            res.is_ok(),
            "Validation should pass when status list is None"
        );
    }
}
