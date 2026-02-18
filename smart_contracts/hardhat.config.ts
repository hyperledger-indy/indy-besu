/**
 * Copyright (c) 2024 DSR Corporation, Denver, Colorado.
 * https://www.dsr-corporation.com
 * SPDX-License-Identifier: Apache-2.0
 */

import '@nomicfoundation/hardhat-toolbox'
import '@openzeppelin/hardhat-upgrades'
import 'hardhat-dependency-compiler'
import 'hardhat-storage-layout'
import { HardhatUserConfig } from 'hardhat/config'
import { host } from './environment'

const config: HardhatUserConfig = {
  solidity: {
    version: '0.8.20',
    settings: {
      optimizer: {
        enabled: true,
        runs: 200,
      },
      evmVersion: 'constantinople',
      viaIR: true,
    },
  },
  dependencyCompiler: {
    paths: ['@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol'],
  },
  networks: {
    besu: {
      url: host,
    },
  },
}

export default config
