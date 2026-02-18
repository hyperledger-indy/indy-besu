// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{JsonValue, LedgerClient, VdrError, VdrResult};
use serde_json::json;

use indy_besu_vdr::{did_resolver, Block, DidResolutionOptions as DidResolutionOptions_, DID};

#[uniffi::export(async_runtime = "tokio")]
pub async fn resolve_did(
    client: &LedgerClient,
    did: &str,
    options: Option<DidResolutionOptions>,
) -> VdrResult<JsonValue> {
    let options = match options {
        Some(options) => Some(DidResolutionOptions_::try_from(options)?),
        None => None,
    };
    let did_with_meta =
        did_resolver::resolve_did(&client.client, &DID::from(did), options.as_ref()).await?;
    Ok(JsonValue::from(json!(did_with_meta)))
}

#[derive(uniffi::Record)]
pub struct DidResolutionOptions {
    pub accept: Option<String>,
    pub block_tag: Option<u64>,
}

impl TryFrom<DidResolutionOptions> for DidResolutionOptions_ {
    type Error = VdrError;

    fn try_from(value: DidResolutionOptions) -> Result<Self, Self::Error> {
        Ok(DidResolutionOptions_ {
            accept: value.accept,
            block_tag: value.block_tag.map(|block_tag| Block::from(block_tag)),
        })
    }
}
