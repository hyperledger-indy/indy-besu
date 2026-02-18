// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

fn main() {
    uniffi::generate_scaffolding("src/indy_besu_vdr.udl").unwrap();
}
