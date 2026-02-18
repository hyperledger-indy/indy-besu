// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use aries_askar::kms::{KeyAlg, LocalKey};
use indy_besu_vdr::{Address, BasicSigner};

pub struct IndyWallet {
    pub did: String,
    pub edkey: String,
    key: LocalKey,
}

impl IndyWallet {
    pub async fn new(seed: Option<&str>) -> IndyWallet {
        let key = match seed {
            Some(seed) => LocalKey::from_secret_bytes(KeyAlg::Ed25519, seed.as_bytes()).unwrap(),
            None => LocalKey::generate(KeyAlg::Ed25519, false).unwrap(),
        };

        let verkey_bytes = key.to_public_bytes().unwrap();
        let did = bs58::encode(&verkey_bytes[0..16]).into_string();
        let edkey = bs58::encode(verkey_bytes.as_ref()).into_string();

        IndyWallet { did, edkey, key }
    }

    pub async fn sign(&self, bytes: &[u8]) -> Vec<u8> {
        self.key.sign_message(bytes, None).unwrap()
    }
}

pub struct BesuWallet {
    pub account: Address,
    pub secpkey: String,
    pub signer: BasicSigner,
}

impl BesuWallet {
    pub fn new(private_key: Option<&str>) -> BesuWallet {
        let mut signer = BasicSigner::new().unwrap();
        let (account, public_key) = signer.create_key(private_key).unwrap();
        let secpkey = bs58::encode(public_key).into_string();
        BesuWallet {
            account,
            secpkey,
            signer,
        }
    }
}
