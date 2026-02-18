// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use log_derive::{logfn, logfn_inputs};

use crate::{
    client::LedgerClient,
    contracts::did::types::{
        did::DID,
        did_doc::{DidDocument, DidRecord},
    },
    error::VdrResult,
    types::{
        Address, Transaction, TransactionBuilder, TransactionEndorsingDataBuilder,
        TransactionParser, TransactionType,
    },
    TransactionEndorsingData,
};

const CONTRACT_NAME: &str = "IndyDidRegistry";
const METHOD_CREATE_DID: &str = "createDid";
const METHOD_CREATE_DID_SIGNED: &str = "createDidSigned";
const METHOD_UPDATE_DID: &str = "updateDid";
const METHOD_UPDATE_DID_SIGNED: &str = "updateDidSigned";
const METHOD_DEACTIVATE_DID: &str = "deactivateDid";
const METHOD_DEACTIVATE_DID_SIGNED: &str = "deactivateDidSigned";
const METHOD_RESOLVE_DID: &str = "resolveDid";

pub const INDYBESU_DID_METHOD: &str = "indybesu";

/// Build a transaction to create a new DID record (IndyDidRegistry.createDid contract method)
///
/// # Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `from`: [Address] - transaction sender account address
/// - `did`: [DID] - DID to create.
/// - `did_doc`: [DidDocument] - DID Document matching to the specification: `<https://www.w3.org/TR/did-core/>`
///
/// # Returns
///   transaction: [Transaction] - prepared write transaction object to sign and submit
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_create_did_transaction(
    client: &LedgerClient,
    from: &Address,
    did: &DID,
    did_doc: &DidDocument,
) -> VdrResult<Transaction> {
    did_doc.validate()?;
    TransactionBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_CREATE_DID)
        .add_param(&Address::try_from(did)?)?
        .add_param(did_doc)?
        .set_type(TransactionType::Write)
        .set_from(from)
        .build(client)
        .await
}

/// Prepared data for endorsing creation of a new DID record (IndyDidRegistry.createDidSigned contract method)
///
/// #Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `did`: [DID] - DID to create.
/// - `did_doc`: [DidDocument] - DID Document matching to the specification: `<https://www.w3.org/TR/did-core/>`
///
/// #Returns
///   data: [TransactionEndorsingData] - transaction endorsement data to sign
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_create_did_endorsing_data(
    client: &LedgerClient,
    did: &DID,
    did_doc: &DidDocument,
) -> VdrResult<TransactionEndorsingData> {
    did_doc.validate()?;
    TransactionEndorsingDataBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_identity(&Address::try_from(did)?)
        .set_method(METHOD_CREATE_DID)
        .set_endorsing_method(METHOD_CREATE_DID_SIGNED)
        .add_param(did_doc)?
        .build(client)
        .await
}

/// Build a transaction to update an existing DID record (IndyDidRegistry.updateDid contract method)
///
/// # Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `from`: [Address] - transaction sender account address
/// - `did`: [DID] - DID to update.
/// - `did_doc`: [DidDocument] - DID Document matching to the specification: `<https://www.w3.org/TR/did-core/>`
///
/// # Returns
///   transaction: [Transaction] - prepared write transaction object to sign and submit
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_update_did_transaction(
    client: &LedgerClient,
    from: &Address,
    did: &DID,
    did_doc: &DidDocument,
) -> VdrResult<Transaction> {
    did_doc.validate()?;
    TransactionBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_UPDATE_DID)
        .add_param(&Address::try_from(did)?)?
        .add_param(did_doc)?
        .set_type(TransactionType::Write)
        .set_from(from)
        .build(client)
        .await
}

/// Prepared data for endorsing update of an existing DID record (IndyDidRegistry.updateDidSigned contract method)
///
/// #Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `did`: [DID] - DID to create.
/// - `did_doc`: [DidDocument] - DID Document matching to the specification: `<https://www.w3.org/TR/did-core/>`
///
/// #Returns
///   data: [TransactionEndorsingData] - transaction endorsement data to sign
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_update_did_endorsing_data(
    client: &LedgerClient,
    did: &DID,
    did_doc: &DidDocument,
) -> VdrResult<TransactionEndorsingData> {
    did_doc.validate()?;
    TransactionEndorsingDataBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_identity(&Address::try_from(did)?)
        .set_method(METHOD_UPDATE_DID)
        .set_endorsing_method(METHOD_UPDATE_DID_SIGNED)
        .add_param(did_doc)?
        .build(client)
        .await
}

/// Build a transaction to deactivate an existing DID record (IndyDidRegistry.deactivateDid contract method)
///
/// # Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `from`: [Address] - transaction sender account address
/// - `did`: [DID] - DID to deactivate.
///
/// # Returns
///   transaction: [Transaction] - prepared write transaction object to sign and submit
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_deactivate_did_transaction(
    client: &LedgerClient,
    from: &Address,
    did: &DID,
) -> VdrResult<Transaction> {
    TransactionBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_DEACTIVATE_DID)
        .add_param(&Address::try_from(did)?)?
        .set_type(TransactionType::Write)
        .set_from(from)
        .build(client)
        .await
}

/// Prepared data for endorsing deactivation of an existing DID record (IndyDidRegistry.deactivateDidSigned contract method)
///
/// #Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `did`: [DID] - DID to create.
/// - `did_doc`: [DidDocument] - DID Document matching to the specification: `<https://www.w3.org/TR/did-core/>`
///
/// #Returns
///   data: [TransactionEndorsingData] - transaction endorsement data to sign
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_deactivate_did_endorsing_data(
    client: &LedgerClient,
    did: &DID,
) -> VdrResult<TransactionEndorsingData> {
    TransactionEndorsingDataBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_identity(&Address::try_from(did)?)
        .set_method(METHOD_DEACTIVATE_DID)
        .set_endorsing_method(METHOD_DEACTIVATE_DID_SIGNED)
        .build(client)
        .await
}

/// Build a transaction to resolve a DID Record (IndyDidRegistry.resolveDid contract method)
///
/// # Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `did`: [DID] - DID to resolve.
///
/// # Returns
///   transaction: [Transaction] - prepared read transaction object to submit
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_resolve_did_transaction(
    client: &LedgerClient,
    did: &DID,
) -> VdrResult<Transaction> {
    let identity = Address::try_from(did)?;
    TransactionBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_RESOLVE_DID)
        .add_param(&identity)?
        .set_type(TransactionType::Read)
        .build(client)
        .await
}

/// Parse the result of execution transaction to resolve a DID Record (IndyDidRegistry.resolveDid contract method)
///
/// # Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `bytes`: [Vec] - result bytes returned from the ledger
///
/// # Returns
/// [DidRecord] DID Record containing DID Document and metadata associated with the DID
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub fn parse_resolve_did_result(client: &LedgerClient, bytes: &[u8]) -> VdrResult<DidRecord> {
    let did_record = TransactionParser::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_RESOLVE_DID)
        .parse::<DidRecord>(client, bytes)?;

    did_record.document.validate()?;

    Ok(did_record)
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::{
        client::client::test::{mock_client, CONFIG, DEFAULT_NONCE, TEST_ACCOUNT},
        contracts::did::types::{
            did::DID,
            did_doc::test::{did_doc, TEST_ETHR_DID},
        },
    };

    mod build_create_did_transaction {
        use super::*;
        use crate::client::client::test::mock_client;

        #[async_std::test]
        async fn build_create_did_transaction_test() {
            let client = mock_client();
            let did = DID::from(TEST_ETHR_DID);
            let did_doc = did_doc(TEST_ETHR_DID);
            let transaction = build_create_did_transaction(&client, &TEST_ACCOUNT, &did, &did_doc)
                .await
                .unwrap();
            let expected_transaction = Transaction {
                type_: TransactionType::Write,
                from: Some(TEST_ACCOUNT.clone()),
                to: CONFIG.contracts.indy_did_registry.address.clone(),
                nonce: Some(DEFAULT_NONCE.clone()),
                chain_id: CONFIG.chain_id,
                data: vec![
                    245, 149, 121, 68, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 240, 226, 219, 108, 141,
                    198, 198, 129, 187, 93, 106, 209, 33, 161, 7, 243, 0, 233, 178, 181, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 2, 16, 123, 34, 64, 99, 111, 110, 116, 101, 120, 116, 34,
                    58, 91, 34, 104, 116, 116, 112, 115, 58, 47, 47, 119, 119, 119, 46, 119, 51,
                    46, 111, 114, 103, 47, 110, 115, 47, 100, 105, 100, 47, 118, 49, 34, 93, 44,
                    34, 105, 100, 34, 58, 34, 100, 105, 100, 58, 105, 110, 100, 121, 98, 101, 115,
                    117, 58, 100, 105, 100, 58, 101, 116, 104, 114, 58, 116, 101, 115, 116, 110,
                    101, 116, 58, 48, 120, 102, 48, 101, 50, 100, 98, 54, 99, 56, 100, 99, 54, 99,
                    54, 56, 49, 98, 98, 53, 100, 54, 97, 100, 49, 50, 49, 97, 49, 48, 55, 102, 51,
                    48, 48, 101, 57, 98, 50, 98, 53, 34, 44, 34, 118, 101, 114, 105, 102, 105, 99,
                    97, 116, 105, 111, 110, 77, 101, 116, 104, 111, 100, 34, 58, 91, 123, 34, 105,
                    100, 34, 58, 34, 100, 105, 100, 58, 105, 110, 100, 121, 98, 101, 115, 117, 58,
                    100, 105, 100, 58, 101, 116, 104, 114, 58, 116, 101, 115, 116, 110, 101, 116,
                    58, 48, 120, 102, 48, 101, 50, 100, 98, 54, 99, 56, 100, 99, 54, 99, 54, 56,
                    49, 98, 98, 53, 100, 54, 97, 100, 49, 50, 49, 97, 49, 48, 55, 102, 51, 48, 48,
                    101, 57, 98, 50, 98, 53, 35, 75, 69, 89, 45, 49, 34, 44, 34, 116, 121, 112,
                    101, 34, 58, 34, 69, 100, 50, 53, 53, 49, 57, 86, 101, 114, 105, 102, 105, 99,
                    97, 116, 105, 111, 110, 75, 101, 121, 50, 48, 49, 56, 34, 44, 34, 99, 111, 110,
                    116, 114, 111, 108, 108, 101, 114, 34, 58, 34, 100, 105, 100, 58, 105, 110,
                    100, 121, 98, 101, 115, 117, 58, 100, 105, 100, 58, 101, 116, 104, 114, 58,
                    116, 101, 115, 116, 110, 101, 116, 58, 48, 120, 102, 48, 101, 50, 100, 98, 54,
                    99, 56, 100, 99, 54, 99, 54, 56, 49, 98, 98, 53, 100, 54, 97, 100, 49, 50, 49,
                    97, 49, 48, 55, 102, 51, 48, 48, 101, 57, 98, 50, 98, 53, 34, 44, 34, 112, 117,
                    98, 108, 105, 99, 75, 101, 121, 77, 117, 108, 116, 105, 98, 97, 115, 101, 34,
                    58, 34, 122, 65, 75, 74, 80, 51, 102, 55, 66, 68, 54, 87, 52, 105, 87, 69, 81,
                    57, 106, 119, 110, 100, 86, 84, 67, 66, 113, 56, 117, 97, 50, 85, 116, 116, 56,
                    69, 69, 106, 74, 54, 86, 120, 115, 102, 34, 125, 93, 44, 34, 97, 117, 116, 104,
                    101, 110, 116, 105, 99, 97, 116, 105, 111, 110, 34, 58, 91, 34, 100, 105, 100,
                    58, 105, 110, 100, 121, 98, 101, 115, 117, 58, 100, 105, 100, 58, 101, 116,
                    104, 114, 58, 116, 101, 115, 116, 110, 101, 116, 58, 48, 120, 102, 48, 101, 50,
                    100, 98, 54, 99, 56, 100, 99, 54, 99, 54, 56, 49, 98, 98, 53, 100, 54, 97, 100,
                    49, 50, 49, 97, 49, 48, 55, 102, 51, 48, 48, 101, 57, 98, 50, 98, 53, 35, 75,
                    69, 89, 45, 49, 34, 93, 125, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ],
                signature: None,
                hash: None,
            };
            assert_eq!(expected_transaction, transaction);
        }
    }

    mod build_resolve_did_transaction {
        use super::*;

        #[async_std::test]
        async fn build_resolve_did_transaction_test() {
            let client = mock_client();
            let transaction = build_resolve_did_transaction(&client, &DID::from(TEST_ETHR_DID))
                .await
                .unwrap();
            let expected_transaction = Transaction {
                type_: TransactionType::Read,
                from: None,
                to: CONFIG.contracts.indy_did_registry.address.clone(),
                nonce: None,
                chain_id: CONFIG.chain_id,
                data: vec![
                    24, 48, 235, 91, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 240, 226, 219, 108, 141,
                    198, 198, 129, 187, 93, 106, 209, 33, 161, 7, 243, 0, 233, 178, 181,
                ],
                signature: None,
                hash: None,
            };
            assert_eq!(expected_transaction, transaction);
        }
    }

    mod parse_resolve_did_result {
        use super::*;

        #[test]
        fn parse_resolve_did_result_with_metadata_test() {
            let client = mock_client();

            let data = vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 192, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 160, 65, 210,
                31, 10, 202, 228, 139, 73, 181, 120, 177, 49, 18, 63, 155, 48, 194, 28, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 101, 207,
                153, 152, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 101, 207, 153, 152, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 204, 123, 34, 64,
                99, 111, 110, 116, 101, 120, 116, 34, 58, 91, 34, 104, 116, 116, 112, 115, 58, 47,
                47, 119, 119, 119, 46, 119, 51, 46, 111, 114, 103, 47, 110, 115, 47, 100, 105, 100,
                47, 118, 49, 34, 93, 44, 34, 105, 100, 34, 58, 34, 100, 105, 100, 58, 105, 110,
                100, 121, 98, 101, 115, 117, 58, 48, 120, 102, 48, 101, 50, 100, 98, 54, 99, 56,
                100, 99, 54, 99, 54, 56, 49, 98, 98, 53, 100, 54, 97, 100, 49, 50, 49, 97, 49, 48,
                55, 102, 51, 48, 48, 101, 57, 98, 50, 98, 53, 34, 44, 34, 118, 101, 114, 105, 102,
                105, 99, 97, 116, 105, 111, 110, 77, 101, 116, 104, 111, 100, 34, 58, 91, 123, 34,
                105, 100, 34, 58, 34, 100, 105, 100, 58, 105, 110, 100, 121, 98, 101, 115, 117, 58,
                48, 120, 102, 48, 101, 50, 100, 98, 54, 99, 56, 100, 99, 54, 99, 54, 56, 49, 98,
                98, 53, 100, 54, 97, 100, 49, 50, 49, 97, 49, 48, 55, 102, 51, 48, 48, 101, 57, 98,
                50, 98, 53, 35, 75, 69, 89, 45, 49, 34, 44, 34, 116, 121, 112, 101, 34, 58, 34, 69,
                100, 50, 53, 53, 49, 57, 86, 101, 114, 105, 102, 105, 99, 97, 116, 105, 111, 110,
                75, 101, 121, 50, 48, 49, 56, 34, 44, 34, 99, 111, 110, 116, 114, 111, 108, 108,
                101, 114, 34, 58, 34, 100, 105, 100, 58, 105, 110, 100, 121, 98, 101, 115, 117, 58,
                48, 120, 102, 48, 101, 50, 100, 98, 54, 99, 56, 100, 99, 54, 99, 54, 56, 49, 98,
                98, 53, 100, 54, 97, 100, 49, 50, 49, 97, 49, 48, 55, 102, 51, 48, 48, 101, 57, 98,
                50, 98, 53, 34, 44, 34, 112, 117, 98, 108, 105, 99, 75, 101, 121, 77, 117, 108,
                116, 105, 98, 97, 115, 101, 34, 58, 34, 122, 65, 75, 74, 80, 51, 102, 55, 66, 68,
                54, 87, 52, 105, 87, 69, 81, 57, 106, 119, 110, 100, 86, 84, 67, 66, 113, 56, 117,
                97, 50, 85, 116, 116, 56, 69, 69, 106, 74, 54, 86, 120, 115, 102, 34, 125, 93, 44,
                34, 97, 117, 116, 104, 101, 110, 116, 105, 99, 97, 116, 105, 111, 110, 34, 58, 91,
                34, 100, 105, 100, 58, 105, 110, 100, 121, 98, 101, 115, 117, 58, 48, 120, 102, 48,
                101, 50, 100, 98, 54, 99, 56, 100, 99, 54, 99, 54, 56, 49, 98, 98, 53, 100, 54, 97,
                100, 49, 50, 49, 97, 49, 48, 55, 102, 51, 48, 48, 101, 57, 98, 50, 98, 53, 35, 75,
                69, 89, 45, 49, 34, 93, 125, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0,
            ];
            let parsed_did_doc = parse_resolve_did_result(&client, &data).unwrap();
            assert_eq!(did_doc(TEST_ACCOUNT.as_ref()), parsed_did_doc.document);
        }
    }
}
