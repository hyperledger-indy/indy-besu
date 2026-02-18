// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use log_derive::{logfn, logfn_inputs};

use crate::{
    client::LedgerClient,
    contracts::anoncreds::types::{
        schema::{Schema, SchemaRecord},
        schema_id::{ParsedSchemaId, SchemaId},
    },
    error::VdrResult,
    types::{
        Address, Transaction, TransactionBuilder, TransactionEndorsingDataBuilder,
        TransactionParser, TransactionType,
    },
    TransactionEndorsingData, VdrError,
};

const CONTRACT_NAME: &str = "SchemaRegistry";
const METHOD_CREATE_SCHEMA: &str = "createSchema";
const METHOD_CREATE_SCHEMA_SIGNED: &str = "createSchemaSigned";
const METHOD_RESOLVE_SCHEMA: &str = "resolveSchema";

/// Build a transaction to create a new Schema (SchemaRegistry.createSchema contract method)
///
/// # Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `from`: [Address] - transaction sender account address
/// - `schema`: [Schema] - object matching to the specification - `<https://hyperledger.github.io/anoncreds-spec/#term:schema>`
///
/// # Returns
///   transaction: [Transaction] - prepared write transaction object to sign and submit
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_create_schema_transaction(
    client: &LedgerClient,
    from: &Address,
    schema: &Schema,
) -> VdrResult<Transaction> {
    schema.validate()?;
    let identity = Address::try_from(&schema.issuer_id)?;
    TransactionBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_CREATE_SCHEMA)
        .add_param(&identity)?
        .add_param(&schema.id().without_network()?)?
        .add_param(&schema.issuer_id.without_network()?)?
        .add_param(schema)?
        .set_type(TransactionType::Write)
        .set_from(from)
        .build(client)
        .await
}

/// Prepared data for endorsing creation of a new Schema (SchemaRegistry.createSchemaSigned contract method)
///
/// #Params
///  - `client`: [LedgerClient] - client connected to the network where contract will be executed
///  - `schema`: [Schema] - object matching to the specification - `<https://hyperledger.github.io/anoncreds-spec/#term:schema>`
///
/// #Returns
///   data: [TransactionEndorsingData] - transaction endorsement data to sign
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_create_schema_endorsing_data(
    client: &LedgerClient,
    schema: &Schema,
) -> VdrResult<TransactionEndorsingData> {
    schema.validate()?;
    TransactionEndorsingDataBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_identity(&Address::try_from(&schema.issuer_id)?)
        .set_method(METHOD_CREATE_SCHEMA)
        .set_endorsing_method(METHOD_CREATE_SCHEMA_SIGNED)
        .add_param(&schema.id().without_network()?)?
        .add_param(&schema.issuer_id.without_network()?)?
        .add_param(schema)?
        .build(client)
        .await
}

/// Build a transaction to resolve an existing Schema record by the given id
///  (SchemaRegistry.resolveSchema contract method)
///
/// # Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `id`: [SchemaId] - id of Schema to resolve
///
/// # Returns
///   transaction: [Transaction] - prepared read transaction object to submit
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_resolve_schema_transaction(
    client: &LedgerClient,
    id: &SchemaId,
) -> VdrResult<Transaction> {
    TransactionBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_RESOLVE_SCHEMA)
        .add_param(&id.without_network()?)?
        .set_type(TransactionType::Read)
        .build(client)
        .await
}

/// Parse the result of execution SchemaRegistry.resolveSchema contract method to receive a Schema associated with the id
///
/// # Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `bytes`: [Vec] - result bytes returned from the ledger
///
/// # Returns
///   record: [SchemaRecord] - parsed Schema Record
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub fn parse_resolve_schema_result(client: &LedgerClient, bytes: &[u8]) -> VdrResult<SchemaRecord> {
    TransactionParser::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_RESOLVE_SCHEMA)
        .parse::<SchemaRecord>(client, bytes)
}

/// Single step function to resolve a Schema for the given ID
///
/// # Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `id`: [SchemaId] - id of schema to resolve
///
/// # Returns
///   schema: [Schema] - Resolved Schema object
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn resolve_schema(client: &LedgerClient, id: &SchemaId) -> VdrResult<Schema> {
    let parsed_id = ParsedSchemaId::try_from(id)?;
    match (parsed_id.network.as_ref(), client.network()) {
        (Some(schema_network), Some(client_network)) if schema_network != client_network => {
            return Err(VdrError::InvalidSchema(format!(
                "Network of request schema id {} does not match to the client network {}",
                schema_network, client_network
            )));
        }
        _ => {}
    };

    let transaction = build_resolve_schema_transaction(client, id).await?;
    let response = client.submit_transaction(&transaction).await?;
    if response.is_empty() {
        return Err(VdrError::ClientInvalidResponse(format!(
            "Schema not found for id: {:?}",
            id
        )));
    }
    let schema_record = parse_resolve_schema_result(client, &response)?;
    let schema = schema_record.schema;

    let schema_id = schema.id();
    if &schema_id != id {
        return Err(VdrError::InvalidSchema(format!(
            "Schema ID {} does not match to requested {}",
            schema_id.to_string(),
            id.to_string()
        )));
    }

    Ok(schema)
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::{
        client::client::test::{mock_client, CONFIG, DEFAULT_NONCE, TEST_ACCOUNT},
        contracts::{
            anoncreds::types::schema::test::{
                schema, SCHEMA_ATTRIBUTES, SCHEMA_NAME, SCHEMA_VERSION,
            },
            did::types::{did::DID, did_doc::test::TEST_ETHR_DID},
        },
    };

    mod build_create_schema_transaction {
        use super::*;
        use rstest::rstest;
        use std::collections::HashSet;

        #[async_std::test]
        async fn build_create_schema_transaction_test() {
            let client = mock_client();
            let schema = schema(&DID::from(TEST_ETHR_DID), Some(SCHEMA_NAME));
            let transaction = build_create_schema_transaction(&client, &TEST_ACCOUNT, &schema)
                .await
                .unwrap();
            let expected_transaction = Transaction {
                type_: TransactionType::Write,
                from: Some(TEST_ACCOUNT.clone()),
                to: CONFIG.contracts.schema_registry.address.clone(),
                nonce: Some(DEFAULT_NONCE.clone()),
                chain_id: CONFIG.chain_id,
                data: vec![
                    131, 211, 251, 60, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 240, 226, 219, 108, 141,
                    198, 198, 129, 187, 93, 106, 209, 33, 161, 7, 243, 0, 233, 178, 181, 224, 153,
                    168, 108, 167, 70, 3, 80, 108, 16, 229, 36, 115, 174, 106, 41, 145, 23, 58,
                    161, 7, 125, 80, 82, 53, 89, 11, 100, 185, 142, 80, 20, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 224, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 51, 100, 105, 100, 58, 101, 116, 104, 114, 58, 48,
                    120, 102, 48, 101, 50, 100, 98, 54, 99, 56, 100, 99, 54, 99, 54, 56, 49, 98,
                    98, 53, 100, 54, 97, 100, 49, 50, 49, 97, 49, 48, 55, 102, 51, 48, 48, 101, 57,
                    98, 50, 98, 53, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 141, 123,
                    34, 105, 115, 115, 117, 101, 114, 73, 100, 34, 58, 34, 100, 105, 100, 58, 101,
                    116, 104, 114, 58, 116, 101, 115, 116, 110, 101, 116, 58, 48, 120, 102, 48,
                    101, 50, 100, 98, 54, 99, 56, 100, 99, 54, 99, 54, 56, 49, 98, 98, 53, 100, 54,
                    97, 100, 49, 50, 49, 97, 49, 48, 55, 102, 51, 48, 48, 101, 57, 98, 50, 98, 53,
                    34, 44, 34, 110, 97, 109, 101, 34, 58, 34, 70, 49, 68, 67, 108, 97, 70, 69,
                    122, 105, 51, 116, 34, 44, 34, 118, 101, 114, 115, 105, 111, 110, 34, 58, 34,
                    49, 46, 48, 46, 48, 34, 44, 34, 97, 116, 116, 114, 78, 97, 109, 101, 115, 34,
                    58, 91, 34, 70, 105, 114, 115, 116, 32, 78, 97, 109, 101, 34, 93, 125, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ],
                signature: None,
                hash: None,
            };
            assert_eq!(expected_transaction, transaction);
        }

        #[rstest]
        #[case::name_not_provided("", SCHEMA_VERSION, & SCHEMA_ATTRIBUTES)]
        #[case::version_not_provided(SCHEMA_NAME, "", & SCHEMA_ATTRIBUTES)]
        #[case::attributes_not_provided(SCHEMA_NAME, SCHEMA_VERSION, & HashSet::new())]
        async fn build_create_schema_transaction_errors(
            #[case] name: &str,
            #[case] version: &str,
            #[case] attributes: &HashSet<String>,
        ) {
            let client = mock_client();
            let mut schema = schema(&DID::from(TEST_ETHR_DID), Some(name));
            schema.name = name.to_string();
            schema.version = version.to_string();
            schema.attr_names = attributes.clone();

            let err = build_create_schema_transaction(&client, &TEST_ACCOUNT, &schema)
                .await
                .unwrap_err();

            assert!(matches!(err, VdrError::InvalidSchema { .. }));
        }
    }

    mod build_resolve_schema_transaction {
        use super::*;

        #[async_std::test]
        async fn build_resolve_schema_transaction_test() {
            let client = mock_client();
            let schema = schema(&DID::from(TEST_ETHR_DID), Some(SCHEMA_NAME));
            let transaction = build_resolve_schema_transaction(&client, &schema.id())
                .await
                .unwrap();
            let expected_transaction = Transaction {
                type_: TransactionType::Read,
                from: None,
                to: CONFIG.contracts.schema_registry.address.clone(),
                nonce: None,
                chain_id: CONFIG.chain_id,
                data: vec![
                    174, 190, 203, 28, 224, 153, 168, 108, 167, 70, 3, 80, 108, 16, 229, 36, 115,
                    174, 106, 41, 145, 23, 58, 161, 7, 125, 80, 82, 53, 89, 11, 100, 185, 142, 80,
                    20,
                ],
                signature: None,
                hash: None,
            };
            assert_eq!(expected_transaction, transaction);
        }
    }

    mod parse_resolve_schema_result {
        use super::*;
        use crate::contracts::did::types::did::DID;

        #[test]
        fn parse_resolve_schema_result_test() {
            let client = mock_client();
            let data = vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 101, 203, 143, 187, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 141, 123, 34, 105, 115,
                115, 117, 101, 114, 73, 100, 34, 58, 34, 100, 105, 100, 58, 101, 116, 104, 114, 58,
                116, 101, 115, 116, 110, 101, 116, 58, 48, 120, 102, 48, 101, 50, 100, 98, 54, 99,
                56, 100, 99, 54, 99, 54, 56, 49, 98, 98, 53, 100, 54, 97, 100, 49, 50, 49, 97, 49,
                48, 55, 102, 51, 48, 48, 101, 57, 98, 50, 98, 53, 34, 44, 34, 110, 97, 109, 101,
                34, 58, 34, 70, 49, 68, 67, 108, 97, 70, 69, 122, 105, 51, 116, 34, 44, 34, 118,
                101, 114, 115, 105, 111, 110, 34, 58, 34, 49, 46, 48, 46, 48, 34, 44, 34, 97, 116,
                116, 114, 78, 97, 109, 101, 115, 34, 58, 91, 34, 70, 105, 114, 115, 116, 32, 78,
                97, 109, 101, 34, 93, 125, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ];
            let parsed_schema = parse_resolve_schema_result(&client, &data).unwrap();
            let expected_schema = schema(&DID::from(TEST_ETHR_DID), Some(SCHEMA_NAME));
            assert_eq!(expected_schema, parsed_schema.schema);
        }
    }
}
