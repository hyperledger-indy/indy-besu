// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

pub mod credential_definition_registry;
pub mod revocation_registry;
pub mod schema_registry;
pub mod types;

pub use types::{schema::Schema, schema_id::SchemaId};

pub use types::{
    credential_definition::CredentialDefinition, credential_definition_id::CredentialDefinitionId,
};
