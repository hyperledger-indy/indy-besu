// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use chrono::Utc;
use log_derive::{logfn, logfn_inputs};

use crate::{
    contracts::{
        did::{types::did_doc_attribute::PublicKeyPurpose, DidResolutionError},
        types::did::ParsedDid,
        DidDocumentWithMeta, DidMetadata, DidRecord, DidResolutionMetadata, DID_RESOLUTION_FORMAT,
    },
    did_ethr_registry::{
        build_get_did_changed_transaction, build_get_did_events_query, parse_did_changed_result,
        parse_did_event_response, ETHR_DID_METHOD,
    },
    did_indy_registry::{
        build_resolve_did_transaction, parse_resolve_did_result, INDYBESU_DID_METHOD,
    },
    Block, DelegateType, DidAttributeChanged, DidDelegateChanged, DidDocAttribute,
    DidDocumentBuilder, DidEvents, DidOwnerChanged, DidResolutionOptions, LedgerClient, VdrError,
    VdrResult, VerificationKeyType, DID,
};

/// Single step function to resolve a DidDocument with metadata for the given DID
///
/// # Params
/// - `client`: [LedgerClient] - client connected to the network where contract will be executed
/// - `did`: [DID] - DID to get a DID Document and metadata
/// - `options`: [DidResolutionOptions] - Resolution options
///
/// # Returns
///   did_with_meta: [DidDocumentWithMeta] - resolved DID Document with associate metadata
#[logfn(Info)]
#[logfn_inputs(Debug)]
pub async fn resolve_did(
    client: &LedgerClient,
    did: &DID,
    options: Option<&DidResolutionOptions>,
) -> VdrResult<DidDocumentWithMeta> {
    // Parse DID
    let parsed_did = match ParsedDid::try_from(did) {
        Ok(did) => did,
        Err(_) => {
            return Ok(DidDocumentWithMeta {
                did_document: None,
                did_document_metadata: DidMetadata::default(),
                did_resolution_metadata: DidResolutionMetadata {
                    content_type: None,
                    error: Some(DidResolutionError::InvalidDid),
                    message: Some(format!("Not a valid did: {}", did.as_ref())),
                },
            });
        }
    };

    let accept = options.and_then(|options| options.accept.as_deref());
    let block_tag = options.and_then(|options| options.block_tag.as_ref());

    match accept {
        Some(DID_RESOLUTION_FORMAT) | None => {
            // ok
        }
        Some(accept) => {
            return Ok(DidDocumentWithMeta {
                did_document: None,
                did_document_metadata: DidMetadata::default(),
                did_resolution_metadata: DidResolutionMetadata {
                    content_type: None,
                    error: Some(DidResolutionError::RepresentationNotSupported),
                    message: Some(format!(
                        "VDR does not support the requested 'accept' format: {}",
                        accept
                    )),
                },
            });
        }
    }

    let did = parsed_did.as_short_did();

    let resolve_result = match parsed_did.method.as_str() {
        INDYBESU_DID_METHOD => indybesu::resolve(client, &did).await,
        ETHR_DID_METHOD => ethr::resolve(client, &did, block_tag).await,
        _ => {
            return Ok(DidDocumentWithMeta {
                did_document: None,
                did_document_metadata: DidMetadata::default(),
                did_resolution_metadata: DidResolutionMetadata {
                    content_type: None,
                    error: Some(DidResolutionError::MethodNotSupported),
                    message: Some(format!(
                        "DID Method is not supported: {}",
                        parsed_did.method
                    )),
                },
            });
        }
    };

    match resolve_result {
        Ok(did_record) => Ok(DidDocumentWithMeta {
            did_document: Some(did_record.document),
            did_document_metadata: did_record.metadata,
            did_resolution_metadata: DidResolutionMetadata {
                content_type: accept.map(String::from),
                error: None,
                message: None,
            },
        }),
        Err(err) => Ok(DidDocumentWithMeta {
            did_document: None,
            did_document_metadata: DidMetadata::default(),
            did_resolution_metadata: DidResolutionMetadata {
                content_type: None,
                error: Some(DidResolutionError::NotFound),
                message: Some(err.to_string()),
            },
        }),
    }
}

mod indybesu {
    use super::*;

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub(crate) async fn resolve(client: &LedgerClient, did: &DID) -> VdrResult<DidRecord> {
        let transaction = build_resolve_did_transaction(client, did).await?;
        let response = client.submit_transaction(&transaction).await?;
        if response.is_empty() {
            return Err(VdrError::ClientInvalidResponse(format!(
                "DID not found: {:?}",
                did
            )));
        }
        parse_resolve_did_result(client, &response)
    }
}

mod ethr {
    use super::*;

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub(crate) async fn resolve(
        client: &LedgerClient,
        did: &DID,
        block: Option<&Block>,
    ) -> VdrResult<DidRecord> {
        // Build base DID document for ethr DID
        let mut did_doc_builder = DidDocumentBuilder::base_for_did(did, client.chain_id())?;

        // TODO: support the case when DID identifier is public key

        // Query block number when DID was changed last time
        let did_changed_block = get_did_changed_block(client, did).await?;

        // if DID has not been ever changed, we do not need to query events and just return base did document
        if did_changed_block.is_none() {
            let document = did_doc_builder.build();
            return Ok(DidRecord {
                document,
                metadata: DidMetadata::default(),
            });
        }

        let mut version_id: Option<Block> = None;
        let mut next_version_id: Option<Block> = None;

        // time in seconds for attributes validity check
        let now = match block {
            Some(block) => {
                // request block time if the resolution happens for specific block
                client.get_block(Some(block)).await?.timestamp
            }
            None => {
                // else current time
                Utc::now().timestamp() as u64
            }
        };

        // request events for a specific block until previous exists
        let did_history = receive_did_history(client, did, did_changed_block).await?;

        // assemble Did Document from the history events
        //  iterate in the reverse order -> oldest to newest
        for (event_block, event) in did_history.into_iter().rev() {
            match block {
                // if we resolve DID for specific block we need to skip all blocks higher
                Some(block) if event_block.value() > block.value() => {
                    if next_version_id.is_none() {
                        next_version_id = Some(event_block)
                    }
                    continue;
                }
                _ => {
                    version_id = Some(event_block);
                }
            }

            // handle event
            handle_did_event(&mut did_doc_builder, &event, client, now)?;

            // break for deactivate DID -> minimal DID Document will be returned
            if did_doc_builder.deactivated() {
                break;
            }
        }

        let metadata = build_did_metadata(
            client,
            did_doc_builder.deactivated(),
            version_id.as_ref(),
            next_version_id.as_ref(),
        )
        .await?;
        Ok(DidRecord {
            document: did_doc_builder.build(),
            metadata,
        })
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    async fn get_did_changed_block(client: &LedgerClient, did: &DID) -> VdrResult<Block> {
        let transaction = build_get_did_changed_transaction(client, did).await?;
        let response = client.submit_transaction(&transaction).await?;
        parse_did_changed_result(client, &response)
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    async fn receive_did_history(
        client: &LedgerClient,
        did: &DID,
        first_block: Block,
    ) -> VdrResult<Vec<(Block, DidEvents)>> {
        let mut history: Vec<(Block, DidEvents)> = Vec::new();
        let mut previous_block: Option<Block> = Some(first_block);
        while previous_block.is_some() {
            let transaction = build_get_did_events_query(
                client,
                did,
                previous_block.as_ref(),
                previous_block.as_ref(),
            )
            .await?;
            let logs = client.query_events(&transaction).await?;

            // if no logs, break the loop as nothing to add to the change history
            if logs.is_empty() {
                break;
            }

            // parse events
            for log in logs {
                let event = parse_did_event_response(client, &log)?;
                previous_block = Some(event.previous_change());
                history.push((log.block, event));
            }
        }
        Ok(history)
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    fn handle_did_owner_changed(
        did_doc_builder: &mut DidDocumentBuilder,
        event: &DidOwnerChanged,
    ) -> VdrResult<()> {
        if event.owner.is_null() {
            // DID is considered to be deactivated
            did_doc_builder.deactivate();
            return Ok(());
        }

        let controller = DID::build(ETHR_DID_METHOD, None, event.owner.as_ref());
        did_doc_builder.set_controller(controller.as_ref());
        Ok(())
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    fn handle_did_delegate_changed(
        did_doc_builder: &mut DidDocumentBuilder,
        event: &DidDelegateChanged,
        now: u64,
        client: &LedgerClient,
    ) -> VdrResult<()> {
        let event_index = event.key();
        let delegate_type = DelegateType::try_from(event.delegate_type.as_slice())?;

        if event.valid_to > now {
            did_doc_builder.add_delegate_key(
                &event_index,
                &VerificationKeyType::EcdsaSecp256k1RecoveryMethod2020,
                Some(event.delegate.as_blockchain_id(client.chain_id()).as_str()),
                None,
                None,
                None,
                None,
                None,
            );

            match delegate_type {
                DelegateType::VeriKey => {
                    did_doc_builder.add_assertion_method_reference(&event_index)?;
                }
                DelegateType::SigAuth => {
                    did_doc_builder.add_authentication_reference(&event_index)?;
                }
            }
        } else {
            // delegate expired
            did_doc_builder.remove_delegate_key(&event_index)?;
            match delegate_type {
                DelegateType::VeriKey => {
                    did_doc_builder.remove_assertion_method_reference(&event_index)?;
                }
                DelegateType::SigAuth => {
                    did_doc_builder.remove_authentication_reference(&event_index)?;
                }
            }
        };
        Ok(())
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    fn handle_did_attribute_changed(
        did_doc_builder: &mut DidDocumentBuilder,
        event: &DidAttributeChanged,
        now: u64,
    ) -> VdrResult<()> {
        let event_index = event.key();
        let attribute = DidDocAttribute::try_from(event)?;

        match attribute {
            DidDocAttribute::PublicKey(key) => {
                if event.valid_to > now {
                    did_doc_builder.add_delegate_key(
                        &event_index,
                        &key.type_.into(),
                        None,
                        None,
                        key.public_key_hex.as_deref(),
                        key.public_key_base58.as_deref(),
                        key.public_key_base64.as_deref(),
                        None,
                    );

                    match key.purpose {
                        PublicKeyPurpose::VeriKey => {
                            did_doc_builder.add_assertion_method_reference(&event_index)?;
                        }
                        PublicKeyPurpose::SigAuth => {
                            did_doc_builder.add_authentication_reference(&event_index)?;
                        }
                        PublicKeyPurpose::Enc => {
                            did_doc_builder.add_key_agreement_reference(&event_index)?;
                        }
                    }
                } else {
                    // key expired
                    did_doc_builder.remove_delegate_key(&event_index)?;
                    match key.purpose {
                        PublicKeyPurpose::VeriKey => {
                            did_doc_builder.remove_assertion_method_reference(&event_index)?;
                        }
                        PublicKeyPurpose::SigAuth => {
                            did_doc_builder.remove_authentication_reference(&event_index)?;
                        }
                        PublicKeyPurpose::Enc => {
                            did_doc_builder.remove_key_agreement_reference(&event_index)?;
                        }
                    }
                }
            }
            DidDocAttribute::Service(service) => {
                if event.valid_to > now {
                    did_doc_builder.add_service(
                        &event_index,
                        None,
                        &service.type_,
                        &service.service_endpoint,
                    );
                } else {
                    did_doc_builder.remove_service(&event_index)?;
                }
            }
        };
        Ok(())
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    fn handle_did_event(
        did_doc_builder: &mut DidDocumentBuilder,
        event: &DidEvents,
        client: &LedgerClient,
        now: u64,
    ) -> VdrResult<()> {
        match event {
            DidEvents::OwnerChanged(event) => handle_did_owner_changed(did_doc_builder, event),
            DidEvents::DelegateChanged(event) => {
                handle_did_delegate_changed(did_doc_builder, event, now, client)
            }
            DidEvents::AttributeChangedEvent(event) => {
                handle_did_attribute_changed(did_doc_builder, event, now)
            }
        }
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    async fn build_did_metadata(
        client: &LedgerClient,
        deactivated: bool,
        version_id: Option<&Block>,
        next_version_id: Option<&Block>,
    ) -> VdrResult<DidMetadata> {
        let (updated, version_id) = match version_id {
            Some(version_id) => {
                let block = client.get_block(Some(version_id)).await?;
                (Some(block.timestamp), Some(block.number))
            }
            None => (None, None),
        };

        let (next_update, next_version_id) = match next_version_id {
            Some(next_version_id) => {
                let block = client.get_block(Some(next_version_id)).await?;
                (Some(block.timestamp), Some(block.number))
            }
            None => (None, None),
        };

        Ok(DidMetadata {
            owner: None,
            created: None,
            deactivated: Some(deactivated),
            updated,
            version_id,
            next_update,
            next_version_id,
        })
    }
}
