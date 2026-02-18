/**
 * Copyright (c) 2024 DSR Corporation, Denver, Colorado.
 * https://www.dsr-corporation.com
 * SPDX-License-Identifier: Apache-2.0
 */

import * as fs from 'fs-extra'
import path from 'path'
import { padLeft, sha3 } from 'web3-utils'
import { readBesuConfig } from '../../utils'

import {
  AccountControlConfig,
  CredentialDefinitionsConfig,
  EthereumDidRegistryConfig,
  IndyDidRegistryConfig,
  LegacyMappingRegistryConfig,
  RevocationRegistryConfig,
  RolesConfig,
  SchemasConfig,
  UniversalDidResolverConfig,
  ValidatorsConfig,
} from './contracts'
import { UpgradeControlConfig } from './contracts/upgradeControl'

// tslint:disable-next-line: no-var-requires
const linker = require('solc/linker')

export const slots = {
  '0': '0000000000000000000000000000000000000000000000000000000000000000',
  '1': '0000000000000000000000000000000000000000000000000000000000000001',
  '2': '0000000000000000000000000000000000000000000000000000000000000002',
  '3': '0000000000000000000000000000000000000000000000000000000000000003',
}

const proxyBytecode = readContractBytecode('ERC1967Proxy')
const proxyImplmentationSlot = '360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc'
const initializableVersionSlot = 'f0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00'

export function computeContractAddress(name: string) {
  const bytecode = readContractBytecode(name)

  const bytecodeHash = sha3(bytecode)

  sha3(bytecode)?.substring(44)
}

export function buildProxySection(
  name: string,
  address: string,
  comment: string,
  storage: Record<string, string>,
  libraries?: { [libraryName: string]: string },
) {
  let implementationBytecode = readContractBytecode(name)

  if (libraries) {
    implementationBytecode = linker.linkBytecode(implementationBytecode, libraries).split('\n')[0]
  }

  // set the version for the initializer to ensure that the initial version of the`initialize` method can not be called.
  storage[initializableVersionSlot] = `0x${padLeft('1', 64)}`

  // calculate and set contract implementation address
  const implementationAddress = sha3(implementationBytecode)!.substring(26)
  storage[proxyImplmentationSlot] = `0x${padLeft(implementationAddress, 64)}`

  return {
    [address]: {
      comment: `Proxy: ${comment}`,
      code: `0x${proxyBytecode}`,
      storage,
    },
    [`0x${implementationAddress}`]: {
      comment: `Implementation: ${comment}`,
      code: `0x${implementationBytecode}`,
    },
  }
}

export function buildSection(
  name: string,
  address: string,
  comment: string,
  storage: Record<string, string>,
  libraries?: { [libraryName: string]: string },
) {
  let bytecode = readContractBytecode(name)

  if (libraries) {
    bytecode = linker.linkBytecode(bytecode, libraries).split('\n')[0]
  }

  return {
    [address]: {
      comment,
      code: `0x${bytecode}`,
      storage,
    },
  }
}

export function readContractBytecode(contractName: string) {
  return fs.readFileSync(path.resolve(__dirname, '../../compiled-contracts', `${contractName}.bin-runtime`), 'utf8')
}

export interface ContractsConfigs {
  accountControl: AccountControlConfig
  credentialDefinitionRegistry: CredentialDefinitionsConfig
  indyDidRegistry: IndyDidRegistryConfig
  ethereumDidRegistry: EthereumDidRegistryConfig
  roleControl: RolesConfig
  schemaRegistry: SchemasConfig
  universalDidResolver: UniversalDidResolverConfig
  upgradeControl: UpgradeControlConfig
  validatorControl: ValidatorsConfig
  legacyMapping: LegacyMappingRegistryConfig
  revocationRegistry: RevocationRegistryConfig
}

export function prepareConfig(): ContractsConfigs {
  const contracts = readBesuConfig().contracts

  return {
    accountControl: {
      name: 'AccountControl',
      address: contracts.accountControl.address,
      description: 'Account permissioning smart contract',
      data: {
        roleControlContractAddress: contracts.roleControl.address,
        upgradeControlAddress: contracts.upgradeControl.address,
      },
    },
    credentialDefinitionRegistry: {
      name: 'CredentialDefinitionRegistry',
      address: contracts.credDefRegistry.address,
      description: 'Smart contract to manage credential definitions',
      data: {
        universalDidResolverAddress: contracts.universalDidResolver.address,
        schemaRegistryAddress: contracts.schemaRegistry.address,
        upgradeControlAddress: contracts.upgradeControl.address,
        roleControlContractAddress: contracts.roleControl.address,
      },
    },
    indyDidRegistry: {
      name: 'IndyDidRegistry',
      address: contracts.indyDidRegistry.address,
      description: 'Smart contract to manage DIDs',
      data: {
        upgradeControlAddress: contracts.upgradeControl.address,
        roleControlContractAddress: contracts.roleControl.address,
      },
    },
    universalDidResolver: {
      name: 'UniversalDidResolver',
      address: contracts.universalDidResolver.address,
      description: 'Smart contract to resolve DIDs from various DID registries',
      data: {
        indyDidRegistryAddress: contracts.indyDidRegistry.address,
        ethrDidRegistryAddress: contracts.ethereumDidRegistry.address,
        upgradeControlAddress: contracts.upgradeControl.address,
      },
    },
    ethereumDidRegistry: {
      name: 'EthereumExtDidRegistry',
      address: contracts.ethereumDidRegistry.address,
      description: 'Ethereum registry for ERC-1056 ethr did methods',
      data: {
        upgradeControlAddress: contracts.upgradeControl.address,
      },
    },
    roleControl: {
      name: 'RoleControl',
      address: contracts.roleControl.address,
      description: 'Smart contract to manage account roles',
      data: {
        accounts: [
          {
            account: '0xfe3b557e8fb62b89f4916b721be55ceb828dbd73',
            role: 1,
          },
          {
            account: '0x627306090abaB3A6e1400e9345bC60c78a8BEf57',
            role: 1,
          },
          {
            account: '0xf17f52151EbEF6C7334FAD080c5704D77216b732',
            role: 1,
          },
          {
            account: '0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5',
            role: 1,
          },
          {
            account: '0xca843569e3427144cead5e4d5999a3d0ccf92b8e',
            role: 1,
          },
        ],
        roleOwners: {
          '1': '1',
          '2': '1',
          '3': '1',
        },
        upgradeControlAddress: contracts.upgradeControl.address,
      },
    },
    schemaRegistry: {
      name: 'SchemaRegistry',
      address: contracts.schemaRegistry.address,
      description: 'Smart contract to manage schemas',
      data: {
        universalDidResolverAddress: contracts.universalDidResolver.address,
        upgradeControlAddress: contracts.upgradeControl.address,
        roleControlContractAddress: contracts.roleControl.address,
      },
    },
    upgradeControl: {
      name: 'UpgradeControl',
      address: contracts.upgradeControl.address,
      description: 'Smart contract to manage proxy contract upgrades',
      data: {
        roleControlContractAddress: contracts.roleControl.address,
      },
    },
    validatorControl: {
      name: 'ValidatorControl',
      address: contracts.validatorControl.address,
      description: 'Smart contract to manage validator nodes',
      data: {
        validators: [
          {
            account: '0xed9d02e382b34818e88b88a309c7fe71e65f419d',
            validator: '0x93917cadbace5dfce132b991732c6cda9bcc5b8a',
          },
          {
            account: '0xb30f304642de3fee4365ed5cd06ea2e69d3fd0ca',
            validator: '0x27a97c9aaf04f18f3014c32e036dd0ac76da5f18',
          },
          {
            account: '0x0886328869e4e1f401e1052a5f4aae8b45f42610',
            validator: '0xce412f988377e31f4d0ff12d74df73b51c42d0ca',
          },
          {
            account: '0xf48de4a0c2939e62891f3c6aca68982975477e45',
            validator: '0x98c1334496614aed49d2e81526d089f7264fed9c',
          },
        ],
        roleControlContractAddress: contracts.roleControl.address,
        upgradeControlAddress: contracts.upgradeControl.address,
      },
    },
    legacyMapping: {
      name: 'LegacyMappingRegistry',
      address: contracts.legacyMappingRegistry.address,
      description: 'Smart contract to store mapping of legacy identifiers to new one',
      data: {
        universalDidResolver: contracts.universalDidResolver.address,
        upgradeControlAddress: contracts.upgradeControl.address,
        roleControlContractAddress: contracts.roleControl.address,
      },
    },
    revocationRegistry: {
      name: 'RevocationRegistry',
      address: contracts.revocationRegistry.address,
      description: 'Smart contract to manage revocations',
      data: {
        universalDidResolverAddress: contracts.universalDidResolver.address,
        credentialDefinitionRegistryAddress: contracts.credDefRegistry.address,
        upgradeControlAddress: contracts.upgradeControl.address,
        roleControlContractAddress: contracts.roleControl.address,
      },
    },
  }
}
