/**
 * Copyright (c) 2024 DSR Corporation, Denver, Colorado.
 * https://www.dsr-corporation.com
 * SPDX-License-Identifier: Apache-2.0
 */

import { writeJson } from '../../utils'
import {
  accountControl,
  credentialDefinitionRegistry,
  ethereumDidRegistry,
  indyDidRegistry,
  legacyMappingRegistry,
  revocationRegistry,
  roleControl,
  schemaRegistry,
  universalDidResolver,
  upgradeControl,
  validatorControl,
} from './contracts'
import { prepareConfig } from './helpers'

function main() {
  const config = prepareConfig()

  const contracts = {
    ...accountControl(config.accountControl),
    ...roleControl(config.roleControl),
    ...validatorControl(config.validatorControl),
    ...upgradeControl(config.upgradeControl),
    ...indyDidRegistry(config.indyDidRegistry),
    ...ethereumDidRegistry(config.ethereumDidRegistry),
    ...universalDidResolver(config.universalDidResolver),
    ...schemaRegistry(config.schemaRegistry),
    ...credentialDefinitionRegistry(config.credentialDefinitionRegistry),
    ...legacyMappingRegistry(config.legacyMapping),
    ...revocationRegistry(config.revocationRegistry),
  }
  writeJson(contracts, 'ContractsGenesis.json')
}

if (require.main === module) {
  main()
}
