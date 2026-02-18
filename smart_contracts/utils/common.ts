/**
 * Copyright (c) 2024 DSR Corporation, Denver, Colorado.
 * https://www.dsr-corporation.com
 * SPDX-License-Identifier: Apache-2.0
 */

import { readFileSync } from 'fs'
import { resolve } from 'path'

export function randomString(len: number = 6) {
  return (Math.random() + 1).toString(36).substring(len)
}

export function delay(ms: number) {
  return new Promise((resolveFunc) => setTimeout(resolveFunc, ms))
}

export interface ContractConfig {
  specPath: string
  address: string
}

export interface ContractsConfigs {
  credDefRegistry: ContractConfig
  schemaRegistry: ContractConfig
  roleControl: ContractConfig
  validatorControl: ContractConfig
  accountControl: ContractConfig
  upgradeControl: ContractConfig
  legacyMappingRegistry: ContractConfig
  ethereumDidRegistry: ContractConfig
  indyDidRegistry: ContractConfig
  universalDidResolver: ContractConfig
  revocationRegistry: ContractConfig
}

export interface BesuConfig {
  chainId: number
  nodeAddress: string
  contracts: ContractsConfigs
}

export function readBesuConfig(): BesuConfig {
  const configPath = resolve('../network/config.json')
  const data = readFileSync(configPath, 'utf8')

  // TODO: validate the file data structure
  return JSON.parse(data) as BesuConfig
}
