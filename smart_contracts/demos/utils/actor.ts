/**
 * Copyright (c) 2024 DSR Corporation, Denver, Colorado.
 * https://www.dsr-corporation.com
 * SPDX-License-Identifier: Apache-2.0
 */

import { readBesuConfig } from '../../utils'
import {
  RoleControl,
  IndyDidRegistry,
  SchemaRegistry,
  CredentialDefinitionRegistry,
  ValidatorControl,
  UpgradeControl,
  EthereumExtDidRegistry,
  RevocationRegistry,
} from '../../contracts-ts'
import { Account, AccountInfo } from '../../utils'

export class Actor {
  public account: Account
  public roleControl!: RoleControl
  public validatorControl!: ValidatorControl
  public didRegistry!: IndyDidRegistry
  public ethereumDIDRegistry!: EthereumExtDidRegistry
  public schemaRegistry!: SchemaRegistry
  public credentialDefinitionRegistry!: CredentialDefinitionRegistry
  public revocationRegistry!: RevocationRegistry
  public upgradeControl!: UpgradeControl

  constructor(accountInfo?: AccountInfo) {
    this.account = accountInfo ? new Account(accountInfo) : new Account()
  }

  public async init() {
    const besuConfig = readBesuConfig()
    const contracts = besuConfig.contracts

    this.roleControl = await new RoleControl(this.account).getInstance(contracts.roleControl.address)
    this.validatorControl = await new ValidatorControl(this.account).getInstance(contracts.validatorControl.address)
    this.didRegistry = await new IndyDidRegistry(this.account).getInstance(contracts.indyDidRegistry.address)
    this.ethereumDIDRegistry = await new EthereumExtDidRegistry(this.account).getInstance(
      contracts.ethereumDidRegistry.address,
    )
    this.schemaRegistry = await new SchemaRegistry(this.account).getInstance(contracts.schemaRegistry.address)
    this.credentialDefinitionRegistry = await new CredentialDefinitionRegistry(this.account).getInstance(
      contracts.credDefRegistry.address,
    )
    this.revocationRegistry = await new RevocationRegistry(this.account).getInstance(
      contracts.revocationRegistry.address,
    )
    this.upgradeControl = await new UpgradeControl(this.account).getInstance(contracts.upgradeControl.address)
    return this
  }

  public get address() {
    return this.account.address
  }

  public get did() {
    return this.account.did
  }

  public get didEthr() {
    return this.account.didEthr
  }

  public get didDocument() {
    return this.account.didDocument
  }
}
