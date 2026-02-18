/**
 * Copyright (c) 2024 DSR Corporation, Denver, Colorado.
 * https://www.dsr-corporation.com
 * SPDX-License-Identifier: Apache-2.0
 */

import { padLeft } from 'web3-utils'
import { ContractConfig } from '../contractConfig'
import { buildProxySection, slots } from '../helpers'

export interface CredentialDefinitionsConfig extends ContractConfig {
  data: {
    universalDidResolverAddress: string
    schemaRegistryAddress: string
    upgradeControlAddress: string
    roleControlContractAddress: string
  }
}

export function credentialDefinitionRegistry(config: CredentialDefinitionsConfig) {
  const { name, address, description, data } = config
  const storage: any = {}

  // address of upgrade control contact stored in slot 0
  storage[slots['0']] = padLeft(data.upgradeControlAddress, 64)
  // address of DID registry contact stored in slot 1
  storage[slots['1']] = padLeft(data.universalDidResolverAddress, 64)
  // address of Role control contact stored in slot 3
  storage[slots['2']] = padLeft(data.roleControlContractAddress, 64)
  // address of schema registry contact stored in slot 2
  storage[slots['3']] = padLeft(data.schemaRegistryAddress, 64)
  return buildProxySection(name, address, description, storage)
}
