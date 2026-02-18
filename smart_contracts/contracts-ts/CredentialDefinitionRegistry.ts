/**
 * Copyright (c) 2024 DSR Corporation, Denver, Colorado.
 * https://www.dsr-corporation.com
 * SPDX-License-Identifier: Apache-2.0
 */

import { concat, getBytes, keccak256, Signature, toUtf8Bytes, toUtf8String } from 'ethers'
import { CredentialDefinitionMetadataStruct } from '../typechain-types/contracts/anoncreds/CredentialDefinitionRegistry'
import { Contract } from '../utils/contract'

export type CredentialDefinitionRecord = {
  credDef: string
  metadata: CredentialDefinitionMetadataStruct
}

export class CredentialDefinitionRegistry extends Contract {
  constructor(sender?: any) {
    super(CredentialDefinitionRegistry.name, sender)
  }

  public async createCredentialDefinition(
    identity: string,
    id: string,
    issuerId: string,
    schemaId: string,
    credDef: string,
  ) {
    const tx = await this.instance.createCredentialDefinition(
      identity,
      keccak256(toUtf8Bytes(id)),
      issuerId,
      keccak256(toUtf8Bytes(schemaId)),
      toUtf8Bytes(credDef),
    )
    return tx.wait()
  }

  public async createCredentialDefinitionSigned(
    identity: string,
    id: string,
    issuerId: string,
    schemaId: string,
    credDef: string,
    signature: Signature,
  ) {
    const tx = await this.instance.createCredentialDefinitionSigned(
      identity,
      signature.v,
      signature.r,
      signature.s,
      keccak256(toUtf8Bytes(id)),
      issuerId,
      keccak256(toUtf8Bytes(schemaId)),
      toUtf8Bytes(credDef),
    )
    return tx.wait()
  }

  public async resolveCredentialDefinition(id: string): Promise<CredentialDefinitionRecord> {
    const record = await this.instance.resolveCredentialDefinition(keccak256(toUtf8Bytes(id)))
    return {
      credDef: toUtf8String(getBytes(record.credDef)),
      metadata: {
        created: record.metadata.created,
      },
    }
  }

  public signCreateCredDefEndorsementData(
    identity: string,
    privateKey: Uint8Array,
    id: string,
    issuerId: string,
    schemaId: string,
    credDef: string,
  ) {
    return this.signEndorsementData(
      privateKey,
      concat([
        identity,
        toUtf8Bytes('createCredentialDefinition'),
        getBytes(keccak256(toUtf8Bytes(id)), 'hex'),
        toUtf8Bytes(issuerId),
        getBytes(keccak256(toUtf8Bytes(schemaId)), 'hex'),
        toUtf8Bytes(credDef),
      ]),
    )
  }
}
