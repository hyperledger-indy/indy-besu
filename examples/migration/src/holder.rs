// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{
    ledger::{BesuLedger, IndyLedger, Ledgers},
    wallet::{BesuWallet, IndyWallet},
};
use indy_credx::types::{
    Credential, CredentialDefinition, CredentialDefinitionId, CredentialOffer, CredentialRequest,
    CredentialRequestMetadata, DidValue, LinkSecret, PresentCredentials, Presentation,
    PresentationRequest, Schema, SchemaId,
};
use std::collections::HashMap;

pub struct Holder {
    pub indy_wallet: IndyWallet,
    pub indy_ledger: IndyLedger,
    pub besu_ledger: BesuLedger,
    pub besu_wallet: BesuWallet,
    pub indy_did: String,
    pub master_secret: LinkSecret,
    pub master_secret_name: String,
    pub used_ledger: Ledgers,
    cred_request_meta: Option<CredentialRequestMetadata>,
}

impl Holder {
    const NAME: &'static str = "holder";

    pub async fn setup() -> Holder {
        let indy_wallet = IndyWallet::new(None).await;
        let indy_ledger = IndyLedger::new();
        let besu_wallet = BesuWallet::new(None);
        let besu_ledger = BesuLedger::new().await;
        let master_secret = indy_credx::prover::create_link_secret().unwrap();
        let indy_did = indy_wallet.did.clone();
        Holder {
            indy_wallet,
            indy_ledger,
            besu_wallet,
            besu_ledger,
            indy_did,
            master_secret,
            master_secret_name: Self::NAME.to_string(),
            used_ledger: Ledgers::Indy,
            cred_request_meta: None,
        }
    }

    pub async fn create_credential_request(
        &mut self,
        cred_offer: &CredentialOffer,
    ) -> CredentialRequest {
        let cred_def = self.get_cred_def(&cred_offer.cred_def_id).await;
        let (cred_req, cred_request_meta) = indy_credx::prover::create_credential_request(
            &DidValue(self.indy_did.to_string()),
            &cred_def,
            &self.master_secret,
            &self.master_secret_name,
            cred_offer,
        )
        .unwrap();
        self.cred_request_meta = Some(cred_request_meta);
        cred_req
    }

    pub async fn store_credential(&self, credential: &mut Credential) {
        let cred_def = self.get_cred_def(&credential.cred_def_id).await;
        let cred_request_meta = self
            .cred_request_meta
            .as_ref()
            .expect("missing cred_request_meta");
        indy_credx::prover::process_credential(
            credential,
            cred_request_meta,
            &self.master_secret,
            &cred_def,
            None,
        )
        .unwrap();
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

    pub async fn make_proof(
        &self,
        proof_request: &PresentationRequest,
        credential: &Credential,
    ) -> Presentation {
        let schema = self.get_schema(&credential.schema_id).await;
        let cred_def = self.get_cred_def(&credential.cred_def_id).await;

        let mut schemas: HashMap<SchemaId, &Schema> = HashMap::new();
        schemas.insert(credential.schema_id.clone(), &schema);

        let mut cred_defs: HashMap<CredentialDefinitionId, &CredentialDefinition> = HashMap::new();
        cred_defs.insert(credential.cred_def_id.clone(), &cred_def);

        let mut credentials = PresentCredentials::new();
        credentials
            .add_credential(credential, None, None)
            .add_requested_attribute("attr1_referent", true);

        indy_credx::prover::create_presentation(
            proof_request,
            credentials,
            None,
            &self.master_secret,
            &schemas,
            &cred_defs,
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
