/**
 * Copyright (c) 2024 DSR Corporation, Denver, Colorado.
 * https://www.dsr-corporation.com
 * SPDX-License-Identifier: Apache-2.0
 */

import { padLeft } from 'web3-utils'
import { ContractConfig } from '../contractConfig'
import { buildProxySection, slots } from '../helpers'

export interface UpgradeControlConfig extends ContractConfig {
  data: {
    roleControlContractAddress: string
  }
}

export function upgradeControl(config: UpgradeControlConfig) {
  const { name, address, description, data } = config
  const storage: any = {}

  // address of upgrade control contact stored in slot 0
  storage[slots['0']] = padLeft(data.roleControlContractAddress, 64)
  return buildProxySection(name, address, description, storage)
}
