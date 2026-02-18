/**
 * Copyright (c) 2024 DSR Corporation, Denver, Colorado.
 * https://www.dsr-corporation.com
 * SPDX-License-Identifier: Apache-2.0
 */

import { concat, getBytes, keccak256, Signature, toUtf8Bytes, toUtf8String } from 'ethers'
import { SchemaMetadataStruct } from '../typechain-types/contracts/anoncreds/SchemaRegistry'
import { Contract } from '../utils/contract'

export type SchemaRecord = {
  schema: string
  metadata: SchemaMetadataStruct
}

export class SchemaRegistry extends Contract {
  constructor(sender?: any) {
    super(SchemaRegistry.name, sender)
  }

  public async createSchema(identity: string, id: string, issuerId: string, schema: string) {
    const tx = await this.instance.createSchema(identity, keccak256(toUtf8Bytes(id)), issuerId, toUtf8Bytes(schema))
    return tx.wait()
  }

  public async createSchemaSigned(
    identity: string,
    id: string,
    issuerId: string,
    schema: string,
    signature: Signature,
  ) {
    const tx = await this.instance.createSchemaSigned(
      identity,
      signature.v,
      signature.r,
      signature.s,
      keccak256(toUtf8Bytes(id)),
      issuerId,
      toUtf8Bytes(schema),
    )
    return tx.wait()
  }

  public async resolveSchema(id: string): Promise<SchemaRecord> {
    const record = await this.instance.resolveSchema(keccak256(toUtf8Bytes(id)))
    return {
      schema: toUtf8String(getBytes(record.schema)),
      metadata: {
        created: record.metadata.created,
      },
    }
  }

  public async signCreateSchemaEndorsementData(
    identity: string,
    privateKey: Uint8Array,
    id: string,
    issuerId: string,
    schema: string,
  ) {
    return this.signEndorsementData(
      privateKey,
      concat([
        identity,
        toUtf8Bytes('createSchema'),
        getBytes(keccak256(toUtf8Bytes(id)), 'hex'),
        toUtf8Bytes(issuerId),
        toUtf8Bytes(schema),
      ]),
    )
  }
}
