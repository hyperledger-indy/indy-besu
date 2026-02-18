// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use log_derive::{logfn, logfn_inputs};

use crate::{
    client::LedgerClient,
    contracts::did::types::{
        did_doc_attribute::{DelegateType, DidDocAttribute, Validity},
        did_events::{DidAttributeChanged, DidDelegateChanged, DidEvents, DidOwnerChanged},
    },
    error::VdrResult,
    types::{
        Address, EventLog, EventParser, EventQuery, EventQueryBuilder, MethodUintBytesParam,
        Transaction, TransactionBuilder, TransactionEndorsingDataBuilder, TransactionParser,
        TransactionType,
    },
    Block, Nonce, TransactionEndorsingData, VdrError, DID,
};

const CONTRACT_NAME: &str = "EthereumExtDidRegistry";

const METHOD_DID_CHANGED: &str = "changed";
const METHOD_DID_NONCE: &str = "nonce";
const METHOD_DID_OWNER: &str = "owners";
const METHOD_CHANGE_OWNER: &str = "changeOwner";
const METHOD_CHANGE_OWNER_SIGNED: &str = "changeOwnerSigned";
const METHOD_ADD_DELEGATE: &str = "addDelegate";
const METHOD_ADD_DELEGATE_SIGNED: &str = "addDelegateSigned";
const METHOD_REVOKE_DELEGATE: &str = "revokeDelegate";
const METHOD_REVOKE_DELEGATE_SIGNED: &str = "revokeDelegateSigned";
const METHOD_SET_ATTRIBUTE: &str = "setAttribute";
const METHOD_SET_ATTRIBUTE_SIGNED: &str = "setAttributeSigned";
const METHOD_REVOKE_ATTRIBUTE: &str = "revokeAttribute";
const METHOD_REVOKE_ATTRIBUTE_SIGNED: &str = "revokeAttributeSigned";

const EVENT_DID_ATTRIBUTE_CHANGED: &str = "DIDAttributeChanged";
const EVENT_DID_DELEGATE_CHANGED: &str = "DIDDelegateChanged";
const EVENT_DID_OWNER_CHANGED: &str = "DIDOwnerChanged";

pub const ETHR_DID_METHOD: &str = "ethr";

/// Build a transaction to change the owner of a DID (EthereumExtDidRegistry.changeOwner contract method)
///
/// #Params
///  - `client`: [LedgerClient] - client connected to the network where contract will be executed
///  - `sender`: [Address] - sender account address (Must be DID owner)
///  - `did`: [DID] - DID to change ownership
///  - `new_owner`: [Address] - account address of new owner
///
/// #Returns
///   transaction: [Transaction] - prepared write transaction object to sign and submit
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_did_change_owner_transaction(
    client: &LedgerClient,
    sender: &Address,
    did: &DID,
    new_owner: &Address,
) -> VdrResult<Transaction> {
    let identity = Address::try_from(did)?;
    TransactionBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_CHANGE_OWNER)
        .add_param(&identity)?
        .add_param(new_owner)?
        .set_type(TransactionType::Write)
        .set_from(sender)
        .build(client)
        .await
}

/// Prepared data for endorsing a change of a DID owner (EthereumExtDidRegistry.changeOwnerSigned contract method)
///
/// #Params
///  - `client`: [LedgerClient] - client connected to the network where contract will be executed
///  - `did`: [DID] - DID to change ownership
///  - `new_owner`: [Address] - account address of new owner
///
/// #Returns
///   data: [TransactionEndorsingData] - transaction endorsement data to sign
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_did_change_owner_endorsing_data(
    client: &LedgerClient,
    did: &DID,
    new_owner: &Address,
) -> VdrResult<TransactionEndorsingData> {
    let identity = Address::try_from(did)?;
    let nonce = resolve_identity_nonce(client, &identity).await?;
    TransactionEndorsingDataBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_identity(&identity)
        .set_nonce(&nonce)
        .set_method(METHOD_CHANGE_OWNER)
        .set_endorsing_method(METHOD_CHANGE_OWNER_SIGNED)
        .add_param(new_owner)?
        .build(client)
        .await
}

/// Build a transaction to add a DID delegate (EthereumExtDidRegistry.addDelegate contract method)
/// An identity can assign multiple delegates to manage signing on their behalf for specific purposes.
///
/// #Params
///  - `client`: [LedgerClient] - client connected to the network where contract will be executed
///  - `sender`: [Address] - sender account address (Must be DID owner)
///  - `did`: [DID] - DID to add a delegate
///  - `delegate_type`: [DelegateType] - type of delegation (`veriKey` or `sigAuth`)
///  - `delegate`: [Address] - account address of delegate
///  - `validity`: [Validity] - delegate validity time
///
/// #Returns
///   transaction: [Transaction] - prepared write transaction object to sign and submit
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_did_add_delegate_transaction(
    client: &LedgerClient,
    sender: &Address,
    did: &DID,
    delegate_type: &DelegateType,
    delegate: &Address,
    validity: &Validity,
) -> VdrResult<Transaction> {
    let identity = Address::try_from(did)?;
    TransactionBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_ADD_DELEGATE)
        .add_param(&identity)?
        .add_param(delegate_type)?
        .add_param(delegate)?
        .add_param(validity)?
        .set_type(TransactionType::Write)
        .set_from(sender)
        .build(client)
        .await
}

/// Prepared data for endorsing adding of a DID delegate (EthereumExtDidRegistry.addDelegateSigned contract method)
/// An identity can assign multiple delegates to manage signing on their behalf for specific purposes.
///
/// #Params
///  - `client`: [LedgerClient] - client connected to the network where contract will be executed
///  - `did`: [DID] - DID to add a delegate
///  - `delegate_type`: [DelegateType] - type of delegation (`veriKey` or `sigAuth`)
///  - `delegate`: [Address] - account address of delegate
///  - `validity`: [Validity] - delegate validity time
///
/// #Returns
///   data: [TransactionEndorsingData] - transaction endorsement data to sign
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_did_add_delegate_endorsing_data(
    client: &LedgerClient,
    did: &DID,
    delegate_type: &DelegateType,
    delegate: &Address,
    validity: &Validity,
) -> VdrResult<TransactionEndorsingData> {
    let identity = Address::try_from(did)?;
    let nonce = resolve_identity_nonce(client, &identity).await?;
    TransactionEndorsingDataBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_identity(&identity)
        .set_nonce(&nonce)
        .set_method(METHOD_ADD_DELEGATE)
        .set_endorsing_method(METHOD_ADD_DELEGATE_SIGNED)
        .add_param(delegate_type)?
        .add_param(delegate)?
        .add_param(&MethodUintBytesParam::from(validity.0))?
        .build(client)
        .await
}

/// Build a transaction to revoke a DID delegate (EthereumExtDidRegistry.revokeDelegate contract method)
/// An identity can assign multiple delegates to manage signing on their behalf for specific purposes.
///
/// #Params
///  - `client`: [LedgerClient] - client connected to the network where contract will be executed
///  - `sender`: [Address] - sender account address (Must be DID owner)
///  - `did`: [DID] - DID to revoke a delegate
///  - `delegate_type`: [DelegateType] - type of delegation (`veriKey` or `sigAuth`)
///  - `delegate`: [Address] - account address of delegate
///
/// #Returns
///   transaction: [Transaction] - prepared write transaction object to sign and submit
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_did_revoke_delegate_transaction(
    client: &LedgerClient,
    sender: &Address,
    did: &DID,
    delegate_type: &DelegateType,
    delegate: &Address,
) -> VdrResult<Transaction> {
    let identity = Address::try_from(did)?;
    TransactionBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_REVOKE_DELEGATE)
        .add_param(&identity)?
        .add_param(delegate_type)?
        .add_param(delegate)?
        .set_type(TransactionType::Write)
        .set_from(sender)
        .build(client)
        .await
}

/// Prepared data for endorsing revocation of a DID delegate (EthereumExtDidRegistry.revokeDelegateSigned contract method)
///
/// #Params
///  - `client`: [LedgerClient] - client connected to the network where contract will be executed
///  - `did`: [DID] - DID to add a delegate
///  - `delegate_type`: [DelegateType] - type of delegation (`veriKey` or `sigAuth`)
///  - `delegate`: [Address] - account address of delegate
///
/// #Returns
///   data: [TransactionEndorsingData] - transaction endorsement data to sign
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_did_revoke_delegate_endorsing_data(
    client: &LedgerClient,
    did: &DID,
    delegate_type: &DelegateType,
    delegate: &Address,
) -> VdrResult<TransactionEndorsingData> {
    let identity = Address::try_from(did)?;
    let nonce = resolve_identity_nonce(client, &identity).await?;
    TransactionEndorsingDataBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_identity(&identity)
        .set_nonce(&nonce)
        .set_method(METHOD_REVOKE_DELEGATE)
        .set_endorsing_method(METHOD_REVOKE_DELEGATE_SIGNED)
        .add_param(delegate_type)?
        .add_param(delegate)?
        .build(client)
        .await
}

/// Build a transaction to add a non ledger DID associated attribute (EthereumExtDidRegistry.setAttribute contract method)
/// An identity may need to publish some information that is only needed off-chain but
///   still requires the security benefits of using a blockchain.
///
/// #Params
///  - `client`: [LedgerClient] - client connected to the network where contract will be executed
///  - `sender`: [Address] - sender account address (Must be DID owner)
///  - `did`: [DID] - DID to add an attribute
///  - `attribute`: [DidDocAttribute] - attribute to add
///  - `validity`: [Validity] - attribute validity time
///
/// #Returns
///   transaction: [Transaction] - prepared write transaction object to sign and submit
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_did_set_attribute_transaction(
    client: &LedgerClient,
    sender: &Address,
    did: &DID,
    attribute: &DidDocAttribute,
    validity: &Validity,
) -> VdrResult<Transaction> {
    let identity = Address::try_from(did)?;
    TransactionBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_SET_ATTRIBUTE)
        .add_param(&identity)?
        .add_param(&attribute.name()?)?
        .add_param(&attribute.value()?)?
        .add_param(validity)?
        .set_type(TransactionType::Write)
        .set_from(sender)
        .build(client)
        .await
}

/// Prepared data for endorsing adding of a non ledger DID associated attribute (EthereumExtDidRegistry.setAttributeSigned contract method)
/// An identity may need to publish some information that is only needed off-chain but
///   still requires the security benefits of using a blockchain.
///
/// #Params
///  - `client`: [LedgerClient] - client connected to the network where contract will be executed
///  - `did`: [DID] - DID to add an attribute
///  - `attribute`: [DidDocAttribute] - attribute to add
///  - `validity`: [Validity] - attribute validity time
///
/// #Returns
///   transaction: [Transaction] - prepared write transaction object to sign and submit
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_did_set_attribute_endorsing_data(
    client: &LedgerClient,
    did: &DID,
    attribute: &DidDocAttribute,
    validity: &Validity,
) -> VdrResult<TransactionEndorsingData> {
    let identity = Address::try_from(did)?;
    let nonce = resolve_identity_nonce(client, &identity).await?;
    TransactionEndorsingDataBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_identity(&identity)
        .set_nonce(&nonce)
        .set_method(METHOD_SET_ATTRIBUTE)
        .set_endorsing_method(METHOD_SET_ATTRIBUTE_SIGNED)
        .add_param(&attribute.name()?)?
        .add_param(&attribute.value()?)?
        .add_param(validity)?
        .build(client)
        .await
}

/// Build a transaction to revoke a non ledger DID associated attribute (EthereumExtDidRegistry.revokeAttribute contract method)
///   a non ledger DID associated attribute.
/// An identity may need to publish some information that is only needed off-chain but
///   still requires the security benefits of using a blockchain.
///
/// #Params
///  - `client`: [LedgerClient] - client connected to the network where contract will be executed
///  - `sender`: [Address] - sender account address (Must be DID owner)
///  - `did`: [DID] - DID to revoke an attribute
///  - `attribute`: [DidDocAttribute] - attribute to revoke
///
/// #Returns
///   transaction: [Transaction] - prepared write transaction object to sign and submit
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_did_revoke_attribute_transaction(
    client: &LedgerClient,
    sender: &Address,
    did: &DID,
    attribute: &DidDocAttribute,
) -> VdrResult<Transaction> {
    let identity = Address::try_from(did)?;
    TransactionBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_REVOKE_ATTRIBUTE)
        .add_param(&identity)?
        .add_param(&attribute.name()?)?
        .add_param(&attribute.value()?)?
        .set_type(TransactionType::Write)
        .set_from(sender)
        .build(client)
        .await
}

/// Prepared data for endorsing revocation of a non ledger DID associated attribute (EthereumExtDidRegistry.revokeAttributeSigned contract method)
///
/// #Params
///  - `client`: [LedgerClient] - client connected to the network where contract will be executed
///  - `did`: [DID] - DID to add an attribute
///  - `attribute`: [DidDocAttribute] - attribute to revoke
///
/// #Returns
///   transaction: [Transaction] - prepared write transaction object to sign and submit
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_did_revoke_attribute_endorsing_data(
    client: &LedgerClient,
    did: &DID,
    attribute: &DidDocAttribute,
) -> VdrResult<TransactionEndorsingData> {
    let identity = Address::try_from(did)?;
    let nonce = resolve_identity_nonce(client, &identity).await?;
    TransactionEndorsingDataBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_identity(&identity)
        .set_nonce(&nonce)
        .set_method(METHOD_REVOKE_ATTRIBUTE)
        .set_endorsing_method(METHOD_REVOKE_ATTRIBUTE_SIGNED)
        .add_param(&attribute.name()?)?
        .add_param(&attribute.value()?)?
        .build(client)
        .await
}

/// Build a transaction to get an account address owning the DID (EthereumExtDidRegistry.owners contract method)
///
/// #Params
///  - `client`: [LedgerClient] - client connected to the network where contract will be executed
///  - `did`: [DID] - target DID
///
/// #Returns
///   transaction: [Transaction] - prepared read transaction object to submit
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_get_did_owner_transaction(
    client: &LedgerClient,
    did: &DID,
) -> VdrResult<Transaction> {
    let identity = Address::try_from(did)?;
    TransactionBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_DID_OWNER)
        .add_param(&identity)?
        .set_type(TransactionType::Read)
        .build(client)
        .await
}

/// Build a transaction to get block number when DID was changed last time (EthereumExtDidRegistry.changed contract method)
///
/// #Params
///  - `client`: [LedgerClient] - client connected to the network where contract will be executed
///  - `did`: [DID] - target DID
///
/// #Returns
///   transaction: [Transaction] - prepared read transaction object to submit
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_get_did_changed_transaction(
    client: &LedgerClient,
    did: &DID,
) -> VdrResult<Transaction> {
    let identity = Address::try_from(did)?;
    TransactionBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_DID_CHANGED)
        .add_param(&identity)?
        .set_type(TransactionType::Read)
        .build(client)
        .await
}

/// Build a transaction to get signing nonce needed for endorsement (EthereumExtDidRegistry.nonce contract method)
///
/// #Params
///  - `client`: [LedgerClient] - client connected to the network where contract will be executed
///  - `did`: [DID] - target DID
///
/// #Returns
///   transaction: [Transaction] - prepared read transaction object to submit
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_get_identity_nonce_transaction(
    client: &LedgerClient,
    identity: &Address,
) -> VdrResult<Transaction> {
    TransactionBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_DID_NONCE)
        .add_param(identity)?
        .set_type(TransactionType::Read)
        .build(client)
        .await
}

/// Build event query to obtain log DID associated events from the ledger
///
/// #Params
///  - `client`: [LedgerClient] - client connected to the network where contract will be executed
///  - `did`: [DID] - target DID
///  - `from_block`: [Block] - start block
///  - `to_block`: [Block] - finish block
///
/// #Returns
///   query: [EventQuery] - prepared event query to send
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn build_get_did_events_query(
    client: &LedgerClient,
    did: &DID,
    from_block: Option<&Block>,
    to_block: Option<&Block>,
) -> VdrResult<EventQuery> {
    let address = Address::try_from(did)?;
    EventQueryBuilder::new()
        .set_contract(CONTRACT_NAME)
        .set_from_block(from_block.cloned())
        .set_to_block(to_block.cloned())
        .set_event_filer(address.to_filter())
        .build(client)
}

/// Parse the result of execution EthereumExtDidRegistry.changed contract method to receive
///   a block number when DID was changed last time
///
/// # Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `bytes`: [Vec] - result bytes returned from the ledger
///
/// # Returns
///   block: [Block] Block number
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub fn parse_did_changed_result(client: &LedgerClient, bytes: &[u8]) -> VdrResult<Block> {
    TransactionParser::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_DID_CHANGED)
        .parse::<Block>(client, bytes)
}

/// Parse the result of execution EthereumExtDidRegistry.owners contract method to receive
///   an account address owning the DID.
///
/// # Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `bytes`: [Vec] - result bytes returned from the ledger
///
/// # Returns
///   owner: [Address] Owner account address
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub fn parse_did_owner_result(client: &LedgerClient, bytes: &[u8]) -> VdrResult<Address> {
    TransactionParser::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_DID_OWNER)
        .parse::<Address>(client, bytes)
}

/// Parse the result of execution EthereumExtDidRegistry.nonce contract method to receive
///   a signing nonce needed for endorsement
///
/// # Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `bytes`: [Vec] - result bytes returned from the ledger
///
/// # Returns
///   nonce: [Nonce] Nonce to use for endorsing
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub fn parse_did_nonce_result(client: &LedgerClient, bytes: &[u8]) -> VdrResult<Nonce> {
    TransactionParser::new()
        .set_contract(CONTRACT_NAME)
        .set_method(METHOD_DID_NONCE)
        .parse::<Nonce>(client, bytes)
}

/// Parse DidAttributeChangedEvent from the event log.
///
/// # Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `bytes`: [Vec] - result bytes returned from the ledger
///
/// # Returns
///   event: [DidAttributeChanged] - Parsed DidAttributeChanged event object
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub fn parse_did_attribute_changed_event_response(
    client: &LedgerClient,
    log: &EventLog,
) -> VdrResult<DidAttributeChanged> {
    EventParser::new()
        .set_contract(CONTRACT_NAME)
        .set_event(EVENT_DID_ATTRIBUTE_CHANGED)
        .parse::<DidAttributeChanged>(client, log)
}

/// Parse DidDelegateChangedEvent from the event log.
///
/// # Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `bytes`: [Vec] - result bytes returned from the ledger
///
/// # Returns
///   event: [DidDelegateChanged] Parsed DidDelegateChanged event object
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub fn parse_did_delegate_changed_event_response(
    client: &LedgerClient,
    log: &EventLog,
) -> VdrResult<DidDelegateChanged> {
    EventParser::new()
        .set_contract(CONTRACT_NAME)
        .set_event(EVENT_DID_DELEGATE_CHANGED)
        .parse::<DidDelegateChanged>(client, log)
}

/// Parse DidOwnerChangedEvent from the event log.
///
/// # Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `bytes`: [Vec] - result bytes returned from the ledger
///
/// # Returns
///   event: [DidOwnerChanged] Parsed DidOwnerChanged event object
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub fn parse_did_owner_changed_event_response(
    client: &LedgerClient,
    log: &EventLog,
) -> VdrResult<DidOwnerChanged> {
    EventParser::new()
        .set_contract(CONTRACT_NAME)
        .set_event(EVENT_DID_OWNER_CHANGED)
        .parse::<DidOwnerChanged>(client, log)
}

/// Parse DID associated event from the event log (it can be one of: DidAttributeChanged, DidDelegateChanged, DidOwnerChanged).
///
/// # Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `bytes`: [Vec] - result bytes returned from the ledger
///
/// # Returns
///   event: [DidEvents] Parsed DID event object
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub fn parse_did_event_response(client: &LedgerClient, event: &EventLog) -> VdrResult<DidEvents> {
    let contract = client.contract(CONTRACT_NAME)?;

    let event_signature = event.topics.first().ok_or_else(|| {
        VdrError::ContractInvalidResponseData("Unable to get event topic".to_string())
    })?;

    let did_attribute_changed_event_signature =
        contract.event(EVENT_DID_ATTRIBUTE_CHANGED)?.signature();
    let did_delegate_changed_event_signature =
        contract.event(EVENT_DID_DELEGATE_CHANGED)?.signature();
    let did_owner_changed_event_signature = contract.event(EVENT_DID_OWNER_CHANGED)?.signature();

    if event_signature.eq(&did_attribute_changed_event_signature) {
        return parse_did_attribute_changed_event_response(client, event)
            .map(DidEvents::AttributeChangedEvent);
    }

    if event_signature.eq(&did_delegate_changed_event_signature) {
        return parse_did_delegate_changed_event_response(client, event)
            .map(DidEvents::DelegateChanged);
    }

    if event_signature.eq(&did_owner_changed_event_signature) {
        return parse_did_owner_changed_event_response(client, event).map(DidEvents::OwnerChanged);
    }

    Err(VdrError::ContractInvalidResponseData(format!(
        "Unexpected contract event. Event signature: {:?}",
        event_signature
    )))
}

#[logfn(Info)]
#[logfn_inputs(Debug)]
async fn resolve_identity_nonce(client: &LedgerClient, identity: &Address) -> VdrResult<Nonce> {
    let transaction = build_get_identity_nonce_transaction(client, identity).await?;
    let response = client.submit_transaction(&transaction).await?;
    parse_did_nonce_result(client, &response)
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::{
        client::client::test::{mock_client, CONFIG, DEFAULT_NONCE, TEST_ACCOUNT, TRUSTEE_ACCOUNT},
        contracts::{
            did::types::{
                did::DID,
                did_doc::test::SERVICE_ENDPOINT,
                did_doc_attribute::{
                    PublicKeyAttribute, PublicKeyPurpose, PublicKeyType, ServiceAttribute,
                },
            },
            ServiceEndpoint, ServiceType,
        },
    };

    fn did() -> DID {
        DID::from(format!("did:ethr:{}", TEST_ACCOUNT.as_ref()).as_str())
    }

    pub fn service() -> DidDocAttribute {
        DidDocAttribute::Service(ServiceAttribute {
            type_: ServiceType::LinkedDomains,
            service_endpoint: ServiceEndpoint::String(SERVICE_ENDPOINT.to_string()),
        })
    }

    pub fn public_key() -> DidDocAttribute {
        DidDocAttribute::PublicKey(PublicKeyAttribute {
            purpose: PublicKeyPurpose::Enc,
            type_: PublicKeyType::X25519KeyAgreementKey2020,
            public_key_hex: None,
            public_key_base64: None,
            public_key_base58: Some("FbQWLPRhTH95MCkQUeFYdiSoQt8zMwetqfWoxqPgaq7x".to_string()),
            public_key_pem: None,
        })
    }

    pub fn public_key_2() -> DidDocAttribute {
        DidDocAttribute::PublicKey(PublicKeyAttribute {
            purpose: PublicKeyPurpose::Enc,
            type_: PublicKeyType::Ed25519VerificationKey2020,
            public_key_base58: Some("H3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV".to_string()),
            ..PublicKeyAttribute::default()
        })
    }

    pub fn public_key_3() -> DidDocAttribute {
        DidDocAttribute::PublicKey(PublicKeyAttribute {
            purpose: PublicKeyPurpose::VeriKey,
            type_: PublicKeyType::EcdsaSecp256k1VerificationKey2020,
            public_key_hex: Some(
                "02b97c30de767f084ce3080168ee293053ba33b235d7116a3263d29f1450936b71".to_string(),
            ),
            ..PublicKeyAttribute::default()
        })
    }

    pub fn validity() -> Validity {
        Validity::from(1000)
    }

    mod build_did_change_owner_transaction {
        use super::*;

        #[async_std::test]
        async fn build_did_change_owner_transaction_test() {
            let client = mock_client();
            let transaction = build_did_change_owner_transaction(
                &client,
                &TEST_ACCOUNT,
                &did(),
                &TRUSTEE_ACCOUNT,
            )
            .await
            .unwrap();
            let expected_transaction = Transaction {
                type_: TransactionType::Write,
                from: Some(TEST_ACCOUNT.clone()),
                to: CONFIG.contracts.ethereum_did_registry.address.clone(),
                nonce: Some(DEFAULT_NONCE.clone()),
                chain_id: CONFIG.chain_id,
                data: vec![
                    240, 13, 75, 93, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 240, 226, 219, 108, 141,
                    198, 198, 129, 187, 93, 106, 209, 33, 161, 7, 243, 0, 233, 178, 181, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 240, 226, 219, 108, 141, 198, 198, 129, 187, 93,
                    106, 209, 33, 161, 7, 243, 0, 233, 178, 181,
                ],
                signature: None,
                hash: None,
            };
            assert_eq!(expected_transaction, transaction);
        }
    }

    mod build_did_add_delegate_transaction {
        use super::*;

        #[async_std::test]
        async fn build_did_add_delegate_transaction_test() {
            let client = mock_client();
            let transaction = build_did_add_delegate_transaction(
                &client,
                &TEST_ACCOUNT,
                &did(),
                &DelegateType::VeriKey,
                &TRUSTEE_ACCOUNT,
                &validity(),
            )
            .await
            .unwrap();
            let expected_transaction = Transaction {
                type_: TransactionType::Write,
                from: Some(TEST_ACCOUNT.clone()),
                to: CONFIG.contracts.ethereum_did_registry.address.clone(),
                nonce: Some(DEFAULT_NONCE.clone()),
                chain_id: CONFIG.chain_id,
                data: vec![
                    167, 6, 141, 102, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 240, 226, 219, 108, 141,
                    198, 198, 129, 187, 93, 106, 209, 33, 161, 7, 243, 0, 233, 178, 181, 118, 101,
                    114, 105, 75, 101, 121, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 240, 226, 219, 108,
                    141, 198, 198, 129, 187, 93, 106, 209, 33, 161, 7, 243, 0, 233, 178, 181, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 3, 232,
                ],
                signature: None,
                hash: None,
            };
            assert_eq!(expected_transaction, transaction);
        }
    }

    mod build_did_revoke_delegate_transaction {
        use super::*;

        #[async_std::test]
        async fn build_did_revoke_delegate_transaction_test() {
            let client = mock_client();
            let transaction = build_did_revoke_delegate_transaction(
                &client,
                &TEST_ACCOUNT,
                &did(),
                &DelegateType::VeriKey,
                &TRUSTEE_ACCOUNT,
            )
            .await
            .unwrap();
            let expected_transaction = Transaction {
                type_: TransactionType::Write,
                from: Some(TEST_ACCOUNT.clone()),
                to: CONFIG.contracts.ethereum_did_registry.address.clone(),
                nonce: Some(DEFAULT_NONCE.clone()),
                chain_id: CONFIG.chain_id,
                data: vec![
                    128, 178, 159, 124, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 240, 226, 219, 108,
                    141, 198, 198, 129, 187, 93, 106, 209, 33, 161, 7, 243, 0, 233, 178, 181, 118,
                    101, 114, 105, 75, 101, 121, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 240, 226, 219, 108,
                    141, 198, 198, 129, 187, 93, 106, 209, 33, 161, 7, 243, 0, 233, 178, 181,
                ],
                signature: None,
                hash: None,
            };
            assert_eq!(expected_transaction, transaction);
        }
    }

    mod build_did_set_attribute_transaction {
        use super::*;

        #[async_std::test]
        async fn build_did_set_service_attribute_transaction_test() {
            let client = mock_client();
            let transaction = build_did_set_attribute_transaction(
                &client,
                &TEST_ACCOUNT,
                &did(),
                &service(),
                &validity(),
            )
            .await
            .unwrap();
            let expected_transaction = Transaction {
                type_: TransactionType::Write,
                from: Some(TEST_ACCOUNT.clone()),
                to: CONFIG.contracts.ethereum_did_registry.address.clone(),
                nonce: Some(DEFAULT_NONCE.clone()),
                chain_id: CONFIG.chain_id,
                data: vec![
                    122, 212, 176, 164, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 240, 226, 219, 108,
                    141, 198, 198, 129, 187, 93, 106, 209, 33, 161, 7, 243, 0, 233, 178, 181, 100,
                    105, 100, 47, 115, 118, 99, 47, 76, 105, 110, 107, 101, 100, 68, 111, 109, 97,
                    105, 110, 115, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    3, 232, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 18, 104, 116, 116, 112, 58, 47, 47, 101, 120, 97, 109,
                    112, 108, 101, 46, 99, 111, 109, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ],
                signature: None,
                hash: None,
            };
            assert_eq!(expected_transaction, transaction);
        }

        #[async_std::test]
        async fn build_did_set_key_attribute_transaction_test() {
            let client = mock_client();
            let transaction = build_did_set_attribute_transaction(
                &client,
                &TEST_ACCOUNT,
                &did(),
                &public_key(),
                &validity(),
            )
            .await
            .unwrap();
            let expected_transaction = Transaction {
                type_: TransactionType::Write,
                from: Some(TEST_ACCOUNT.clone()),
                to: CONFIG.contracts.ethereum_did_registry.address.clone(),
                nonce: Some(DEFAULT_NONCE.clone()),
                chain_id: CONFIG.chain_id,
                data: vec![
                    122, 212, 176, 164, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 240, 226, 219, 108,
                    141, 198, 198, 129, 187, 93, 106, 209, 33, 161, 7, 243, 0, 233, 178, 181, 100,
                    105, 100, 47, 112, 117, 98, 47, 88, 50, 53, 53, 49, 57, 47, 101, 110, 99, 47,
                    98, 97, 115, 101, 53, 56, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3,
                    232, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 32, 216, 211, 241, 73, 138, 202, 20, 247, 254, 92, 152, 102,
                    20, 103, 236, 224, 41, 108, 66, 163, 228, 133, 29, 248, 18, 225, 230, 17, 163,
                    84, 230, 43,
                ],
                signature: None,
                hash: None,
            };
            assert_eq!(expected_transaction, transaction);
        }
    }

    mod build_did_revoke_attribute_transaction {
        use super::*;

        #[async_std::test]
        async fn build_did_revoke_attribute_transaction_test() {
            let client = mock_client();
            let transaction =
                build_did_revoke_attribute_transaction(&client, &TEST_ACCOUNT, &did(), &service())
                    .await
                    .unwrap();
            let expected_transaction = Transaction {
                type_: TransactionType::Write,
                from: Some(TEST_ACCOUNT.clone()),
                to: CONFIG.contracts.ethereum_did_registry.address.clone(),
                nonce: Some(DEFAULT_NONCE.clone()),
                chain_id: CONFIG.chain_id,
                data: vec![
                    0, 192, 35, 218, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 240, 226, 219, 108, 141,
                    198, 198, 129, 187, 93, 106, 209, 33, 161, 7, 243, 0, 233, 178, 181, 100, 105,
                    100, 47, 115, 118, 99, 47, 76, 105, 110, 107, 101, 100, 68, 111, 109, 97, 105,
                    110, 115, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 96, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 18,
                    104, 116, 116, 112, 58, 47, 47, 101, 120, 97, 109, 112, 108, 101, 46, 99, 111,
                    109, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ],
                signature: None,
                hash: None,
            };
            assert_eq!(expected_transaction, transaction);
        }
    }
}
