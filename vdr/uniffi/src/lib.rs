use anyhow::Error;
use serde_json::Value;
use std::convert::TryFrom;

#[allow(clippy::module_inception)]
mod ffi;

pub use ffi::*;
use uniffi::deps::anyhow;

/// Wrapper para serde_json::Value usado pelo UniFFI
#[derive(Clone, Debug)]
pub struct JsonValue(pub Value);

impl JsonValue {
    /// Consome o wrapper e devolve o Value interno
    pub fn into_inner(self) -> Value {
        self.0
    }

    /// Referência ao Value interno (para usar sem mover)
    pub fn as_inner(&self) -> &Value {
        &self.0
    }
}

// String -> JsonValue (pode falhar)
impl TryFrom<String> for JsonValue {
    type Error = Error;
    fn try_from(val: String) -> Result<Self, Self::Error> {
        Ok(JsonValue(serde_json::from_str(&val)?))
    }
}

// JsonValue -> String (sempre deve funcionar)
impl From<JsonValue> for String {
    fn from(val: JsonValue) -> Self {
        serde_json::to_string(&val.0).expect("unable to unwrap json value")
    }
}

// serde_json::Value -> JsonValue
impl From<Value> for JsonValue {
    fn from(v: Value) -> Self {
        JsonValue(v)
    }
}

// JsonValue -> serde_json::Value
impl From<JsonValue> for Value {
    fn from(v: JsonValue) -> Self {
        v.0
    }
}

// Acesso conveniente (como Value)
impl std::ops::Deref for JsonValue {
    type Target = Value;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Mapeamento para UniFFI
uniffi::custom_type!(JsonValue, String);
uniffi::include_scaffolding!("indy_besu_vdr");
