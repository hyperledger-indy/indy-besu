// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{
    client::client::test::{client, TRUSTEE_ACCOUNT},
    contracts::{
        anoncreds::types::{
            credential_definition::test::credential_definition,
            revocation_registry_definition::test::revocation_registry_definition,
            revocation_registry_entry::test::revocation_registry_entry, schema::test::schema,
        },
        auth::{role_control, Role},
        did::{did_indy_registry, types::did_doc::test::did_doc, DidRecord, DID, ETHR_DID_METHOD},
    },
    signer::basic_signer::{test::basic_signer, BasicSigner},
    types::{Address, SignatureData, Transaction},
    LedgerClient,
};

mod helpers {
    use super::*;
    use crate::{contracts::endorsing, Address, LedgerClient, TransactionEndorsingData};

    pub async fn sign_and_submit_transaction(
        client: &LedgerClient,
        mut transaction: Transaction,
        signer: &BasicSigner,
    ) -> String {
        let sign_bytes = transaction.get_signing_bytes().unwrap();
        let from = transaction.from.as_ref().unwrap();
        let signature = signer.sign(&sign_bytes, from.as_ref()).unwrap();
        transaction.set_signature(signature);
        let block_hash = client.submit_transaction(&transaction).await.unwrap();
        client.get_receipt(&block_hash).await.unwrap()
    }

    pub fn sign_endorsing_data(
        data: &TransactionEndorsingData,
        signer: &BasicSigner,
    ) -> SignatureData {
        signer
            .sign(&data.get_signing_bytes().unwrap(), data.from.as_ref())
            .unwrap()
    }

    pub async fn endorse_transaction(
        client: &LedgerClient,
        signer: &BasicSigner,
        mut data: TransactionEndorsingData,
    ) {
        let signature = signer
            .sign(&data.get_signing_bytes().unwrap(), data.from.as_ref())
            .unwrap();

        data.set_signature(signature);

        let transaction = endorsing::build_endorsement_transaction(client, &TRUSTEE_ACCOUNT, &data)
            .await
            .unwrap();

        super::helpers::sign_and_submit_transaction(&client, transaction, &signer).await;
    }

    pub async fn assign_role(
        client: &LedgerClient,
        assignee_account: &Address,
        role_to_assign: &Role,
        signer: &BasicSigner,
    ) -> String {
        let transaction = role_control::build_assign_role_transaction(
            client,
            &TRUSTEE_ACCOUNT,
            role_to_assign,
            assignee_account,
        )
        .await
        .unwrap();
        sign_and_submit_transaction(client, transaction, signer).await
    }

    pub async fn create_trustee(signer: &mut BasicSigner, client: &LedgerClient) -> Address {
        let (identity, _) = signer.create_key(None).unwrap();
        assign_role(&client, &identity, &Role::Trustee, &signer).await;
        identity
    }
}

mod did_indy {
    use super::*;
    use crate::{client::client::test::TRUSTEE_ACCOUNT, contracts::endorsing};

    async fn resolve_did(client: &LedgerClient, did: &DID) -> DidRecord {
        let transaction = did_indy_registry::build_resolve_did_transaction(client, did)
            .await
            .unwrap();
        let result = client.submit_transaction(&transaction).await.unwrap();
        did_indy_registry::parse_resolve_did_result(client, &result).unwrap()
    }

    #[async_std::test]
    async fn create_did_test() {
        let mut signer = basic_signer();
        let client = client();
        let identity = super::helpers::create_trustee(&mut signer, &client).await;

        // create
        let did_doc = did_doc(identity.as_ref());
        let transaction = did_indy_registry::build_create_did_transaction(
            &client,
            &identity,
            &did_doc.id,
            &did_doc,
        )
        .await
        .unwrap();
        super::helpers::sign_and_submit_transaction(&client, transaction, &signer).await;

        // read
        let resolved_did_record = resolve_did(&client, &did_doc.id).await;
        assert_eq!(did_doc, resolved_did_record.document);
    }

    #[async_std::test]
    async fn create_and_deactivate_test() {
        let mut signer = basic_signer();
        let client = client();
        let identity = super::helpers::create_trustee(&mut signer, &client).await;

        // create
        let did_doc = did_doc(identity.as_ref());
        let transaction = did_indy_registry::build_create_did_transaction(
            &client,
            &identity,
            &did_doc.id,
            &did_doc,
        )
        .await
        .unwrap();
        super::helpers::sign_and_submit_transaction(&client, transaction, &signer).await;

        // deactivate
        let transaction =
            did_indy_registry::build_deactivate_did_transaction(&client, &identity, &did_doc.id)
                .await
                .unwrap();
        super::helpers::sign_and_submit_transaction(&client, transaction, &signer).await;

        // read
        let resolved_did_record = resolve_did(&client, &did_doc.id).await;
        assert_eq!(true, resolved_did_record.metadata.deactivated.unwrap());
    }

    #[async_std::test]
    async fn endorse_did_test() {
        let mut signer = basic_signer();
        let client = client();
        let (identity, _) = signer.create_key(None).unwrap();

        // create
        let did_doc = did_doc(identity.as_ref());
        let mut endorsement_data =
            did_indy_registry::build_create_did_endorsing_data(&client, &did_doc.id, &did_doc)
                .await
                .unwrap();

        let signature = super::helpers::sign_endorsing_data(&endorsement_data, &signer);
        endorsement_data.set_signature(signature);

        let transaction =
            endorsing::build_endorsement_transaction(&client, &TRUSTEE_ACCOUNT, &endorsement_data)
                .await
                .unwrap();

        super::helpers::sign_and_submit_transaction(&client, transaction, &signer).await;

        // read
        let resolved_did_record = resolve_did(&client, &did_doc.id).await;
        assert_eq!(did_doc, resolved_did_record.document);
    }
}

mod did_ethr {
    use super::*;
    use crate::{
        contracts::{
            did::{
                did_ethr_registry,
                did_ethr_registry::test::{public_key, service, validity},
                did_resolver,
                types::{
                    did_doc::test::default_ethr_did_document, did_doc_attribute::DidDocAttribute,
                },
                DID,
            },
            types::did::ParsedDid,
            ETHR_DID_METHOD,
        },
        did_ethr_registry::test::{public_key_2, public_key_3},
        Address, LedgerClient, Validity,
    };

    async fn endorse_set_did_attribute(
        client: &LedgerClient,
        did: &DID,
        attribute: &DidDocAttribute,
        validity: &Validity,
        signer: &BasicSigner,
    ) {
        let transaction_endorsing_data = did_ethr_registry::build_did_set_attribute_endorsing_data(
            client, did, attribute, validity,
        )
        .await
        .unwrap();

        super::helpers::endorse_transaction(client, signer, transaction_endorsing_data).await;
    }

    async fn endorse_revoke_did_attribute(
        client: &LedgerClient,
        did: &DID,
        attribute: &DidDocAttribute,
        signer: &BasicSigner,
    ) {
        let transaction_endorsing_data =
            did_ethr_registry::build_did_revoke_attribute_endorsing_data(client, did, attribute)
                .await
                .unwrap();

        super::helpers::endorse_transaction(client, signer, transaction_endorsing_data).await;
    }

    #[async_std::test]
    async fn demo_create_did_ethr() {
        let mut signer = basic_signer();
        let client = client();
        let identity = super::helpers::create_trustee(&mut signer, &client).await;

        let did = DID::build(ETHR_DID_METHOD, None, identity.as_ref());

        // read DID changed block -> it must be none
        let transaction = did_ethr_registry::build_get_did_changed_transaction(&client, &did)
            .await
            .unwrap();
        let result = client.submit_transaction(&transaction).await.unwrap();
        let changed = did_ethr_registry::parse_did_changed_result(&client, &result).unwrap();
        assert!(changed.is_none());

        // add service attribute to DID
        let transaction = did_ethr_registry::build_did_set_attribute_transaction(
            &client,
            &identity,
            &did,
            &service(),
            &validity(),
        )
        .await
        .unwrap();
        super::helpers::sign_and_submit_transaction(&client, transaction, &signer).await;

        // Read DID events
        let transaction = did_ethr_registry::build_get_did_events_query(&client, &did, None, None)
            .await
            .unwrap();
        let events = client.query_events(&transaction).await.unwrap();
        assert_eq!(1, events.len());
        let event = did_ethr_registry::parse_did_event_response(&client, &events[0]).unwrap();
        let _attribute: DidDocAttribute = event.try_into().unwrap();

        // read DID changed block -> it must be NOT none
        let transaction = did_ethr_registry::build_get_did_changed_transaction(&client, &did)
            .await
            .unwrap();
        let result = client.submit_transaction(&transaction).await.unwrap();
        let changed = did_ethr_registry::parse_did_changed_result(&client, &result).unwrap();
        assert!(!changed.is_none());

        // add service key to DID
        let transaction = did_ethr_registry::build_did_set_attribute_transaction(
            &client,
            &identity,
            &did,
            &public_key(),
            &validity(),
        )
        .await
        .unwrap();
        super::helpers::sign_and_submit_transaction(&client, transaction, &signer).await;

        // resolve DID document
        let did_doc_with_meta = did_resolver::resolve_did(&client, &did, None)
            .await
            .unwrap();
        let did_document = did_doc_with_meta.did_document.unwrap();
        assert_eq!(1, did_document.service.len());
        assert_eq!(2, did_document.verification_method.len());
        assert_eq!(
            false,
            did_doc_with_meta.did_document_metadata.deactivated.unwrap()
        );
    }

    #[async_std::test]
    async fn demo_endorse_did_ethr() {
        let mut signer = basic_signer();
        let client = client();
        let (identity, _) = signer.create_key(None).unwrap();
        let did = DID::build(ETHR_DID_METHOD, None, identity.as_ref());

        // endorse service attribute
        endorse_set_did_attribute(&client, &did, &service(), &validity(), &signer).await;

        // endorse key attribute
        endorse_set_did_attribute(&client, &did, &public_key(), &validity(), &signer).await;

        // resolve DID document
        let did_doc_with_meta = did_resolver::resolve_did(&client, &did, None)
            .await
            .unwrap();
        let did_document = did_doc_with_meta.did_document.unwrap();
        assert_eq!(1, did_document.service.len());
        assert_eq!(2, did_document.verification_method.len());
    }

    #[async_std::test]
    async fn demo_did_ethr_deactivate() {
        let mut signer = basic_signer();
        let client = client();
        let (identity, _) = signer.create_key(None).unwrap();

        let did = DID::build(ETHR_DID_METHOD, None, identity.as_ref());

        // add service attribute
        let service = service();
        let validity = validity();
        endorse_set_did_attribute(&client, &did, &service, &validity, &signer).await;

        // deactivate DID
        let new_owner = Address::null();
        let transaction_endorsing_data =
            did_ethr_registry::build_did_change_owner_endorsing_data(&client, &did, &new_owner)
                .await
                .unwrap();
        super::helpers::endorse_transaction(&client, &signer, transaction_endorsing_data).await;

        // Resole DID document
        let did_doc_with_meta = did_resolver::resolve_did(&client, &did, None)
            .await
            .unwrap();
        let did_document = did_doc_with_meta.did_document.unwrap();

        // DID is deactivated
        assert!(did_doc_with_meta.did_document_metadata.deactivated.unwrap());

        // DID Document is empty
        let parse_did = ParsedDid::try_from(&did).unwrap();
        assert_eq!(parse_did.as_short_did(), did_document.id);
        assert_eq!(0, did_document.service.len());
        assert_eq!(0, did_document.verification_method.len());
        assert_eq!(0, did_document.authentication.len());
        assert_eq!(0, did_document.assertion_method.len());
    }

    #[async_std::test]
    async fn demo_did_ethr_add_remove_attribute() {
        let mut signer = basic_signer();
        let client = client();
        let (identity, _) = signer.create_key(None).unwrap();

        let did = DID::build(ETHR_DID_METHOD, None, identity.as_ref());

        // set service attribute
        let service = service();
        let validity = validity();
        endorse_set_did_attribute(&client, &did, &service, &validity, &signer).await;

        // set first key attribute
        let public_key = public_key();
        endorse_set_did_attribute(&client, &did, &public_key, &validity, &signer).await;

        // set second key attribute
        let public_key_2 = public_key_2();
        endorse_set_did_attribute(&client, &did, &public_key_2, &validity, &signer).await;

        // resolve DID document
        let did_doc_with_meta = did_resolver::resolve_did(&client, &did, None)
            .await
            .unwrap();
        let did_document_before_remove = did_doc_with_meta.did_document.unwrap();
        assert_eq!(1, did_document_before_remove.service.len());
        assert_eq!(3, did_document_before_remove.verification_method.len());
        assert_eq!(2, did_document_before_remove.key_agreement.len());
        assert_eq!(1, did_document_before_remove.authentication.len());
        assert_eq!(1, did_document_before_remove.assertion_method.len());

        // remove service and second key
        endorse_revoke_did_attribute(&client, &did, &public_key, &signer).await;
        endorse_revoke_did_attribute(&client, &did, &service, &signer).await;

        // resolve DID document
        let did_doc_with_meta = did_resolver::resolve_did(&client, &did, None)
            .await
            .unwrap();
        let did_document_after_remove = did_doc_with_meta.did_document.unwrap();
        assert_eq!(0, did_document_after_remove.service.len());
        assert_eq!(2, did_document_after_remove.verification_method.len());
        assert_eq!(1, did_document_after_remove.key_agreement.len());
        assert_eq!(1, did_document_after_remove.authentication.len());
        assert_eq!(1, did_document_after_remove.assertion_method.len());

        // add third key
        let public_key_3 = public_key_3();
        endorse_set_did_attribute(&client, &did, &public_key_3, &validity, &signer).await;

        // resolve DID document
        let did_doc_with_meta = did_resolver::resolve_did(&client, &did, None)
            .await
            .unwrap();
        let did_document_after_add = did_doc_with_meta.did_document.unwrap();
        assert_eq!(0, did_document_after_add.service.len());
        assert_eq!(3, did_document_after_add.verification_method.len());
        assert_eq!(1, did_document_after_add.key_agreement.len());
        assert_eq!(1, did_document_after_add.authentication.len());
        assert_eq!(2, did_document_after_add.assertion_method.len());
    }

    #[async_std::test]
    async fn demo_resolve_offchain_did() {
        let mut signer = basic_signer();
        let client = client();
        let (identity, _) = signer.create_key(None).unwrap();

        let did = DID::build(ETHR_DID_METHOD, None, identity.as_ref());

        // Resole DID document
        let did_doc_with_meta = did_resolver::resolve_did(&client, &did, None)
            .await
            .unwrap();
        let did_document = did_doc_with_meta.did_document.unwrap();

        // DID Document is empty
        assert_eq!(
            default_ethr_did_document(identity.as_ref(), Some(client.chain_id())),
            did_document
        );
    }
}

mod schema {
    use super::*;
    use crate::{schema_registry, Schema};

    pub(crate) async fn endorse_schema(
        client: &LedgerClient,
        did: &DID,
        signer: &BasicSigner,
    ) -> Schema {
        let schema = schema(did, None);
        let transaction_endorsing_data =
            schema_registry::build_create_schema_endorsing_data(client, &schema)
                .await
                .unwrap();

        super::helpers::endorse_transaction(client, signer, transaction_endorsing_data).await;

        schema
    }

    #[async_std::test]
    async fn demo_create_schema() {
        let signer = basic_signer();
        let client = client();

        // create DID
        let did = DID::build(ETHR_DID_METHOD, None, TRUSTEE_ACCOUNT.as_ref());

        // write
        let schema = schema(&did, None);
        let transaction = schema_registry::build_create_schema_transaction(
            &client,
            &TRUSTEE_ACCOUNT.clone(),
            &schema,
        )
        .await
        .unwrap();
        super::helpers::sign_and_submit_transaction(&client, transaction, &signer).await;

        // read
        let resolved_schema = schema_registry::resolve_schema(&client, &schema.id())
            .await
            .unwrap();
        assert_eq!(schema, resolved_schema);
    }

    #[async_std::test]
    async fn demo_endorse_schema() {
        let mut signer = basic_signer();
        let client = client();
        let (identity, _) = signer.create_key(None).unwrap();

        // create DID
        let did = DID::build(ETHR_DID_METHOD, None, identity.as_ref());

        // endorse schema
        let schema = endorse_schema(&client, &did, &signer).await;

        // read
        let resolved_schema = schema_registry::resolve_schema(&client, &schema.id())
            .await
            .unwrap();
        assert_eq!(schema, resolved_schema);
    }
}

mod credential_definition {
    use super::*;
    use crate::{credential_definition_registry, schema_registry, CredentialDefinition, Schema};

    pub(crate) async fn endorse_credential_definition(
        client: &LedgerClient,
        did: &DID,
        schema: &Schema,
        signer: &BasicSigner,
    ) -> CredentialDefinition {
        let credential_definition = credential_definition(&did, &schema.id(), None);
        let transaction_endorsing_data =
            credential_definition_registry::build_create_credential_definition_endorsing_data(
                &client,
                &credential_definition,
            )
            .await
            .unwrap();

        super::helpers::endorse_transaction(&client, &signer, transaction_endorsing_data).await;

        credential_definition
    }

    #[async_std::test]
    async fn demo_create_credential_definition() {
        let signer = basic_signer();
        let client = client();

        // create DID
        let did = DID::build(ETHR_DID_METHOD, None, TRUSTEE_ACCOUNT.as_ref());

        // create Schema
        let schema = schema(&did, None);
        let transaction =
            schema_registry::build_create_schema_transaction(&client, &TRUSTEE_ACCOUNT, &schema)
                .await
                .unwrap();
        super::helpers::sign_and_submit_transaction(&client, transaction, &signer).await;

        // write
        let credential_definition = credential_definition(&did, &schema.id(), None);
        let transaction =
            credential_definition_registry::build_create_credential_definition_transaction(
                &client,
                &TRUSTEE_ACCOUNT,
                &credential_definition,
            )
            .await
            .unwrap();
        super::helpers::sign_and_submit_transaction(&client, transaction, &signer).await;

        // read
        let resolved_credential_definition =
            credential_definition_registry::resolve_credential_definition(
                &client,
                &credential_definition.id(),
            )
            .await
            .unwrap();
        assert_eq!(credential_definition, resolved_credential_definition);
    }

    #[async_std::test]
    async fn demo_endorse_credential_definition() {
        let mut signer = basic_signer();
        let client = client();
        let (identity, _) = signer.create_key(None).unwrap();

        // create DID Document
        let did = DID::build(ETHR_DID_METHOD, None, identity.as_ref());

        // create Schema
        let schema = super::schema::endorse_schema(&client, &did, &signer).await;

        // write
        let credential_definition =
            endorse_credential_definition(&client, &did, &schema, &signer).await;

        // read
        let resolved_credential_definition =
            credential_definition_registry::resolve_credential_definition(
                &client,
                &credential_definition.id(),
            )
            .await
            .unwrap();
        assert_eq!(credential_definition, resolved_credential_definition);
    }
}

mod revocation_registry_definition {
    use super::*;
    use crate::{
        credential_definition_registry, revocation_registry, schema_registry, CredentialDefinition,
        RevocationRegistryDefinition,
    };

    pub(crate) async fn endorse_revocation_registry_definition(
        client: &LedgerClient,
        did: &DID,
        credential_definition: &CredentialDefinition,
        signer: &BasicSigner,
    ) -> RevocationRegistryDefinition {
        let revocation_registry_definition =
            revocation_registry_definition(&did, &credential_definition.id(), None);
        let transaction_endorsing_data =
            revocation_registry::build_create_revocation_registry_definition_endorsing_data(
                &client,
                &revocation_registry_definition,
            )
            .await
            .unwrap();

        super::helpers::endorse_transaction(&client, &signer, transaction_endorsing_data).await;

        revocation_registry_definition
    }

    #[async_std::test]
    async fn demo_create_revocation_registry() {
        let signer = basic_signer();
        let client = client();

        // create DID
        let did = DID::build(ETHR_DID_METHOD, None, TRUSTEE_ACCOUNT.as_ref());

        // create Schema
        let schema = schema(&did, None);
        let transaction =
            schema_registry::build_create_schema_transaction(&client, &TRUSTEE_ACCOUNT, &schema)
                .await
                .unwrap();
        super::helpers::sign_and_submit_transaction(&client, transaction, &signer).await;

        // create Credential Definition
        let credential_definition = credential_definition(&did, &schema.id(), None);
        let transaction =
            credential_definition_registry::build_create_credential_definition_transaction(
                &client,
                &TRUSTEE_ACCOUNT,
                &credential_definition,
            )
            .await
            .unwrap();
        super::helpers::sign_and_submit_transaction(&client, transaction, &signer).await;

        // write
        let revocation_registry =
            revocation_registry_definition(&did, &credential_definition.id(), None);
        let transaction =
            revocation_registry::build_create_revocation_registry_definition_transaction(
                &client,
                &TRUSTEE_ACCOUNT,
                &revocation_registry,
            )
            .await
            .unwrap();
        super::helpers::sign_and_submit_transaction(&client, transaction, &signer).await;

        // read
        let resolved_revocation_registry =
            revocation_registry::resolve_revocation_registry_definition(
                &client,
                &revocation_registry.id(),
            )
            .await
            .unwrap();
        assert_eq!(revocation_registry, resolved_revocation_registry);
    }

    #[async_std::test]
    async fn demo_endorse_revocation_registry() {
        let mut signer = basic_signer();
        let client = client();
        let (identity, _) = signer.create_key(None).unwrap();

        // create DID Document
        let did = DID::build(ETHR_DID_METHOD, None, identity.as_ref());

        // create Schema
        let schema = super::schema::endorse_schema(&client, &did, &signer).await;

        // create Credential Definition
        let credential_definition = super::credential_definition::endorse_credential_definition(
            &client, &did, &schema, &signer,
        )
        .await;

        // write
        let revocation_registry_definition =
            endorse_revocation_registry_definition(&client, &did, &credential_definition, &signer)
                .await;

        // read
        let resolved_revocation_registry =
            revocation_registry::resolve_revocation_registry_definition(
                &client,
                &revocation_registry_definition.id(),
            )
            .await
            .unwrap();
        assert_eq!(revocation_registry_definition, resolved_revocation_registry);
    }
}

mod revocation_registry_entry {
    use super::*;
    use crate::{
        credential_definition_registry, revocation_registry, schema_registry, Accumulator,
        RevocationRegistryDefinition, RevocationRegistryEntry,
    };

    pub(crate) async fn endorse_revocation_registry_entry(
        client: &LedgerClient,
        did: &DID,
        revocation_registry_definition: &RevocationRegistryDefinition,
        signer: &BasicSigner,
    ) -> RevocationRegistryEntry {
        let revocation_registry_entry =
            revocation_registry_entry(&did, &revocation_registry_definition.id(), None, None, None);
        let transaction_endorsing_data =
            revocation_registry::build_create_revocation_registry_entry_endorsing_data(
                &client,
                &revocation_registry_entry,
            )
            .await
            .unwrap();

        super::helpers::endorse_transaction(&client, &signer, transaction_endorsing_data).await;

        revocation_registry_entry
    }

    #[async_std::test]
    async fn demo_create_revocation_registry_entry() {
        let signer = basic_signer();
        let client = client();

        // create DID
        let did = DID::build(ETHR_DID_METHOD, None, TRUSTEE_ACCOUNT.as_ref());

        // create Schema
        let schema = schema(&did, None);
        let transaction =
            schema_registry::build_create_schema_transaction(&client, &TRUSTEE_ACCOUNT, &schema)
                .await
                .unwrap();
        super::helpers::sign_and_submit_transaction(&client, transaction, &signer).await;

        // create Credential Definition
        let credential_definition = credential_definition(&did, &schema.id(), None);
        let transaction =
            credential_definition_registry::build_create_credential_definition_transaction(
                &client,
                &TRUSTEE_ACCOUNT,
                &credential_definition,
            )
            .await
            .unwrap();

        super::helpers::sign_and_submit_transaction(&client, transaction, &signer).await;

        // create Revocation Registry Definition
        let revocation_registry_definition =
            revocation_registry_definition(&did, &credential_definition.id(), None);
        let transaction =
            revocation_registry::build_create_revocation_registry_definition_transaction(
                &client,
                &TRUSTEE_ACCOUNT,
                &revocation_registry_definition,
            )
            .await
            .unwrap();
        super::helpers::sign_and_submit_transaction(&client, transaction, &signer).await;

        // write
        let revocation_registry_entry1 = revocation_registry_entry(
            &did,
            &revocation_registry_definition.id(),
            None,
            Some("currentAccum"),
            None,
        );
        let transaction = revocation_registry::build_create_revocation_registry_entry_transaction(
            &client,
            &TRUSTEE_ACCOUNT,
            &revocation_registry_entry1,
        )
        .await
        .unwrap();
        super::helpers::sign_and_submit_transaction(&client, transaction, &signer).await;
        let past_time = chrono::Utc::now().timestamp() as u64;

        // write 2
        let revocation_registry_entry2 = revocation_registry_entry(
            &did,
            &revocation_registry_definition.id(),
            Some(vec![10, 11, 12]),
            Some("prevAccum"),
            Some("currentAccum"),
        );
        let transaction = revocation_registry::build_create_revocation_registry_entry_transaction(
            &client,
            &TRUSTEE_ACCOUNT,
            &revocation_registry_entry2,
        )
        .await
        .unwrap();
        super::helpers::sign_and_submit_transaction(&client, transaction, &signer).await;

        // read
        let revocation_status_list1 = revocation_registry::resolve_revocation_registry_status_list(
            &client,
            &revocation_registry_definition.id(),
            chrono::Utc::now().timestamp() as u64,
        )
        .await
        .unwrap();

        // read 2
        let revocation_status_list2 = revocation_registry::resolve_revocation_registry_status_list(
            &client,
            &revocation_registry_definition.id(),
            past_time,
        )
        .await
        .unwrap();

        let mut status_list1 = vec![
            0;
            revocation_registry_definition
                .value
                .max_cred_num
                .try_into()
                .unwrap()
        ];
        let mut status_list2 = vec![
            0;
            revocation_registry_definition
                .value
                .max_cred_num
                .try_into()
                .unwrap()
        ];
        for list1 in revocation_registry_entry1
            .rev_reg_entry_data
            .revoked
            .unwrap_or(vec![])
            .clone()
        {
            status_list1[list1 as usize] = 1;
            status_list2[list1 as usize] = 1;
        }
        for list2 in revocation_registry_entry2
            .rev_reg_entry_data
            .revoked
            .unwrap_or(vec![])
            .clone()
        {
            status_list2[list2 as usize] = 1;
        }

        assert_eq!(revocation_status_list1.revocation_list, status_list2);
        assert_eq!(revocation_status_list2.revocation_list, status_list1);
        assert_eq!(
            revocation_registry_entry1
                .rev_reg_entry_data
                .current_accumulator,
            Accumulator::try_from(revocation_status_list2.current_accumulator.as_str()).unwrap()
        );
        assert_eq!(
            revocation_registry_entry1.issuer_id,
            revocation_status_list1.issuer_id
        );
        assert_eq!(
            revocation_registry_entry1.rev_reg_def_id,
            revocation_status_list1.rev_reg_def_id
        );
    }

    #[async_std::test]
    async fn demo_endorse_revocation_registry_entry() {
        let mut signer = basic_signer();
        let client = client();
        let (identity, _) = signer.create_key(None).unwrap();

        // create DID Document
        let did = DID::build(ETHR_DID_METHOD, None, identity.as_ref());

        // create Schema
        let schema = super::schema::endorse_schema(&client, &did, &signer).await;

        // create Credential Definition
        let credential_definition = super::credential_definition::endorse_credential_definition(
            &client, &did, &schema, &signer,
        )
        .await;

        // create Revocation Registry Definition
        let revocation_registry_definition =
            super::revocation_registry_definition::endorse_revocation_registry_definition(
                &client,
                &did,
                &credential_definition,
                &signer,
            )
            .await;

        // write
        let revocation_registry_entry = endorse_revocation_registry_entry(
            &client,
            &did,
            &revocation_registry_definition,
            &signer,
        )
        .await;

        // read
        let revocation_status_list = revocation_registry::resolve_revocation_registry_status_list(
            &client,
            &revocation_registry_entry.rev_reg_def_id,
            chrono::Utc::now().timestamp() as u64,
        )
        .await
        .unwrap();

        // assert_eq!(revocation_registry_entry.rev_reg_entry_data.current_accumulator, Accumulator::try_from(revocation_status_list.current_accumulator.as_str()).unwrap());
        // assert_eq!(revocation_registry_entry.issuer_id, revocation_status_list.issuer_id);
        assert_eq!(
            revocation_registry_entry.rev_reg_def_id,
            revocation_status_list.rev_reg_def_id
        );
    }
}

mod role {
    use super::*;
    use crate::role_control;

    async fn revoke_role(
        client: &LedgerClient,
        revokee_account: &Address,
        role_to_revoke: &Role,
        signer: &BasicSigner,
    ) -> String {
        let mut transaction = role_control::build_revoke_role_transaction(
            client,
            &TRUSTEE_ACCOUNT,
            role_to_revoke,
            revokee_account,
        )
        .await
        .unwrap();

        let sign_bytes = transaction.get_signing_bytes().unwrap();
        let signature = signer.sign(&sign_bytes, TRUSTEE_ACCOUNT.as_ref()).unwrap();
        transaction.set_signature(signature);

        let block_hash = client.submit_transaction(&transaction).await.unwrap();

        client.get_receipt(&block_hash).await.unwrap()
    }

    async fn build_and_submit_get_role_transaction(
        client: &LedgerClient,
        assignee_account: &Address,
    ) -> Role {
        let transaction = role_control::build_get_role_transaction(client, assignee_account)
            .await
            .unwrap();
        let result = client.submit_transaction(&transaction).await.unwrap();
        role_control::parse_get_role_result(&client, &result).unwrap()
    }

    async fn build_and_submit_has_role_transaction(
        client: &LedgerClient,
        role: &Role,
        assignee_account: &Address,
    ) -> bool {
        let transaction = role_control::build_has_role_transaction(client, role, assignee_account)
            .await
            .unwrap();
        let result = client.submit_transaction(&transaction).await.unwrap();
        role_control::parse_has_role_result(&client, &result).unwrap()
    }

    #[async_std::test]
    async fn demo_build_and_submit_assign_and_remove_role_transactions_test() {
        let signer = basic_signer();
        let (assignee_account, _) = signer.create_account(None).unwrap();
        let client = client();
        let role_to_assign = Role::Endorser;

        super::helpers::assign_role(&client, &assignee_account, &role_to_assign, &signer).await;

        let assigned_role = build_and_submit_get_role_transaction(&client, &assignee_account).await;
        assert_eq!(role_to_assign, assigned_role);

        revoke_role(&client, &assignee_account, &role_to_assign, &signer).await;

        let has_role =
            build_and_submit_has_role_transaction(&client, &role_to_assign, &assignee_account)
                .await;
        assert!(!has_role);
    }
}

mod validator {
    use crate::{
        contracts::network::ValidatorAddresses, signer::basic_signer::test::basic_signer,
        validator_control,
    };

    use super::*;

    async fn build_and_submit_get_validators_transaction(
        client: &LedgerClient,
    ) -> ValidatorAddresses {
        let transaction = validator_control::build_get_validators_transaction(&client)
            .await
            .unwrap();
        let result = client.submit_transaction(&transaction).await.unwrap();

        validator_control::parse_get_validators_result(&client, &result).unwrap()
    }

    async fn build_and_submit_add_validator_transaction(
        client: &LedgerClient,
        new_validator_address: &Address,
        signer: &BasicSigner,
    ) -> String {
        let transaction = validator_control::build_add_validator_transaction(
            &client,
            &TRUSTEE_ACCOUNT,
            new_validator_address,
        )
        .await
        .unwrap();
        super::helpers::sign_and_submit_transaction(client, transaction, signer).await
    }

    async fn build_and_submit_remove_validator_transaction(
        client: &LedgerClient,
        validator_address: &Address,
        signer: &BasicSigner,
    ) -> String {
        // write
        let transaction = validator_control::build_remove_validator_transaction(
            &client,
            &TRUSTEE_ACCOUNT,
            validator_address,
        )
        .await
        .unwrap();
        super::helpers::sign_and_submit_transaction(client, transaction, signer).await
    }

    #[async_std::test]
    async fn demo_build_and_submit_transaction_test() {
        let signer = basic_signer();
        let (new_validator_address, _) = signer.create_account(None).unwrap();
        let client = client();
        super::helpers::assign_role(&client, &TRUSTEE_ACCOUNT, &Role::Steward, &signer).await;

        build_and_submit_add_validator_transaction(&client, &new_validator_address, &signer).await;

        let validator_list = build_and_submit_get_validators_transaction(&client).await;
        assert_eq!(validator_list.len(), 5);
        assert!(validator_list.contains(&new_validator_address));

        build_and_submit_remove_validator_transaction(&client, &new_validator_address, &signer)
            .await;

        let validator_list = build_and_submit_get_validators_transaction(&client).await;
        assert_eq!(validator_list.len(), 4);
        assert!(!validator_list.contains(&new_validator_address));
    }
}

mod mapping {
    use super::*;
    use crate::{
        client::client::test::client,
        contracts::{
            anoncreds::types::schema::test::{SCHEMA_NAME, SCHEMA_VERSION},
            migration::types::did::{LegacyDid, LegacyVerkey},
        },
        legacy_mapping_registry, Ed25519Signature, ResourceIdentifier, SchemaId,
    };
    use ed25519_dalek::{SigningKey, VerifyingKey};
    use indy_data_types::did::DidValue;
    use rand::rngs::OsRng;

    fn generate_legacy_did() -> (LegacyDid, LegacyVerkey, SigningKey) {
        let mut csprng = OsRng;
        let signing_key: SigningKey = SigningKey::generate(&mut csprng);
        let verifying_key: VerifyingKey = signing_key.verifying_key();
        let did = bs58::encode(&verifying_key.as_bytes()[..16]).into_string();
        let verkey = bs58::encode(&verifying_key).into_string();
        (
            LegacyDid::from(did.as_str()),
            LegacyVerkey::from(verkey.as_str()),
            signing_key,
        )
    }

    #[async_std::test]
    async fn demo_create_mappings() {
        let signer = basic_signer();
        let client = client();

        let did = DID::build(ETHR_DID_METHOD, None, TRUSTEE_ACCOUNT.as_ref());
        let (legacy_did, legacy_verkey, _) = generate_legacy_did();
        let legacy_signature = Ed25519Signature::from(vec![1, 2, 3, 4, 5, 6].as_slice());

        // create DID mapping
        let transaction = legacy_mapping_registry::build_create_did_mapping_transaction(
            &client,
            &TRUSTEE_ACCOUNT.clone(),
            &did,
            &legacy_did,
            &legacy_verkey,
            &legacy_signature,
        )
        .await
        .unwrap();
        super::helpers::sign_and_submit_transaction(&client, transaction, &signer).await;

        // read DID mapping
        let transaction =
            legacy_mapping_registry::build_get_did_mapping_transaction(&client, &legacy_did)
                .await
                .unwrap();
        let response = client.submit_transaction(&transaction).await.unwrap();
        let resolved_did =
            legacy_mapping_registry::parse_did_mapping_result(&client, &response).unwrap();
        assert_eq!(did, resolved_did);

        // create mapping for schema id
        let legacy_schema_id = indy_data_types::SchemaId::new(
            &DidValue(legacy_did.to_string()),
            SCHEMA_NAME,
            SCHEMA_VERSION,
        );
        let legacy_schema_id = ResourceIdentifier::from(&legacy_schema_id);
        let schema_id = SchemaId::build(&did, SCHEMA_NAME, SCHEMA_VERSION);
        let schema_id = ResourceIdentifier::from(&schema_id);

        let transaction = legacy_mapping_registry::build_create_resource_mapping_transaction(
            &client,
            &TRUSTEE_ACCOUNT.clone(),
            &did,
            &legacy_did,
            &legacy_schema_id,
            &schema_id,
        )
        .await
        .unwrap();
        super::helpers::sign_and_submit_transaction(&client, transaction, &signer).await;

        // read schema mapping
        let transaction = legacy_mapping_registry::build_get_resource_mapping_transaction(
            &client,
            &legacy_schema_id,
        )
        .await
        .unwrap();
        let response = client.submit_transaction(&transaction).await.unwrap();
        let resolved_schema_id =
            legacy_mapping_registry::parse_resource_mapping_result(&client, &response).unwrap();
        assert_eq!(schema_id, resolved_schema_id);
    }

    #[async_std::test]
    async fn demo_endorse_mappings() {
        let mut signer = basic_signer();
        let client = client();
        let (identity, _) = signer.create_key(None).unwrap();

        let did = DID::build(ETHR_DID_METHOD, None, identity.as_ref());
        let (legacy_did, legacy_verkey, _) = generate_legacy_did();
        let legacy_signature = Ed25519Signature::from(vec![1, 2, 3, 4, 5, 6].as_slice());

        // endorse DID mapping
        let transaction_endorsing_data =
            legacy_mapping_registry::build_create_did_mapping_endorsing_data(
                &client,
                &did,
                &legacy_did,
                &legacy_verkey,
                &legacy_signature,
            )
            .await
            .unwrap();

        super::helpers::endorse_transaction(&client, &signer, transaction_endorsing_data).await;

        // read DID mapping
        let transaction =
            legacy_mapping_registry::build_get_did_mapping_transaction(&client, &legacy_did)
                .await
                .unwrap();
        let response = client.submit_transaction(&transaction).await.unwrap();
        let resolved_did =
            legacy_mapping_registry::parse_did_mapping_result(&client, &response).unwrap();
        assert_eq!(did, resolved_did);

        // endorse mapping for schema id
        let legacy_schema_id = indy_data_types::SchemaId::new(
            &DidValue(legacy_did.to_string()),
            SCHEMA_NAME,
            SCHEMA_VERSION,
        );
        let legacy_schema_id = ResourceIdentifier::from(&legacy_schema_id);
        let schema_id = SchemaId::build(&did, SCHEMA_NAME, SCHEMA_VERSION);
        let schema_id = ResourceIdentifier::from(&schema_id);

        let transaction_endorsing_data =
            legacy_mapping_registry::build_create_resource_mapping_endorsing_data(
                &client,
                &did,
                &legacy_did,
                &legacy_schema_id,
                &schema_id,
            )
            .await
            .unwrap();

        super::helpers::endorse_transaction(&client, &signer, transaction_endorsing_data).await;

        // read schema mapping
        let transaction = legacy_mapping_registry::build_get_resource_mapping_transaction(
            &client,
            &legacy_schema_id,
        )
        .await
        .unwrap();
        let response = client.submit_transaction(&transaction).await.unwrap();
        let resolved_schema_id =
            legacy_mapping_registry::parse_resource_mapping_result(&client, &response).unwrap();
        assert_eq!(schema_id, resolved_schema_id);
    }
}
