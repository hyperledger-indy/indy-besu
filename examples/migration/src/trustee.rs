// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{
    ledger::{BesuLedger, IndyLedger, Ledgers},
    wallet::{BesuWallet, IndyWallet},
};
use indy_besu_vdr::{Address, Role, DID};
use std::time::Duration;

pub struct Trustee {
    pub indy_wallet: IndyWallet,
    pub indy_ledger: IndyLedger,
    pub besu_wallet: BesuWallet,
    pub besu_ledger: BesuLedger,

    pub indy_did: String,
    pub besu_did: String,
    pub edkey: String,
    pub account: Address,
    pub secpkey: String,
    pub used_ledger: Ledgers,
}

impl Trustee {
    const ED25519_SEED: &'static str = "000000000000000000000000Trustee1";

    const SECP_PRIVATE_KEY: &'static str =
        "8bbbb1b345af56b560a5b20bd4b0ed1cd8cc9958a16262bc75118453cb546df7";

    pub async fn setup() -> Trustee {
        let indy_wallet = IndyWallet::new(Some(Self::ED25519_SEED)).await;
        let indy_ledger = IndyLedger::new();
        let besu_wallet = BesuWallet::new(Some(Self::SECP_PRIVATE_KEY));
        let besu_ledger = BesuLedger::new().await;

        let indy_did = indy_wallet.did.clone();
        let edkey = indy_wallet.edkey.clone();
        let account = besu_wallet.account.clone();
        let secpkey = besu_wallet.secpkey.clone();
        let besu_did = DID::build("ethr", None, account.as_ref());

        Trustee {
            indy_wallet,
            indy_ledger,
            besu_wallet,
            besu_ledger,
            indy_did,
            besu_did: besu_did.to_string(),
            edkey,
            account,
            secpkey,
            used_ledger: Ledgers::Indy,
        }
    }

    pub async fn publish_indy_did(&self, did: &str, verkey: &str) {
        self.indy_ledger
            .publish_nym(
                &self.indy_wallet,
                &self.indy_did,
                did,
                verkey,
                Some("ENDORSER"),
            )
            .await;
        std::thread::sleep(Duration::from_millis(500));
    }

    pub async fn assign_besu_role(&self, role: &Role, to: &Address) {
        self.besu_ledger
            .assign_role(&self.account, role, to, &self.besu_wallet)
            .await
    }

    pub fn use_indy_ledger(&mut self) {
        self.used_ledger = Ledgers::Indy
    }

    pub fn use_besu_ledger(&mut self) {
        self.used_ledger = Ledgers::Besu
    }
}
