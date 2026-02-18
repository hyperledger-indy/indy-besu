/**
 * Copyright (c) 2024 DSR Corporation, Denver, Colorado.
 * https://www.dsr-corporation.com
 * SPDX-License-Identifier: Apache-2.0
 */

import { padLeft } from 'web3-utils'
import { ContractConfig } from '../contractConfig'
import { buildProxySection, slots } from '../helpers'

export interface IndyDidRegistryConfig extends ContractConfig {
  data: {
    upgradeControlAddress: string
    roleControlContractAddress: string
  }
}

export function indyDidRegistry(config: IndyDidRegistryConfig) {
  const { name, address, description, data } = config
  const storage: any = {}

  storage[slots['0']] = padLeft(data.upgradeControlAddress, 64)
  // address of Role control contact stored in slot 1
  storage[slots['1']] = padLeft(data.roleControlContractAddress, 64)
  return buildProxySection(name, address, description, storage)
}
