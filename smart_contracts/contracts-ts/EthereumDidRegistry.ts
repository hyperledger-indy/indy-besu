/**
 * Copyright (c) 2024 DSR Corporation, Denver, Colorado.
 * https://www.dsr-corporation.com
 * SPDX-License-Identifier: Apache-2.0
 */

import { encodeBytes32String, toUtf8Bytes } from 'ethers'
import { Contract } from '../utils/contract'

export class EthereumExtDidRegistry extends Contract {
  constructor(sender?: any) {
    super(EthereumExtDidRegistry.name, sender)
  }

  public async identityOwner(identity: string): Promise<string> {
    return await this.instance.identityOwner(identity)
  }

  public async changed(identity: string) {
    return await this.instance.changed(identity)
  }

  public async changeOwner(identity: string, newOwner: string) {
    const tx = await this.instance.changeOwner(identity, newOwner)
    return tx.wait()
  }

  public async validDelegate(identity: string, delegateType: string, delegate: string): Promise<boolean> {
    return await this.instance.validDelegate(identity, delegateType, delegate)
  }

  public async addDelegate(identity: string, delegateType: string, delegate: string, validity: number) {
    const tx = await this.instance.addDelegate(identity, delegateType, delegate, validity)
    return tx.wait()
  }

  public async revokeDelegate(identity: string, delegateType: string, delegate: string) {
    const tx = await this.instance.revokeDelegate(identity, delegateType, delegate)
    return tx.wait()
  }

  public async setAttribute(identity: string, name: string, value: string, validity: number) {
    const tx = await this.instance.setAttribute(identity, encodeBytes32String(name), toUtf8Bytes(value), validity)
    return tx.wait()
  }

  public async revokeAttribute(identity: string, name: string, value: string) {
    const tx = await this.instance.revokeAttribute(identity, encodeBytes32String(name), toUtf8Bytes(value))
    return tx.wait()
  }

  // Methods that require signatures
  public async changeOwnerSigned(identity: string, sigV: number, sigR: string, sigS: string, newOwner: string) {
    const tx = await this.instance.changeOwnerSigned(identity, sigV, sigR, sigS, newOwner)
    return tx.wait()
  }

  public async addDelegateSigned(
    identity: string,
    sigV: number,
    sigR: string,
    sigS: string,
    delegateType: string,
    delegate: string,
    validity: number,
  ) {
    const tx = await this.instance.addDelegateSigned(identity, sigV, sigR, sigS, delegateType, delegate, validity)
    return tx.wait()
  }

  public async revokeDelegateSigned(
    identity: string,
    sigV: number,
    sigR: string,
    sigS: string,
    delegateType: string,
    delegate: string,
  ) {
    const tx = await this.instance.revokeDelegateSigned(identity, sigV, sigR, sigS, delegateType, delegate)
    return tx.wait()
  }

  public async setAttributeSigned(
    identity: string,
    sigV: number,
    sigR: string,
    sigS: string,
    name: string,
    value: Uint8Array,
    validity: number,
  ) {
    const tx = await this.instance.setAttributeSigned(identity, sigV, sigR, sigS, name, value, validity)
    return tx.wait()
  }

  public async revokeAttributeSigned(
    identity: string,
    sigV: number,
    sigR: string,
    sigS: string,
    name: string,
    value: Uint8Array,
  ) {
    const tx = await this.instance.revokeAttributeSigned(identity, sigV, sigR, sigS, name, value)
    return tx.wait()
  }
}
