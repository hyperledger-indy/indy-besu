// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::ledger::{BesuLedger, IndyLedger, Ledgers};
use indy_credx::types::{
    CredentialDefinition, CredentialDefinitionId, Presentation, PresentationRequest, Schema,
    SchemaId,
};
use serde_json::json;
use std::collections::HashMap;

pub struct Verifier {
    pub indy_ledger: IndyLedger,
    pub besu_ledger: BesuLedger,
    pub used_ledger: Ledgers,
}

impl Verifier {
    pub async fn setup() -> Verifier {
        let indy_ledger = IndyLedger::new();
        let besu_ledger = BesuLedger::new().await;
        Verifier {
            indy_ledger,
            besu_ledger,
            used_ledger: Ledgers::Indy,
        }
    }

    async fn get_schema(&self, schema_id: &str) -> Schema {
        match self.used_ledger {
            Ledgers::Indy => self.indy_ledger.get_schema(schema_id).await,
            Ledgers::Besu => self.besu_ledger.get_schema(schema_id).await,
        }
    }

    async fn get_cred_def(&self, cred_def_id: &str) -> CredentialDefinition {
        match self.used_ledger {
            Ledgers::Indy => self.indy_ledger.get_cred_def(cred_def_id).await,
            Ledgers::Besu => self.besu_ledger.get_cred_def(cred_def_id).await,
        }
    }

    pub fn request() -> PresentationRequest {
        serde_json::from_value(json!({
           "nonce":"123432421212",
           "name":"proof_req_1",
           "version":"0.1",
           "requested_attributes": {
                "attr1_referent": {
                    "name": "name"
                }
           },
           "requested_predicates": {}
        }))
        .unwrap()
    }

    pub async fn verify_proof(
        &self,
        proof_request: &PresentationRequest,
        proof: &Presentation,
    ) -> bool {
        let identifier = proof.identifiers[0].clone();
        let schema_id = identifier.schema_id;
        let cred_def_id = identifier.cred_def_id;

        let schema = self.get_schema(&schema_id).await;
        let cred_def = self.get_cred_def(&cred_def_id).await;

        let mut schemas: HashMap<SchemaId, &Schema> = HashMap::new();
        schemas.insert(schema_id, &schema);

        let mut cred_defs: HashMap<CredentialDefinitionId, &CredentialDefinition> = HashMap::new();
        cred_defs.insert(cred_def_id, &cred_def);

        indy_credx::verifier::verify_presentation(
            proof,
            proof_request,
            &schemas,
            &cred_defs,
            None,
            None,
        )
        .unwrap()
    }

    pub fn use_indy_ledger(&mut self) {
        self.used_ledger = Ledgers::Indy
    }

    pub fn use_besu_ledger(&mut self) {
        self.used_ledger = Ledgers::Besu
    }
}
