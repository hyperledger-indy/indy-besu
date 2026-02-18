// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{
    client::{implementation::web3::client::Web3Client, Contract},
    error::{VdrError, VdrResult},
    types::ContractSpec,
    Address,
};
use std::fmt::{Debug, Formatter};

use ethabi::{AbiError, Event};
use log::warn;
use log_derive::{logfn, logfn_inputs};
use std::str::FromStr;

#[cfg(not(feature = "wasm"))]
use web3::{
    contract::Contract as Web3ContractImpl,
    ethabi::{Address as EthAddress, Function},
    transports::Http,
};
#[cfg(feature = "wasm")]
use web3_wasm::{
    contract::Contract as Web3ContractImpl,
    ethabi::{Address as EthAddress, Function},
    transports::Http,
};

pub struct Web3Contract {
    address: Address,
    contract: Web3ContractImpl<Http>,
}

impl Web3Contract {
    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    pub fn new(
        web3_client: &Web3Client,
        address: &str,
        contract_spec: &ContractSpec,
    ) -> VdrResult<Web3Contract> {
        let abi = serde_json::to_vec(&contract_spec.abi).map_err(|err| {
            let vdr_error = VdrError::CommonInvalidData(format!(
                "Unable to parse contract ABI from specification. Err: {:?}",
                err.to_string()
            ));

            warn!("Error: {:?} during creating new Web3Contract", vdr_error);

            vdr_error
        })?;
        let parsed_address = EthAddress::from_str(address).map_err(|err| {
            let vdr_error = VdrError::CommonInvalidData(format!(
                "Unable to parse contract address. Err: {:?}",
                err.to_string()
            ));

            warn!("Error: {:?} during creating new Web3Contract", vdr_error);

            vdr_error
        })?;
        let contract =
            Web3ContractImpl::from_json(web3_client.eth(), parsed_address, abi.as_slice())?;

        Ok(Web3Contract {
            contract,
            address: Address::from(address),
        })
    }
}

impl Contract for Web3Contract {
    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    fn address(&self) -> &Address {
        &self.address
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    fn function(&self, name: &str) -> VdrResult<&Function> {
        self.contract.abi().function(name).map_err(|err| {
            let vdr_error = VdrError::from(err);

            warn!(
                "Error: {:?} during getting smart contract function: {}",
                vdr_error, name
            );

            vdr_error
        })
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    fn event(&self, name: &str) -> VdrResult<&Event> {
        self.contract.abi().event(name).map_err(|err| {
            let vdr_error = VdrError::from(err);

            warn!(
                "Error: {:?} during getting smart contract event: {}",
                vdr_error, name
            );

            vdr_error
        })
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    fn errors(&self) -> Vec<&AbiError> {
        self.contract.abi().errors().collect()
    }
}

impl Debug for Web3Contract {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, r#"Web3Contract {{ address: {:?} }}"#, self.address)
    }
}

#[cfg(test)]
pub mod test {
    use crate::{
        client::client::test::{mock_client, INVALID_ADDRESS},
        validator_control::test::VALIDATOR_CONTROL_NAME,
    };

    use super::*;
    #[async_std::test]
    async fn function_method_does_not_exist() {
        let client = mock_client();
        let contract = client
            .contract(&VALIDATOR_CONTROL_NAME.to_string())
            .unwrap();

        let err = contract.function(INVALID_ADDRESS).unwrap_err();

        assert!(matches!(
            err,  | VdrError::ContractInvalidName { .. }
        ));
    }
}
