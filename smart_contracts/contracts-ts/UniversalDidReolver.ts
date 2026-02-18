/**
 * Copyright (c) 2024 DSR Corporation, Denver, Colorado.
 * https://www.dsr-corporation.com
 * SPDX-License-Identifier: Apache-2.0
 */

import { getBytes, toUtf8String } from 'ethers'
import { DidMetadataStruct } from '../typechain-types/contracts/did/IndyDidRegistry'
import { Contract } from '../utils'

export class UniversalDidResolver extends Contract {
  constructor(sender?: any) {
    super(UniversalDidResolver.name, sender)
  }

  public async resolveDocument(id: string): Promise<string> {
    return toUtf8String(getBytes(await this.instance.resolveDocument(id)))
  }

  public async resolveMetadata(id: string): Promise<DidMetadataStruct> {
    const metadata = await this.instance.resolveMetadata(id)
    return {
      owner: metadata.owner,
      sender: metadata.sender,
      created: metadata.created,
      updated: metadata.updated,
      versionId: metadata.versionId,
      deactivated: metadata.deactivated,
    }
  }
}
