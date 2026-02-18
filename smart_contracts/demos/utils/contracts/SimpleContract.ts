/**
 * Copyright (c) 2024 DSR Corporation, Denver, Colorado.
 * https://www.dsr-corporation.com
 * SPDX-License-Identifier: Apache-2.0
 */

import { Contract } from '../../../utils'

export class SimpleContract extends Contract {
  constructor(sender?: any) {
    super(SimpleContract.name, sender)
  }

  public async message() {
    return this.instance.message()
  }

  public async update(message: string) {
    let tx = await this.instance.update(message)
    return tx.wait()
  }
}
