// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use log_derive::{logfn, logfn_inputs};

use crate::{
    client::LedgerClient,
    contracts::anoncreds::types::{
        credential_definition::{CredentialDefinition, CredentialDefinitionRecord},
        credential_definition_id::{CredentialDefinitionId, ParsedCredentialDefinitionId},
    },
    error::VdrResult,
    types::{
        Address, Transaction, TransactionBuilder, TransactionEndorsingDataBuilder,
        TransactionParser, TransactionType,
    },
    TransactionEndorsingData, VdrError,
};

const CONTRACT_NAME: &str = "CredentialDefinitionRegistry";
const METHOD_CREATE_CREDENTIAL_DEFINITION: &str = "createCredentialDefinition";
const METHOD_CREATE_CREDENTIAL_DEFINITION_SIGNED: &str = "createCredentialDefinitionSigned";
const METHOD_RESOLVE_CREDENTIAL_DEFINITION: &str = "resolveCredentialDefinition";

/// Build a transaction to create a new Credential Definition record (CredentialDefinitionRegistry.createCredentialDefinition contract method)
///
/// # Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `from`: [Address] - transaction sender account address
/// - `credential_definition`: [CredentialDefinition] - object matching to the specification - `<https://hyperledger.github.io/anoncreds-spec/#term:credential-definition>`
///
/// # Returns
///   transaction: [Transaction] - prepared write transaction object to sign and submit
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_create_credential_definition_transaction(
    client: &LedgerClient,
    from: &Address,
    credential_definition: &CredentialDefinition,
) -> VdrResult<Transaction> {
    credential_definition.validate()?;
    let identity = Address::try_from(&credential_definition.issuer_id)?;
    TransactionBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_CREATE_CREDENTIAL_DEFINITION)
        .add_param(&identity)?
        .add_param(&credential_definition.id().without_network()?)?
        .add_param(&credential_definition.issuer_id.without_network()?)?
        .add_param(&credential_definition.schema_id.without_network()?)?
        .add_param(credential_definition)?
        .set_type(TransactionType::Write)
        .set_from(from)
        .build(client)
        .await
}

/// Prepared data for endorsing creation of a new Credential Definition record
///     (CredentialDefinitionRegistry.createCredentialDefinitionSigned contract method)
///
/// #Params
///  - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `credential_definition`: [CredentialDefinition] - object matching to the specification - `<https://hyperledger.github.io/anoncreds-spec/#term:credential-definition>`
///
/// #Returns
///   data: [TransactionEndorsingData] - transaction endorsement data to sign
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_create_credential_definition_endorsing_data(
    client: &LedgerClient,
    credential_definition: &CredentialDefinition,
) -> VdrResult<TransactionEndorsingData> {
    credential_definition.validate()?;
    TransactionEndorsingDataBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_identity(&Address::try_from(&credential_definition.issuer_id)?)
        .set_method(METHOD_CREATE_CREDENTIAL_DEFINITION)
        .set_endorsing_method(METHOD_CREATE_CREDENTIAL_DEFINITION_SIGNED)
        .add_param(&credential_definition.id().without_network()?)?
        .add_param(&credential_definition.issuer_id.without_network()?)?
        .add_param(&credential_definition.schema_id.without_network()?)?
        .add_param(credential_definition)?
        .build(client)
        .await
}

/// Build a transaction to resolve an existing Credential Definition record by the given id
///  (CredentialDefinitionRegistry.resolveCredentialDefinition contract method)
///
/// # Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `id`: [CredentialDefinitionId] - id of Credential Definition to resolve
///
/// # Returns
///   transaction: [Transaction] - prepared read transaction object to submit
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_resolve_credential_definition_transaction(
    client: &LedgerClient,
    id: &CredentialDefinitionId,
) -> VdrResult<Transaction> {
    TransactionBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_RESOLVE_CREDENTIAL_DEFINITION)
        .add_param(&id.without_network()?)?
        .set_type(TransactionType::Read)
        .build(client)
        .await
}

/// Parse the result of execution CredentialDefinitionRegistry.resolveCredentialDefinition contract
/// method to receive a Credential Definition associated with the id
///
/// # Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `bytes`: [Vec] - result bytes returned from the ledger
///
/// # Returns
///   record: [CredentialDefinitionRecord] - parsed Credential Definition Record
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub fn parse_resolve_credential_definition_result(
    client: &LedgerClient,
    bytes: &[u8],
) -> VdrResult<CredentialDefinitionRecord> {
    TransactionParser::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_RESOLVE_CREDENTIAL_DEFINITION)
        .parse::<CredentialDefinitionRecord>(client, bytes)
}

/// Single step function to resolve a Credential Definition for the given ID
///
/// # Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `id`: [CredentialDefinitionId] - id of credential definition to resolve
///
/// # Returns
///   credential_definition: [CredentialDefinition] - Resolved Credential Definition object
pub async fn resolve_credential_definition(
    client: &LedgerClient,
    id: &CredentialDefinitionId,
) -> VdrResult<CredentialDefinition> {
    let parsed_id = ParsedCredentialDefinitionId::try_from(id)?;
    match (parsed_id.network.as_ref(), client.network()) {
        (Some(schema_network), Some(client_network)) if schema_network != client_network => {
            return Err(VdrError::InvalidCredentialDefinition(format!("Network of request credential definition id {} does not match to the client network {}", schema_network, client_network)));
        }
        _ => {}
    };

    let transaction = build_resolve_credential_definition_transaction(client, id).await?;
    let response = client.submit_transaction(&transaction).await?;
    if response.is_empty() {
        return Err(VdrError::ClientInvalidResponse(format!(
            "Credential Definition not found for id: {:?}",
            id
        )));
    }
    let cred_def_record = parse_resolve_credential_definition_result(client, &response)?;

    let cred_def_id = cred_def_record.credential_definition.id();
    if &cred_def_id != id {
        return Err(VdrError::InvalidCredentialDefinition(format!(
            "Credential Definition ID {} does not match to requested {}",
            cred_def_id.to_string(),
            id.to_string()
        )));
    }

    Ok(cred_def_record.credential_definition)
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::{
        client::client::test::{mock_client, CONFIG, DEFAULT_NONCE, TEST_ACCOUNT, TRUSTEE_ACCOUNT},
        contracts::{
            anoncreds::types::{
                credential_definition::test::{
                    credential_definition, credential_definition_value, CREDENTIAL_DEFINITION_TAG,
                },
                schema::test::SCHEMA_ID,
                schema_id::SchemaId,
            },
            did::types::{did::DID, did_doc::test::TEST_ETHR_DID},
        },
    };
    use rstest::rstest;

    mod build_create_credential_definition_transaction {
        use super::*;
        use serde_json::Value;

        #[async_std::test]
        async fn build_create_credential_definition_transaction_test() {
            let client = mock_client();
            let cred_def = credential_definition(
                &DID::from(TEST_ETHR_DID),
                &SchemaId::from(SCHEMA_ID),
                Some(CREDENTIAL_DEFINITION_TAG),
            );
            let transaction =
                build_create_credential_definition_transaction(&client, &TEST_ACCOUNT, &cred_def)
                    .await
                    .unwrap();
            let expected_transaction = Transaction {
                type_: TransactionType::Write,
                from: Some(TEST_ACCOUNT.clone()),
                to: CONFIG.contracts.cred_def_registry.address.clone(),
                nonce: Some(DEFAULT_NONCE.clone()),
                chain_id: CONFIG.chain_id,
                data: vec![
                    182, 196, 9, 117, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 240, 226, 219, 108, 141,
                    198, 198, 129, 187, 93, 106, 209, 33, 161, 7, 243, 0, 233, 178, 181, 252, 84,
                    206, 140, 139, 17, 81, 102, 58, 13, 78, 38, 76, 85, 25, 157, 189, 250, 139,
                    220, 160, 110, 164, 90, 238, 147, 145, 200, 16, 72, 81, 138, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 160,
                    224, 153, 168, 108, 167, 70, 3, 80, 108, 16, 229, 36, 115, 174, 106, 41, 145,
                    23, 58, 161, 7, 125, 80, 82, 53, 89, 11, 100, 185, 142, 80, 20, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 51, 100, 105, 100, 58, 101, 116, 104, 114, 58, 48, 120, 102,
                    48, 101, 50, 100, 98, 54, 99, 56, 100, 99, 54, 99, 54, 56, 49, 98, 98, 53, 100,
                    54, 97, 100, 49, 50, 49, 97, 49, 48, 55, 102, 51, 48, 48, 101, 57, 98, 50, 98,
                    53, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 42, 123, 34, 105, 115,
                    115, 117, 101, 114, 73, 100, 34, 58, 34, 100, 105, 100, 58, 101, 116, 104, 114,
                    58, 116, 101, 115, 116, 110, 101, 116, 58, 48, 120, 102, 48, 101, 50, 100, 98,
                    54, 99, 56, 100, 99, 54, 99, 54, 56, 49, 98, 98, 53, 100, 54, 97, 100, 49, 50,
                    49, 97, 49, 48, 55, 102, 51, 48, 48, 101, 57, 98, 50, 98, 53, 34, 44, 34, 115,
                    99, 104, 101, 109, 97, 73, 100, 34, 58, 34, 100, 105, 100, 58, 101, 116, 104,
                    114, 58, 116, 101, 115, 116, 110, 101, 116, 58, 48, 120, 102, 48, 101, 50, 100,
                    98, 54, 99, 56, 100, 99, 54, 99, 54, 56, 49, 98, 98, 53, 100, 54, 97, 100, 49,
                    50, 49, 97, 49, 48, 55, 102, 51, 48, 48, 101, 57, 98, 50, 98, 53, 47, 97, 110,
                    111, 110, 99, 114, 101, 100, 115, 47, 118, 48, 47, 83, 67, 72, 69, 77, 65, 47,
                    70, 49, 68, 67, 108, 97, 70, 69, 122, 105, 51, 116, 47, 49, 46, 48, 46, 48, 34,
                    44, 34, 99, 114, 101, 100, 68, 101, 102, 84, 121, 112, 101, 34, 58, 34, 67, 76,
                    34, 44, 34, 116, 97, 103, 34, 58, 34, 100, 101, 102, 97, 117, 108, 116, 34, 44,
                    34, 118, 97, 108, 117, 101, 34, 58, 123, 34, 110, 34, 58, 34, 55, 55, 57, 46,
                    46, 46, 51, 57, 55, 34, 44, 34, 114, 99, 116, 120, 116, 34, 58, 34, 55, 55, 52,
                    46, 46, 46, 57, 55, 55, 34, 44, 34, 115, 34, 58, 34, 55, 53, 48, 46, 46, 56,
                    57, 51, 34, 44, 34, 122, 34, 58, 34, 54, 51, 50, 46, 46, 46, 48, 48, 53, 34,
                    125, 125, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ],
                signature: None,
                hash: None,
            };
            assert_eq!(expected_transaction, transaction);
        }

        #[rstest]
        #[case("", credential_definition_value())]
        #[case(CREDENTIAL_DEFINITION_TAG, Value::Null)]
        async fn build_create_credential_definition_transaction_errors(
            #[case] tag: &str,
            #[case] value: Value,
        ) {
            let client = mock_client();
            let mut cred_def = credential_definition(
                &DID::from(TEST_ETHR_DID),
                &SchemaId::from(SCHEMA_ID),
                Some(tag),
            );
            cred_def.tag = tag.to_string();
            cred_def.value = value;

            let err = build_create_credential_definition_transaction(
                &client,
                &TRUSTEE_ACCOUNT,
                &cred_def,
            )
            .await
            .unwrap_err();

            assert!(matches!(err, VdrError::InvalidCredentialDefinition { .. }));
        }
    }

    mod build_resolve_credential_definition_transaction {
        use super::*;

        #[async_std::test]
        async fn build_resolve_credential_definition_transaction_test() {
            let client = mock_client();
            let cred_def = credential_definition(
                &DID::from(TEST_ETHR_DID),
                &SchemaId::from(SCHEMA_ID),
                Some(CREDENTIAL_DEFINITION_TAG),
            );
            let transaction =
                build_resolve_credential_definition_transaction(&client, &cred_def.id())
                    .await
                    .unwrap();
            let expected_transaction = Transaction {
                type_: TransactionType::Read,
                from: None,
                to: CONFIG.contracts.cred_def_registry.address.clone(),
                nonce: None,
                chain_id: CONFIG.chain_id,
                data: vec![
                    159, 136, 157, 181, 252, 84, 206, 140, 139, 17, 81, 102, 58, 13, 78, 38, 76,
                    85, 25, 157, 189, 250, 139, 220, 160, 110, 164, 90, 238, 147, 145, 200, 16, 72,
                    81, 138,
                ],
                signature: None,
                hash: None,
            };
            assert_eq!(expected_transaction, transaction);
        }
    }

    mod parse_resolve_credential_definition_result {
        use super::*;

        #[test]
        fn parse_resolve_credential_definition_result_test() {
            let client = mock_client();
            let data = vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 101, 203, 143, 197, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 42, 123, 34, 105, 115,
                115, 117, 101, 114, 73, 100, 34, 58, 34, 100, 105, 100, 58, 101, 116, 104, 114, 58,
                116, 101, 115, 116, 110, 101, 116, 58, 48, 120, 102, 48, 101, 50, 100, 98, 54, 99,
                56, 100, 99, 54, 99, 54, 56, 49, 98, 98, 53, 100, 54, 97, 100, 49, 50, 49, 97, 49,
                48, 55, 102, 51, 48, 48, 101, 57, 98, 50, 98, 53, 34, 44, 34, 115, 99, 104, 101,
                109, 97, 73, 100, 34, 58, 34, 100, 105, 100, 58, 101, 116, 104, 114, 58, 116, 101,
                115, 116, 110, 101, 116, 58, 48, 120, 102, 48, 101, 50, 100, 98, 54, 99, 56, 100,
                99, 54, 99, 54, 56, 49, 98, 98, 53, 100, 54, 97, 100, 49, 50, 49, 97, 49, 48, 55,
                102, 51, 48, 48, 101, 57, 98, 50, 98, 53, 47, 97, 110, 111, 110, 99, 114, 101, 100,
                115, 47, 118, 48, 47, 83, 67, 72, 69, 77, 65, 47, 70, 49, 68, 67, 108, 97, 70, 69,
                122, 105, 51, 116, 47, 49, 46, 48, 46, 48, 34, 44, 34, 99, 114, 101, 100, 68, 101,
                102, 84, 121, 112, 101, 34, 58, 34, 67, 76, 34, 44, 34, 116, 97, 103, 34, 58, 34,
                100, 101, 102, 97, 117, 108, 116, 34, 44, 34, 118, 97, 108, 117, 101, 34, 58, 123,
                34, 110, 34, 58, 34, 55, 55, 57, 46, 46, 46, 51, 57, 55, 34, 44, 34, 114, 99, 116,
                120, 116, 34, 58, 34, 55, 55, 52, 46, 46, 46, 57, 55, 55, 34, 44, 34, 115, 34, 58,
                34, 55, 53, 48, 46, 46, 56, 57, 51, 34, 44, 34, 122, 34, 58, 34, 54, 51, 50, 46,
                46, 46, 48, 48, 53, 34, 125, 125, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0,
            ];
            let parsed_cred_def =
                parse_resolve_credential_definition_result(&client, &data).unwrap();
            let expected_cred_def = credential_definition(
                &DID::from(TEST_ETHR_DID),
                &SchemaId::from(SCHEMA_ID),
                Some(CREDENTIAL_DEFINITION_TAG),
            );
            assert_eq!(expected_cred_def, parsed_cred_def.credential_definition);
        }
    }
}
