/**
 * Copyright (c) 2024 DSR Corporation, Denver, Colorado.
 * https://www.dsr-corporation.com
 * SPDX-License-Identifier: Apache-2.0
 */

import { BytesLike } from 'ethers'
import { Contract } from '../../../utils'

export class UpgradablePrototype extends Contract {
  constructor(version: string, sender?: any) {
    super(version == '2.0' ? 'UpgradablePrototypeV2' : 'UpgradablePrototypeV1', sender)
  }

  public async upgradeToAndCall(newImplementation: string, data: BytesLike) {
    let tx = await this.instance.upgradeToAndCall(newImplementation, data)
    return tx.wait()
  }

  public get version(): Promise<string> {
    return this.instance.getVersion()
  }
}
