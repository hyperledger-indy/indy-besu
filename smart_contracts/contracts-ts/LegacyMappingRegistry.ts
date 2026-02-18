/**
 * Copyright (c) 2024 DSR Corporation, Denver, Colorado.
 * https://www.dsr-corporation.com
 * SPDX-License-Identifier: Apache-2.0
 */

import { concat, Signature, toUtf8Bytes } from 'ethers'
import { Contract } from '../utils/contract'

export class LegacyMappingRegistry extends Contract {
  constructor(sender?: any) {
    super(LegacyMappingRegistry.name, sender)
  }

  public async createDidMapping(
    identity: string,
    legacyIdentifier: string,
    newDid: string,
    ed25519Key: Uint8Array,
    ed25519Signature: Uint8Array,
  ) {
    const tx = await this.instance.createDidMapping(identity, legacyIdentifier, newDid, ed25519Key, ed25519Signature)
    return tx.wait()
  }

  public async createDidMappingSigned(
    identity: string,
    legacyIdentifier: string,
    newDid: string,
    ed25519Key: Uint8Array,
    ed25519Signature: Uint8Array,
    signature: Signature,
  ) {
    const tx = await this.instance.createDidMappingSigned(
      identity,
      signature.v,
      signature.r,
      signature.s,
      legacyIdentifier,
      newDid,
      ed25519Key,
      ed25519Signature,
    )
    return tx.wait()
  }

  public async didMapping(id: string): Promise<string> {
    return this.instance.didMapping(id)
  }

  public async createResourceMapping(
    identity: string,
    legacyIssuerIdentifier: string,
    legacyIdentifier: string,
    newIdentifier: string,
  ) {
    const tx = await this.instance.createResourceMapping(
      identity,
      legacyIssuerIdentifier,
      legacyIdentifier,
      newIdentifier,
    )
    return tx.wait()
  }

  public async createResourceMappingSigned(
    identity: string,
    legacyIssuerIdentifier: string,
    legacyIdentifier: string,
    newIdentifier: string,
    signature: Signature,
  ) {
    const tx = await this.instance.createResourceMappingSigned(
      identity,
      signature.v,
      signature.r,
      signature.s,
      legacyIssuerIdentifier,
      legacyIdentifier,
      newIdentifier,
    )
    return tx.wait()
  }

  public async resourceMapping(id: string): Promise<string> {
    return this.instance.resourceMapping(id)
  }

  public async signDidMappingEndorsementData(
    legacyMappingRegistry: LegacyMappingRegistry,
    identity: string,
    privateKey: Uint8Array,
    identifier: string,
    issuerDid: string,
    ed25519Key: Uint8Array,
    ed25519Signature: Uint8Array,
  ) {
    return this.signEndorsementData(
      privateKey,
      concat([
        identity,
        toUtf8Bytes('createDidMapping'),
        toUtf8Bytes(identifier),
        toUtf8Bytes(issuerDid),
        ed25519Key,
        ed25519Signature,
      ]),
    )
  }

  public async signResourceMappingEndorsementData(
    legacyMappingRegistry: LegacyMappingRegistry,
    identity: string,
    privateKey: Uint8Array,
    legacyIssuerIdentifier: string,
    legacyIdentifier: string,
    newIdentifier: string,
  ) {
    return this.signEndorsementData(
      privateKey,
      concat([
        identity,
        toUtf8Bytes('createResourceMapping'),
        toUtf8Bytes(legacyIssuerIdentifier),
        toUtf8Bytes(legacyIdentifier),
        toUtf8Bytes(newIdentifier),
      ]),
    )
  }
}
