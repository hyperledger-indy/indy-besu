/**
 * Copyright (c) 2024 DSR Corporation, Denver, Colorado.
 * https://www.dsr-corporation.com
 * SPDX-License-Identifier: Apache-2.0
 */

import { Signer } from 'ethers'
import { ethers } from 'hardhat'
import { host, web3 } from '../environment'
import { createBaseDidDocument } from './entity-factories'

export interface AccountInfo {
  address: string
  privateKey: string
}

export class Account {
  public address: string
  public privateKey: string
  public signer: Signer

  constructor(data?: AccountInfo) {
    const provider = new ethers.JsonRpcProvider(host)
    if (data) {
      this.signer = new ethers.Wallet(data.privateKey, provider)
      this.address = data.address
      this.privateKey = data.privateKey
    } else {
      const account = web3.eth.accounts.create()
      this.signer = new ethers.Wallet(account.privateKey, provider)
      this.address = account.address
      this.privateKey = account.privateKey
    }
  }

  public get did() {
    return `did:indybesu:${this.address}`
  }

  public get didEthr() {
    return `did:ethr:${this.address}`
  }

  public get didDocument() {
    return createBaseDidDocument(this.did)
  }
}
