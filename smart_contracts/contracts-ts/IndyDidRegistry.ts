/**
 * Copyright (c) 2024 DSR Corporation, Denver, Colorado.
 * https://www.dsr-corporation.com
 * SPDX-License-Identifier: Apache-2.0
 */

import { concat, getBytes, Signature, toUtf8Bytes, toUtf8String } from 'ethers'
import { DidMetadataStruct } from '../typechain-types/contracts/did/IndyDidRegistry'
import { Contract } from '../utils/contract'

export type DidRecord = {
  document: string
  metadata: DidMetadataStruct
}

export class IndyDidRegistry extends Contract {
  constructor(sender?: any) {
    super(IndyDidRegistry.name, sender)
  }

  public async createDid(identity: string, document: string) {
    const tx = await this.instance.createDid(identity, toUtf8Bytes(document))
    return tx.wait()
  }

  public async createDidSigned(identity: string, document: string, signature: Signature) {
    const tx = await this.instance.createDidSigned(
      identity,
      signature.v,
      signature.r,
      signature.s,
      toUtf8Bytes(document),
    )
    return tx.wait()
  }

  public async updateDid(identity: string, document: string) {
    const tx = await this.instance.updateDid(identity, toUtf8Bytes(document))
    return tx.wait()
  }

  public async updateDidSigned(identity: string, document: string, signature: Signature) {
    const tx = await this.instance.updateDidSigned(
      identity,
      signature.v,
      signature.r,
      signature.s,
      toUtf8Bytes(document),
    )
    return tx.wait()
  }

  public async deactivateDid(identity: string) {
    const tx = await this.instance.deactivateDid(identity)
    return tx.wait()
  }

  public async deactivateDidSigned(identity: string, signature: Signature) {
    const tx = await this.instance.deactivateDidSigned(identity, signature.v, signature.r, signature.s)
    return tx.wait()
  }

  public async resolveDid(identity: string): Promise<DidRecord> {
    const record = await this.instance.resolveDid(identity)
    return {
      document: toUtf8String(getBytes(record.document)),
      metadata: {
        owner: record.metadata.owner,
        sender: record.metadata.sender,
        created: record.metadata.created,
        updated: record.metadata.updated,
        versionId: record.metadata.versionId,
        deactivated: record.metadata.deactivated,
      },
    }
  }

  public signCreateDidEndorsementData(identity: string, privateKey: Uint8Array, document: string) {
    return this.signEndorsementData(privateKey, concat([identity, toUtf8Bytes('createDid'), toUtf8Bytes(document)]))
  }

  public signUpdateDidEndorsementData(identity: string, privateKey: Uint8Array, document: string) {
    return this.signEndorsementData(privateKey, concat([identity, toUtf8Bytes('updateDid'), toUtf8Bytes(document)]))
  }

  public signDeactivateDidEndorsementData(identity: string, privateKey: Uint8Array) {
    return this.signEndorsementData(privateKey, concat([identity, toUtf8Bytes('deactivateDid')]))
  }
}
