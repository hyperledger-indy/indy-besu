// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use log_derive::{logfn, logfn_inputs};

use crate::{
    client::LedgerClient,
    contracts::migration::types::{
        did::{LegacyDid, LegacyVerkey},
        ed25519_signature::Ed25519Signature,
        resource_identifier::ResourceIdentifier,
    },
    error::VdrResult,
    types::{
        Address, Transaction, TransactionBuilder, TransactionEndorsingDataBuilder,
        TransactionParser, TransactionType,
    },
    TransactionEndorsingData, DID,
};

const CONTRACT_NAME: &str = "LegacyMappingRegistry";
const METHOD_CREATE_DID_MAPPING: &str = "createDidMapping";
const METHOD_CREATE_DID_MAPPING_SIGNED: &str = "createDidMappingSigned";
const METHOD_CREATE_RESOURCE_MAPPING: &str = "createResourceMapping";
const METHOD_CREATE_RESOURCE_MAPPING_SIGNED: &str = "createResourceMappingSigned";
const METHOD_DID_MAPPING: &str = "didMapping";
const METHOD_RESOURCE_MAPPING: &str = "resourceMapping";

/// Build a transaction to create a legacy DID identifier mapping
///     (LegacyMappingRegistry.createDidMapping contract method)
///
/// # Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `from`: [Address] - transaction sender account address (new account)
/// - `did`: [DID] - new DID
/// - `legacy_identifier`: [LegacyDid] - identifier of legacy sov/indy DID
/// - `legacy_verkey`: [LegacyVerkey] - Ed25519 verification key of the legacy DID identifier.
/// - `ed25519_signature`: [Ed25519Signature] - ED25519 signature to prove key possession.
///
/// # Returns
///   transaction: [Transaction] - prepared write transaction object to sign and submit
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_create_did_mapping_transaction(
    client: &LedgerClient,
    from: &Address,
    did: &DID,
    legacy_identifier: &LegacyDid,
    legacy_verkey: &LegacyVerkey,
    ed25519_signature: &Ed25519Signature,
) -> VdrResult<Transaction> {
    let identity = Address::try_from(did)?;
    TransactionBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_CREATE_DID_MAPPING)
        .add_param(&identity)?
        .add_param(legacy_identifier)?
        .add_param(&did.without_network()?)?
        .add_param(legacy_verkey)?
        .add_param(ed25519_signature)?
        .set_type(TransactionType::Write)
        .set_from(from)
        .build(client)
        .await
}

/// Prepared data for endorsing creation of a legacy DID identifier mapping
///     (LegacyMappingRegistry.createDidMappingSigned contract method)
///
/// #Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `did`: [DID] - new DID
/// - `legacy_identifier`: [LegacyDid] - identifier of legacy sov/indy DID
/// - `legacy_verkey`: [LegacyVerkey] - Ed25519 verification key of the legacy DID identifier.
/// - `ed25519_signature`: [Ed25519Signature] - ED25519 signature to prove key possession.
///
/// #Returns
///   data: [TransactionEndorsingData] - transaction endorsement data to sign
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_create_did_mapping_endorsing_data(
    client: &LedgerClient,
    did: &DID,
    legacy_identifier: &LegacyDid,
    legacy_verkey: &LegacyVerkey,
    ed25519_signature: &Ed25519Signature,
) -> VdrResult<TransactionEndorsingData> {
    TransactionEndorsingDataBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_identity(&Address::try_from(did)?)
        .set_method(METHOD_CREATE_DID_MAPPING)
        .set_endorsing_method(METHOD_CREATE_DID_MAPPING_SIGNED)
        .add_param(legacy_identifier)?
        .add_param(&did.without_network()?)?
        .add_param(legacy_verkey)?
        .add_param(ed25519_signature)?
        .build(client)
        .await
}

/// Build a transaction to resolve new identity DID for the given legacy DID identifier
///  (LegacyMappingRegistry.didMapping contract method)
///
/// #Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `legacy_identifier`: [LegacyDid] - identifier of legacy sov/indy DID
///
/// #Returns
///   transaction: [Transaction] - prepared read transaction object to submit
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_get_did_mapping_transaction(
    client: &LedgerClient,
    legacy_identifier: &LegacyDid,
) -> VdrResult<Transaction> {
    TransactionBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_DID_MAPPING)
        .add_param(legacy_identifier)?
        .set_type(TransactionType::Read)
        .build(client)
        .await
}

/// Parse the result of execution LegacyMappingRegistry.didMapping contract method to receive
///   new identity DID for legacy DID identifier
///
/// # Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `bytes`: [Vec] - result bytes returned from the ledger
///
/// # Returns
///   Identity DID
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub fn parse_did_mapping_result(client: &LedgerClient, bytes: &[u8]) -> VdrResult<DID> {
    TransactionParser::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_DID_MAPPING)
        .parse::<DID>(client, bytes)
}

/// Build a transaction to create a mapping of legacy schema/credential definition identifier to new one
///  (LegacyMappingRegistry.createResourceMapping contract method)
///
/// # Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `from`: [Address] - transaction sender account address (new account)
/// - `did`: [DID] - new DID
/// - `legacy_issuer_identifier`: [LegacyDid] - identifier of legacy sov/indy DID
/// - `legacy_identifier`: [ResourceIdentifier] - legacy identifier.
/// - `new_identifier`: [ResourceIdentifier] - new identifier.
///
/// # Returns
///   transaction: [Transaction] - prepared write transaction object to sign and submit
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_create_resource_mapping_transaction(
    client: &LedgerClient,
    from: &Address,
    did: &DID,
    legacy_issuer_identifier: &LegacyDid,
    legacy_identifier: &ResourceIdentifier,
    new_identifier: &ResourceIdentifier,
) -> VdrResult<Transaction> {
    let identity = Address::try_from(did)?;
    TransactionBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_CREATE_RESOURCE_MAPPING)
        .add_param(&identity)?
        .add_param(legacy_issuer_identifier)?
        .add_param(legacy_identifier)?
        .add_param(new_identifier)?
        .set_type(TransactionType::Write)
        .set_from(from)
        .build(client)
        .await
}

/// Prepared data for endorsing creation of a new resource mapping
///     (LegacyMappingRegistry.createResourceMappingSigned contract method)
///
/// #Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `did`: [DID] - new DID
/// - `legacy_issuer_identifier`: [LegacyDid] - identifier of legacy sov/indy DID
/// - `legacy_identifier`: [ResourceIdentifier] - legacy identifier.
/// - `new_identifier`: [ResourceIdentifier] - new identifier.
///
/// #Returns
///   data: [TransactionEndorsingData] - transaction endorsement data to sign
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_create_resource_mapping_endorsing_data(
    client: &LedgerClient,
    did: &DID,
    legacy_issuer_identifier: &LegacyDid,
    legacy_identifier: &ResourceIdentifier,
    new_identifier: &ResourceIdentifier,
) -> VdrResult<TransactionEndorsingData> {
    TransactionEndorsingDataBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_identity(&Address::try_from(did)?)
        .set_method(METHOD_CREATE_RESOURCE_MAPPING)
        .set_endorsing_method(METHOD_CREATE_RESOURCE_MAPPING_SIGNED)
        .add_param(legacy_issuer_identifier)?
        .add_param(legacy_identifier)?
        .add_param(new_identifier)?
        .build(client)
        .await
}

/// Build a transaction to resolve new identifier for the given legacy Schema/CredentialDefinition ID
///  (LegacyMappingRegistry.resourceMapping contract method)
///
/// #Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `legacy_identifier`: [ResourceIdentifier] - identifier of legacy Schema/CredentialDefinition
///
/// #Returns
///   transaction: [Transaction] - prepared read transaction object to submit
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_get_resource_mapping_transaction(
    client: &LedgerClient,
    legacy_identifier: &ResourceIdentifier,
) -> VdrResult<Transaction> {
    TransactionBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_RESOURCE_MAPPING)
        .add_param(legacy_identifier)?
        .set_type(TransactionType::Read)
        .build(client)
        .await
}

/// Parse the result of execution LegacyMappingRegistry.resourceMapping contract method to receive
///   new identifier for a legacy Schema/CredentialDefinition
///
/// # Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `bytes`: [Vec] - result bytes returned from the ledger
///
/// # Returns
///   New identifier
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub fn parse_resource_mapping_result(
    client: &LedgerClient,
    bytes: &[u8],
) -> VdrResult<ResourceIdentifier> {
    TransactionParser::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_RESOURCE_MAPPING)
        .parse::<ResourceIdentifier>(client, bytes)
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::{
        client::client::test::{mock_client, CONFIG, DEFAULT_NONCE, TEST_ACCOUNT},
        contracts::{did::types::did::DID, types::did_doc::test::TEST_ETHR_DID},
    };

    const LEGACY_DID: &str = "VsKV7grR1BUE29mG2Fm2kX";
    const LEGACY_VERKEY: &str = "GjZWsBLgZCR18aL468JAT7w9CZRiBnpxUPPgyQxh4voa";
    const LEGACY_SCHEMA_ID: &str = "VsKV7grR1BUE29mG2Fm2kX:2:test_credential:1.0.0";
    const LEGACY_SIGNATURE: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];
    const NEW_SCHEMA_ID: &str = "did:ethr:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5/anoncreds/v0/SCHEMA/test_credential/1.0.0";

    mod build_create_did_mapping_transaction {
        use super::*;

        #[async_std::test]
        async fn build_create_did_mapping_transaction_test() {
            let client = mock_client();
            let transaction = build_create_did_mapping_transaction(
                &client,
                &TEST_ACCOUNT,
                &DID::from(TEST_ETHR_DID),
                &LegacyDid::from(LEGACY_DID),
                &LegacyVerkey::from(LEGACY_VERKEY),
                &Ed25519Signature::from(LEGACY_SIGNATURE.as_slice()),
            )
            .await
            .unwrap();
            let expected_transaction = Transaction {
                type_: TransactionType::Write,
                from: Some(TEST_ACCOUNT.clone()),
                to: CONFIG.contracts.legacy_mapping_registry.address.clone(),
                nonce: Some(DEFAULT_NONCE.clone()),
                chain_id: CONFIG.chain_id,
                data: vec![
                    154, 50, 101, 217, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 240, 226, 219, 108, 141,
                    198, 198, 129, 187, 93, 106, 209, 33, 161, 7, 243, 0, 233, 178, 181, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 160, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 224, 233, 198, 118, 109, 146, 166, 42, 37, 34, 23, 211,
                    11, 40, 37, 124, 32, 134, 8, 18, 195, 139, 30, 194, 70, 230, 160, 73, 245, 42,
                    208, 96, 25, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 1, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 22, 86, 115, 75, 86, 55, 103, 114,
                    82, 49, 66, 85, 69, 50, 57, 109, 71, 50, 70, 109, 50, 107, 88, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 51, 100, 105, 100, 58, 101, 116, 104, 114, 58,
                    48, 120, 102, 48, 101, 50, 100, 98, 54, 99, 56, 100, 99, 54, 99, 54, 56, 49,
                    98, 98, 53, 100, 54, 97, 100, 49, 50, 49, 97, 49, 48, 55, 102, 51, 48, 48, 101,
                    57, 98, 50, 98, 53, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0,
                    1, 2, 3, 4, 5, 6, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0,
                ],
                signature: None,
                hash: None,
            };
            assert_eq!(expected_transaction, transaction);
        }
    }

    mod build_get_did_mapping_transaction {
        use super::*;

        #[async_std::test]
        async fn build_get_did_mapping_transaction_test() {
            let client = mock_client();
            let transaction =
                build_get_did_mapping_transaction(&client, &LegacyDid::from(LEGACY_DID))
                    .await
                    .unwrap();
            let expected_transaction = Transaction {
                type_: TransactionType::Read,
                from: None,
                to: CONFIG.contracts.legacy_mapping_registry.address.clone(),
                nonce: None,
                chain_id: CONFIG.chain_id,
                data: vec![
                    147, 168, 199, 66, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 22, 86, 115, 75, 86, 55,
                    103, 114, 82, 49, 66, 85, 69, 50, 57, 109, 71, 50, 70, 109, 50, 107, 88, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0,
                ],
                signature: None,
                hash: None,
            };
            assert_eq!(expected_transaction, transaction);
        }
    }

    mod build_create_resource_mapping_transaction {
        use super::*;

        #[async_std::test]
        async fn build_create_resource_mapping_transaction_test() {
            let client = mock_client();
            let transaction = build_create_resource_mapping_transaction(
                &client,
                &TEST_ACCOUNT,
                &DID::from(TEST_ETHR_DID),
                &LegacyDid::from(LEGACY_DID),
                &ResourceIdentifier::from(LEGACY_SCHEMA_ID),
                &ResourceIdentifier::from(NEW_SCHEMA_ID),
            )
            .await
            .unwrap();
            let expected_transaction = Transaction {
                type_: TransactionType::Write,
                from: Some(TEST_ACCOUNT.clone()),
                to: CONFIG.contracts.legacy_mapping_registry.address.clone(),
                nonce: Some(DEFAULT_NONCE.clone()),
                chain_id: CONFIG.chain_id,
                data: vec![
                    217, 36, 174, 219, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 240, 226, 219, 108, 141,
                    198, 198, 129, 187, 93, 106, 209, 33, 161, 7, 243, 0, 233, 178, 181, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 192, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 22, 86, 115, 75,
                    86, 55, 103, 114, 82, 49, 66, 85, 69, 50, 57, 109, 71, 50, 70, 109, 50, 107,
                    88, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 46, 86, 115, 75, 86, 55, 103,
                    114, 82, 49, 66, 85, 69, 50, 57, 109, 71, 50, 70, 109, 50, 107, 88, 58, 50, 58,
                    116, 101, 115, 116, 95, 99, 114, 101, 100, 101, 110, 116, 105, 97, 108, 58, 49,
                    46, 48, 46, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 93, 100, 105, 100, 58, 101, 116, 104, 114, 58, 48, 120, 102, 48, 101, 50,
                    100, 98, 54, 99, 56, 100, 99, 54, 99, 54, 56, 49, 98, 98, 53, 100, 54, 97, 100,
                    49, 50, 49, 97, 49, 48, 55, 102, 51, 48, 48, 101, 57, 98, 50, 98, 53, 47, 97,
                    110, 111, 110, 99, 114, 101, 100, 115, 47, 118, 48, 47, 83, 67, 72, 69, 77, 65,
                    47, 116, 101, 115, 116, 95, 99, 114, 101, 100, 101, 110, 116, 105, 97, 108, 47,
                    49, 46, 48, 46, 48, 0, 0, 0,
                ],
                signature: None,
                hash: None,
            };
            assert_eq!(expected_transaction, transaction);
        }
    }

    mod build_get_resource_mapping_transaction {
        use super::*;

        #[async_std::test]
        async fn build_get_resource_mapping_transaction_test() {
            let client = mock_client();
            let transaction = build_get_resource_mapping_transaction(
                &client,
                &ResourceIdentifier::from(LEGACY_SCHEMA_ID),
            )
            .await
            .unwrap();
            let expected_transaction = Transaction {
                type_: TransactionType::Read,
                from: None,
                to: CONFIG.contracts.legacy_mapping_registry.address.clone(),
                nonce: None,
                chain_id: CONFIG.chain_id,
                data: vec![
                    198, 18, 171, 88, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 46, 86, 115, 75, 86, 55,
                    103, 114, 82, 49, 66, 85, 69, 50, 57, 109, 71, 50, 70, 109, 50, 107, 88, 58,
                    50, 58, 116, 101, 115, 116, 95, 99, 114, 101, 100, 101, 110, 116, 105, 97, 108,
                    58, 49, 46, 48, 46, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ],
                signature: None,
                hash: None,
            };
            assert_eq!(expected_transaction, transaction);
        }
    }
}
