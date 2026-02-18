// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{
    ledger::{BesuLedger, IndyLedger, Ledgers},
    wallet::{BesuWallet, IndyWallet},
};
use indy_besu_vdr::{Address, DidDocAttribute, ResourceIdentifier, DID};
use indy_credx::types::{
    AttributeNames, AttributeValues, Credential, CredentialDefinition, CredentialDefinitionConfig,
    CredentialDefinitionId, CredentialDefinitionPrivate, CredentialKeyCorrectnessProof,
    CredentialOffer, CredentialRequest, CredentialValues, DidValue, Schema, SchemaId,
    SignatureType,
};
use serde_json::json;
use std::collections::HashMap;

pub struct Issuer {
    pub indy_wallet: IndyWallet,
    pub indy_ledger: IndyLedger,
    pub besu_wallet: BesuWallet,
    pub besu_ledger: BesuLedger,
    pub indy_did: String,
    pub besu_did: String,
    pub edkey: String,
    pub account: Address,
    pub secpkey: String,
    pub service: String,
    pub used_ledger: Ledgers,
    pub schema_attributes: Vec<String>,
    pub credential_values: HashMap<String, AttributeValues>,
    cred_def_priv: Option<CredentialDefinitionPrivate>,
    correctness_proof: Option<CredentialKeyCorrectnessProof>,
}

impl Issuer {
    const SCHEMA_NAME: &'static str = "test_credential";
    const SCHEMA_VERSION: &'static str = "1.0.0";

    const SERVICE_ENDPOINT: &'static str = "127.0.0.1:5555";

    pub async fn setup() -> Issuer {
        let indy_wallet = IndyWallet::new(None).await;
        let indy_ledger = IndyLedger::new();
        let besu_wallet = BesuWallet::new(None);
        let besu_ledger = BesuLedger::new().await;

        let indy_did = indy_wallet.did.clone();
        let edkey = indy_wallet.edkey.clone();
        let account = besu_wallet.account.clone();
        let secpkey = besu_wallet.secpkey.clone();
        let besu_did = DID::build("ethr", None, account.as_ref());

        let schema_attributes = vec!["name".to_string(), "age".to_string()];

        let mut credential_values: HashMap<String, AttributeValues> = HashMap::new();
        credential_values.insert(
            "name".to_string(),
            AttributeValues {
                raw: "Alex".to_string(),
                encoded: "1139481716457488690172217916278103335".to_string(),
            },
        );
        credential_values.insert(
            "age".to_string(),
            AttributeValues {
                raw: "28".to_string(),
                encoded: "28".to_string(),
            },
        );

        Issuer {
            indy_ledger,
            indy_wallet,
            besu_ledger,
            besu_wallet,
            indy_did,
            besu_did: besu_did.to_string(),
            edkey,
            account,
            secpkey,
            used_ledger: Ledgers::Indy,
            service: Self::SERVICE_ENDPOINT.to_string(),
            cred_def_priv: None,
            correctness_proof: None,
            schema_attributes,
            credential_values,
        }
    }

    pub async fn create_schema(&self) -> (SchemaId, Schema) {
        let schema = indy_credx::issuer::create_schema(
            &DidValue(self.indy_did.to_string()),
            Self::SCHEMA_NAME,
            Self::SCHEMA_VERSION,
            AttributeNames::from(self.schema_attributes.clone()),
            None,
        )
        .unwrap();
        (schema.id().clone(), schema)
    }

    pub async fn create_cred_def(
        &mut self,
        schema_id: &SchemaId,
    ) -> (CredentialDefinitionId, CredentialDefinition) {
        let schema = self.indy_ledger.get_schema(schema_id).await;
        let (cred_def, cred_def_priv, cred_def_proof) =
            indy_credx::issuer::create_credential_definition(
                &DidValue(self.indy_did.to_string()),
                &schema,
                Self::SCHEMA_VERSION,
                SignatureType::CL,
                CredentialDefinitionConfig::new(false),
            )
            .unwrap();

        self.cred_def_priv = Some(cred_def_priv);
        self.correctness_proof = Some(cred_def_proof);

        (cred_def.id().clone(), cred_def)
    }

    pub fn create_credential_offer(
        &self,
        schema_id: &SchemaId,
        cred_def: &CredentialDefinition,
    ) -> CredentialOffer {
        let correctness_proof = self
            .correctness_proof
            .as_ref()
            .expect("missing correctness_proof");
        indy_credx::issuer::create_credential_offer(schema_id, cred_def, correctness_proof).unwrap()
    }

    pub fn sign_credential(
        &self,
        cred_def: &CredentialDefinition,
        cred_offer: &CredentialOffer,
        cred_req: &CredentialRequest,
    ) -> Credential {
        let cred_def_priv = self.cred_def_priv.as_ref().expect("missing cred_def_priv");
        let (credential, _, _) = indy_credx::issuer::create_credential(
            cred_def,
            cred_def_priv,
            cred_offer,
            cred_req,
            CredentialValues(self.credential_values.clone()),
            None,
        )
        .unwrap();
        credential
    }

    pub async fn publish_attrib_to_indy(&self, attrib: &serde_json::Value) {
        self.indy_ledger
            .publish_attrib(&self.indy_wallet, &self.indy_did, &self.indy_did, attrib)
            .await;
    }

    pub async fn publish_service_endpoint_to_indy(&self, endpoint: &str) {
        let endpoint = json!({
            "endpoint":{
                "ha": endpoint
            }
        });
        self.publish_attrib_to_indy(&endpoint).await
    }

    pub async fn publish_besu_ledger_account_to_indy(&self, key: &str) {
        let key = json!({
            "besu":{
                "key": key
            }
        });
        self.publish_attrib_to_indy(&key).await
    }

    pub async fn publish_schema_to_indy(&self, schema: &Schema) {
        self.indy_ledger
            .publish_schema(&self.indy_wallet, &self.indy_did, schema)
            .await;
    }

    pub async fn publish_cred_def_to_indy(&self, cred_def: &CredentialDefinition) {
        self.indy_ledger
            .publish_cred_def(&self.indy_wallet, &self.indy_did, cred_def)
            .await;
    }

    pub async fn publish_did_attribute_to_besu(&self, attribute: &DidDocAttribute) {
        self.besu_ledger
            .publish_did_attribute(&self.account, &self.besu_did, attribute, &self.besu_wallet)
            .await
    }
    pub async fn publish_did_mapping_to_besu(&self) {
        self.besu_ledger
            .publish_did_mapping(
                &self.account,
                &self.besu_did,
                &self.indy_did,
                &self.edkey,
                &vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                &self.besu_wallet,
            )
            .await
    }

    pub async fn publish_schema_to_besu(
        &self,
        schema: &indy_besu_vdr::Schema,
    ) -> indy_besu_vdr::SchemaId {
        self.besu_ledger
            .publish_schema(&self.account, schema, &self.besu_wallet)
            .await
    }

    pub async fn publish_schema_id_mapping_to_besu(
        &self,
        legacy_did: &str,
        legacy_schema_id: &SchemaId,
        new_schema_id: &indy_besu_vdr::SchemaId,
    ) {
        self.besu_ledger
            .publish_resource_mapping(
                &self.account,
                &indy_besu_vdr::DID::from(self.besu_did.as_str()),
                &indy_data_types::did::DidValue(legacy_did.to_string()),
                &ResourceIdentifier::from(legacy_schema_id.0.as_str()),
                &ResourceIdentifier::from(new_schema_id.as_ref()),
                &self.besu_wallet,
            )
            .await
    }

    pub async fn publish_cred_def_to_besu(
        &self,
        cred_def: &indy_besu_vdr::CredentialDefinition,
    ) -> indy_besu_vdr::CredentialDefinitionId {
        self.besu_ledger
            .publish_cred_def(&self.account, cred_def, &self.besu_wallet)
            .await
    }

    pub async fn publish_cred_def_id_mapping_to_besu(
        &self,
        legacy_did: &str,
        legacy_cred_def_id: &CredentialDefinitionId,
        new_cred_def_id: &indy_besu_vdr::CredentialDefinitionId,
    ) {
        self.besu_ledger
            .publish_resource_mapping(
                &self.account,
                &indy_besu_vdr::DID::from(self.besu_did.as_str()),
                &indy_data_types::did::DidValue(legacy_did.to_string()),
                &ResourceIdentifier::from(legacy_cred_def_id.0.as_str()),
                &ResourceIdentifier::from(new_cred_def_id.as_ref()),
                &self.besu_wallet,
            )
            .await
    }

    pub fn use_indy_ledger(&mut self) {
        self.used_ledger = Ledgers::Indy
    }

    pub fn use_besu_ledger(&mut self) {
        self.used_ledger = Ledgers::Besu
    }
}
