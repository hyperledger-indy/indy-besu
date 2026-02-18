// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::error::{VdrError, VdrResult};

use log::warn;
use log_derive::{logfn, logfn_inputs};
use secp256k1::{All, Message, PublicKey, Secp256k1, SecretKey};
use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
};

use crate::types::{Address, SignatureData};
use std::str::FromStr;
use web3::signing::keccak256;

pub struct KeyPair {
    pub public_key: PublicKey,
    private_key: SecretKey,
}

pub struct BasicSigner {
    secp: Secp256k1<All>,
    keys: HashMap<String, KeyPair>,
}

impl BasicSigner {
    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub fn new() -> VdrResult<BasicSigner> {
        Ok(BasicSigner {
            secp: Secp256k1::new(),
            keys: HashMap::new(),
        })
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub fn create_key(&mut self, private_key: Option<&str>) -> VdrResult<(Address, Vec<u8>)> {
        let (account, key_pair) = self.create_account(private_key)?;
        let public_key_bytes = key_pair.public_key.serialize_uncompressed().to_vec();
        self.keys.insert(account.to_string(), key_pair);
        Ok((account, public_key_bytes))
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    fn key_for_account(&self, account: &str) -> VdrResult<&KeyPair> {
        self.keys.get(account).ok_or_else(|| {
            let vdr_error = VdrError::SignerMissingKey(account.to_string());

            warn!(
                "Error: {:?} during getting keys for account: {}",
                vdr_error, account
            );

            vdr_error
        })
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    fn account_from_key(&self, public_key: &PublicKey) -> String {
        let hash = keccak256(&public_key.serialize_uncompressed()[1..]);
        format!("0x{}", hex::encode(&hash[12..]))
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub fn create_account(&self, private_key: Option<&str>) -> VdrResult<(Address, KeyPair)> {
        let private_key = match private_key {
            Some(private_key) => SecretKey::from_str(private_key)?,
            None => SecretKey::new(&mut rand::thread_rng()),
        };
        let public_key = PublicKey::from_secret_key(&self.secp, &private_key);
        let address = Address::from(self.account_from_key(&public_key).as_str());
        let key_pair = KeyPair {
            public_key,
            private_key,
        };
        Ok((address, key_pair))
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub fn sign(&self, message: &[u8], account: &str) -> VdrResult<SignatureData> {
        let key = self.key_for_account(account)?;
        let message = Message::from_digest_slice(message)?;
        let (recovery_id, signature) = self
            .secp
            .sign_ecdsa_recoverable(&message, &key.private_key)
            .serialize_compact();
        Ok(SignatureData {
            recovery_id: recovery_id.to_i32() as u64,
            signature: signature.to_vec(),
        })
    }
}

impl Debug for BasicSigner {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, r#"BasicSigner {{ }}"#)
    }
}

impl Debug for KeyPair {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, r#"KeyPair {{ }}"#)
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use once_cell::sync::Lazy;

    pub static TRUSTEE_ACC: Lazy<Address> =
        Lazy::new(|| Address::from("0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5"));

    pub const TRUSTEE_PRIVATE_KEY: &str =
        "8bbbb1b345af56b560a5b20bd4b0ed1cd8cc9958a16262bc75118453cb546df7";

    pub fn basic_signer() -> BasicSigner {
        let mut signer = BasicSigner::new().unwrap();
        signer.create_key(Some(TRUSTEE_PRIVATE_KEY)).unwrap();
        signer
    }

    #[allow(unused)]
    pub fn basic_signer_custom_key(private_key: &str) -> BasicSigner {
        let mut signer = BasicSigner::new().unwrap();
        signer.create_key(Some(private_key)).unwrap();
        signer
    }

    #[test]
    fn add_key_test() {
        let basic_signer = basic_signer();
        basic_signer.key_for_account(TRUSTEE_ACC.as_ref()).unwrap();
    }

    #[test]
    fn basic_signer_sign_test() {
        let basic_signer = basic_signer();
        let data = vec![
            43, 20, 70, 238, 250, 209, 10, 195, 87, 39, 219, 125, 26, 151, 3, 233, 70, 185, 237,
            52, 240, 127, 64, 8, 98, 136, 107, 144, 241, 122, 142, 64,
        ];
        let signature = basic_signer.sign(&data, TRUSTEE_ACC.as_ref()).unwrap();
        let expected = vec![
            200, 178, 128, 72, 163, 176, 188, 177, 119, 110, 11, 2, 194, 50, 220, 215, 0, 161, 247,
            77, 43, 80, 139, 173, 141, 122, 58, 206, 72, 28, 63, 59, 9, 59, 95, 160, 244, 66, 209,
            158, 150, 108, 172, 63, 216, 251, 53, 49, 237, 179, 250, 35, 127, 23, 178, 175, 72,
            164, 219, 64, 129, 109, 109, 93,
        ];
        assert_eq!(expected, signature.signature);
    }
}
