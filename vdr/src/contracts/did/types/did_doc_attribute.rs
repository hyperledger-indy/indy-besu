// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{
    contracts::{
        did::types::{
            did_doc::ServiceType,
            did_events::{DidAttributeChanged, DidEvents},
        },
        ServiceEndpoint,
    },
    types::ContractParam,
    utils::{format_bytes32_string, parse_bytes32_string},
    VdrError, VdrResult, VerificationKeyType,
};
use base64::Engine;
use ethabi::Uint;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::str::from_utf8;

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicKeyAttribute {
    pub purpose: PublicKeyPurpose,
    #[serde(rename = "type")]
    pub type_: PublicKeyType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key_hex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key_base64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key_base58: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key_pem: Option<String>,
}

impl PublicKeyAttribute {
    pub fn encoding(&self) -> VdrResult<&str> {
        if self.public_key_base58.is_some() {
            return Ok("base58");
        }
        if self.public_key_hex.is_some() {
            return Ok("hex");
        }
        if self.public_key_base64.is_some() {
            return Ok("base64");
        }
        Err(VdrError::ContractInvalidInputData)
    }

    pub fn value(&self) -> VdrResult<Vec<u8>> {
        if let Some(ref public_key_base58) = self.public_key_base58 {
            return bs58::decode(&public_key_base58).into_vec().map_err(|err| {
                VdrError::CommonInvalidData(format!(
                    "Unable to decode base58 public key. Err: {:?}",
                    err
                ))
            });
        }
        if let Some(ref public_key_hex) = self.public_key_hex {
            return hex::decode(public_key_hex).map_err(|err| {
                VdrError::CommonInvalidData(format!(
                    "Unable to decode hex public key. Err: {:?}",
                    err
                ))
            });
        }
        if let Some(ref public_key_base64) = self.public_key_base64 {
            return base64::engine::general_purpose::URL_SAFE_NO_PAD
                .decode(public_key_base64)
                .map_err(|err| {
                    VdrError::CommonInvalidData(format!(
                        "Unable to decode base64 public key. Err: {:?}",
                        err
                    ))
                });
        }
        Err(VdrError::ContractInvalidInputData)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub enum PublicKeyPurpose {
    #[default]
    #[serde(rename = "veriKey")]
    VeriKey,
    #[serde(rename = "sigAuth")]
    SigAuth,
    #[serde(rename = "enc")]
    Enc,
}

impl AsRef<str> for PublicKeyPurpose {
    fn as_ref(&self) -> &str {
        match self {
            PublicKeyPurpose::VeriKey => "veriKey",
            PublicKeyPurpose::SigAuth => "sigAuth",
            PublicKeyPurpose::Enc => "enc",
        }
    }
}

impl TryFrom<&str> for PublicKeyPurpose {
    type Error = VdrError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "veriKey" => Ok(PublicKeyPurpose::VeriKey),
            "sigAuth" => Ok(PublicKeyPurpose::SigAuth),
            "enc" => Ok(PublicKeyPurpose::Enc),
            value => Err(VdrError::CommonInvalidData(format!(
                "Unexpected public key purpose {}",
                value
            ))),
        }
    }
}

/// Enum listing possible DID delegate types
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub enum DelegateType {
    #[default]
    #[serde(rename = "veriKey")]
    VeriKey,
    #[serde(rename = "sigAuth")]
    SigAuth,
}

impl AsRef<str> for DelegateType {
    fn as_ref(&self) -> &str {
        match self {
            DelegateType::VeriKey => "veriKey",
            DelegateType::SigAuth => "sigAuth",
        }
    }
}

impl TryFrom<&str> for DelegateType {
    type Error = VdrError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "veriKey" => Ok(DelegateType::VeriKey),
            "sigAuth" => Ok(DelegateType::SigAuth),
            value => Err(VdrError::CommonInvalidData(format!(
                "Unexpected public key delegate type {}",
                value
            ))),
        }
    }
}

impl TryFrom<&DelegateType> for ContractParam {
    type Error = VdrError;

    fn try_from(value: &DelegateType) -> Result<Self, Self::Error> {
        let name = format_bytes32_string(value.as_ref())?.to_vec();
        Ok(ContractParam::FixedBytes(name))
    }
}

impl TryFrom<&[u8]> for DelegateType {
    type Error = VdrError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let str = parse_bytes32_string(value)?;
        Self::try_from(str)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub enum PublicKeyType {
    #[default]
    Ed25519VerificationKey2020,
    X25519KeyAgreementKey2020,
    EcdsaSecp256k1VerificationKey2020,
}

impl PublicKeyType {
    pub fn to_name(&self) -> &str {
        match self {
            PublicKeyType::EcdsaSecp256k1VerificationKey2020 => "Secp256k1",
            PublicKeyType::Ed25519VerificationKey2020 => "Ed25519",
            PublicKeyType::X25519KeyAgreementKey2020 => "X25519",
        }
    }

    pub fn from_name(name: &str) -> VdrResult<Self> {
        match name {
            "Secp256k1" => Ok(PublicKeyType::EcdsaSecp256k1VerificationKey2020),
            "Ed25519" => Ok(PublicKeyType::Ed25519VerificationKey2020),
            "X25519" => Ok(PublicKeyType::X25519KeyAgreementKey2020),
            value => Err(VdrError::CommonInvalidData(format!(
                "Unexpected public key type {}",
                value
            ))),
        }
    }
}

impl From<PublicKeyType> for VerificationKeyType {
    fn from(value: PublicKeyType) -> Self {
        match value {
            PublicKeyType::EcdsaSecp256k1VerificationKey2020 => {
                VerificationKeyType::EcdsaSecp256k1VerificationKey2020
            }
            PublicKeyType::Ed25519VerificationKey2020 => {
                VerificationKeyType::Ed25519VerificationKey2020
            }
            PublicKeyType::X25519KeyAgreementKey2020 => {
                VerificationKeyType::X25519KeyAgreementKey2020
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceAttribute {
    #[serde(rename = "type")]
    pub type_: ServiceType,
    pub service_endpoint: ServiceEndpoint,
}

/// Enum listing attributes which can be associated with a DID
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum DidDocAttribute {
    PublicKey(PublicKeyAttribute),
    Service(ServiceAttribute),
}

impl DidDocAttribute {
    pub(crate) fn name(&self) -> VdrResult<DidDocAttributeName> {
        match self {
            DidDocAttribute::PublicKey(key) => {
                let key_encoding = key.encoding()?;
                Ok(DidDocAttributeName(format!(
                    "did/pub/{}/{}/{}",
                    key.type_.to_name(),
                    key.purpose.as_ref(),
                    key_encoding
                )))
            }
            DidDocAttribute::Service(service) => Ok(DidDocAttributeName(format!(
                "did/svc/{}",
                service.type_.to_string()
            ))),
        }
    }

    pub(crate) fn value(&self) -> VdrResult<DidDocAttributeValue> {
        match self {
            DidDocAttribute::PublicKey(key) => Ok(DidDocAttributeValue(key.value()?)),
            DidDocAttribute::Service(ref service) => {
                let value = match &service.service_endpoint {
                    ServiceEndpoint::String(endpoint) => endpoint.to_string(),
                    service => json!(service).to_string(),
                };
                Ok(DidDocAttributeValue(value.as_bytes().to_vec()))
            }
        }
    }
}

impl TryFrom<DidEvents> for DidDocAttribute {
    type Error = VdrError;

    fn try_from(value: DidEvents) -> Result<Self, Self::Error> {
        match value {
            DidEvents::AttributeChangedEvent(event) => {
                DidDocAttribute::try_from(&event).map_err(VdrError::from)
            }
            _ => Err(VdrError::CommonInvalidData(
                "Unable to get DidDocAttribute from event.".to_string(),
            )),
        }
    }
}

impl TryFrom<&DidAttributeChanged> for DidDocAttribute {
    type Error = VdrError;

    fn try_from(event: &DidAttributeChanged) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = event.name.split('/').collect();

        let kind = parts.get(1).ok_or_else(|| {
            VdrError::CommonInvalidData(
                "Unable to convert DIDAttributeChangedEvent into DidDocAttribute".to_string(),
            )
        })?;

        match *kind {
            "pub" => {
                // key attribute
                let type_ = parts.get(2).ok_or_else(|| VdrError::CommonInvalidData(
                    format!("Unable to convert DIDAttributeChangedEvent into public key DidDocAttribute. `type` not found {}", event.name)
                ))?;
                let purpose = parts.get(3).ok_or_else(|| VdrError::CommonInvalidData(
                    format!("Unable to convert DIDAttributeChangedEvent into public key DidDocAttribute. `purpose` not found {}", event.name)
                ))?;
                let encoding = parts.get(4).ok_or_else(|| VdrError::CommonInvalidData(
                    format!("Unable to convert DIDAttributeChangedEvent into public key DidDocAttribute. `encoding` not found {}", event.name)
                ))?;

                let mut public_key = PublicKeyAttribute {
                    purpose: PublicKeyPurpose::try_from(*purpose)?,
                    type_: PublicKeyType::from_name(type_)?,
                    public_key_hex: None,
                    public_key_base64: None,
                    public_key_base58: None,
                    public_key_pem: None,
                };

                match *encoding {
                    "base58" => {
                        public_key.public_key_base58 =
                            Some(bs58::encode(&event.value).into_string());
                    }
                    "hex" => {
                        public_key.public_key_hex = Some(hex::encode(&event.value));
                    }
                    "base64" => {
                        let base64_encoded_key =
                            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&event.value);
                        public_key.public_key_base64 = Some(base64_encoded_key);
                    }
                    value => {
                        return Err(VdrError::CommonInvalidData(
                            format!("Unable to convert DIDAttributeChangedEvent into public key DidDocAttribute. `encoding` not found {}", value)
                        ));
                    }
                }

                Ok(DidDocAttribute::PublicKey(public_key))
            }
            "svc" => {
                // service attribute
                let type_ = parts.get(2).ok_or_else(|| VdrError::CommonInvalidData(
                    format!("Unable to convert DIDAttributeChangedEvent into service DidDocAttribute. `type` not found {}", event.name)
                ))?;

                let value = from_utf8(&event.value)
                    .map_err(|err| VdrError::CommonInvalidData(
                        format!("Unable to convert DIDAttributeChangedEvent into service DidDocAttribute. Failed to parse value: {}", err)
                    ))?;

                let service_endpoint = if value.starts_with('{') {
                    // if JSON
                    serde_json::from_slice(&event.value)
                        .map_err(|err| VdrError::CommonInvalidData(
                            format!("Unable to convert DIDAttributeChangedEvent into service DidDocAttribute. Failed to parse value: {}", err)
                        ))?
                } else {
                    ServiceEndpoint::String(value.to_string())
                };

                Ok(DidDocAttribute::Service(ServiceAttribute {
                    type_: ServiceType::try_from(*type_)?,
                    service_endpoint,
                }))
            }
            val => Err(VdrError::CommonInvalidData(format!(
                "Unable to convert DIDAttributeChangedEvent into DidDocAttribute. Unknown kind: {}",
                val
            ))),
        }
    }
}

#[derive(Debug)]
pub(crate) struct DidDocAttributeName(pub(crate) String);

impl TryFrom<&DidDocAttributeName> for ContractParam {
    type Error = VdrError;

    fn try_from(value: &DidDocAttributeName) -> Result<Self, Self::Error> {
        let name = format_bytes32_string(&value.0)?.to_vec();
        Ok(ContractParam::FixedBytes(name))
    }
}

#[derive(Debug)]
pub(crate) struct DidDocAttributeValue(pub(crate) Vec<u8>);

impl TryFrom<&DidDocAttributeValue> for ContractParam {
    type Error = VdrError;

    fn try_from(value: &DidDocAttributeValue) -> Result<Self, Self::Error> {
        Ok(ContractParam::Bytes(value.0.to_vec()))
    }
}

/// Wrapper structure for DID attribute validity time in seconds
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct Validity(pub(crate) u64);

impl TryFrom<&Validity> for ContractParam {
    type Error = VdrError;

    fn try_from(value: &Validity) -> Result<Self, Self::Error> {
        Ok(ContractParam::Uint(Uint::from(value.0)))
    }
}

impl From<u64> for Validity {
    fn from(value: u64) -> Self {
        Validity(value)
    }
}
