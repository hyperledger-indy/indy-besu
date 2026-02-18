// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
use rand::{distributions::Alphanumeric, Rng};

#[cfg(test)]
pub fn rand_string() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(12)
        .map(char::from)
        .collect()
}

#[cfg(test)]
pub fn init_env_logger() {
    let _ = env_logger::builder().is_test(true).try_init();
}
