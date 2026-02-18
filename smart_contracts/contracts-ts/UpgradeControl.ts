/**
 * Copyright (c) 2024 DSR Corporation, Denver, Colorado.
 * https://www.dsr-corporation.com
 * SPDX-License-Identifier: Apache-2.0
 */

import { Contract } from '../utils/contract'

export class UpgradeControl extends Contract {
  constructor(sender?: any) {
    super(UpgradeControl.name, sender)
  }

  public async propose(proxy: string, implementation: string) {
    const tx = await this.instance.propose(proxy, implementation)
    return tx.wait()
  }

  public async approve(proxy: string, implementation: string) {
    const tx = await this.instance.approve(proxy, implementation)
    return tx.wait()
  }

  public async ensureSufficientApprovals(proxy: string, implementation: string): Promise<boolean> {
    return await this.instance.ensureSufficientApprovals(proxy, implementation)
  }
}
