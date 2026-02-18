import { concat, getBytes, keccak256, Signature, toBigInt, toUtf8Bytes, toUtf8String } from 'ethers'
import {
  RevocationRegistryDefinitionMetadataStruct,
  RevocationRegistryEntryCreatedEvent,
} from '../typechain-types/contracts/anoncreds/RevocationRegistry'
import { Contract } from '../utils/contract'

export type RevocationRegistryDefinitionRecord = {
  revRegDef: string
  metadata: RevocationRegistryDefinitionMetadataStruct
}

export class RevocationRegistry extends Contract {
  constructor(sender?: any) {
    super(RevocationRegistry.name, sender)
  }

  public async createRevocationRegistryDefinition(
    identity: string,
    id: string,
    credDefId: string,
    issuerId: string,
    revRegDef: string,
  ) {
    const tx = await this.instance.createRevocationRegistryDefinition(
      identity,
      keccak256(toUtf8Bytes(id)),
      keccak256(toUtf8Bytes(credDefId)),
      issuerId,
      toUtf8Bytes(revRegDef),
    )
    return tx.wait()
  }

  public async createRevocationRegistryDefinitionSigned(
    identity: string,
    id: string,
    credDefId: string,
    issuerId: string,
    revRegDef: string,
    signature: Signature,
  ) {
    const tx = await this.instance.createRevocationRegistryDefinitionSigned(
      identity,
      signature.v,
      signature.r,
      signature.s,
      keccak256(toUtf8Bytes(id)),
      keccak256(toUtf8Bytes(credDefId)),
      issuerId,
      toUtf8Bytes(revRegDef),
    )
    return tx.wait()
  }

  public async createRevocationRegistryEntry(
    identity: string,
    revRegId: string,
    issuerId: string,
    revRegEntry: string,
  ) {
    const tx = await this.instance.createRevocationRegistryEntry(
      identity,
      keccak256(toUtf8Bytes(revRegId)),
      issuerId,
      toUtf8Bytes(revRegEntry),
    )
    return tx.wait()
  }

  public async createRevocationRegistryEntrySigned(
    identity: string,
    revRegDefId: string,
    issuerId: string,
    revRegEntry: string,
    signature: Signature,
  ) {
    const tx = await this.instance.createRevocationRegistryEntrySigned(
      identity,
      signature.v,
      signature.r,
      signature.s,
      keccak256(toUtf8Bytes(revRegDefId)),
      issuerId,
      toUtf8Bytes(revRegEntry),
    )
    return tx.wait()
  }

  public async resolveRevocationRegistryDefinition(id: string): Promise<RevocationRegistryDefinitionRecord> {
    const record = await this.instance.resolveRevocationRegistryDefinition(keccak256(toUtf8Bytes(id)))
    return {
      revRegDef: toUtf8String(getBytes(record.revRegDef)),
      metadata: {
        created: record.metadata.created,
      },
    }
  }
  public async fetchAllRevocationEntries(id: string): Promise<string[]> {
    const latestBlock = await this.instance.getLastEventBlockNumber(keccak256(toUtf8Bytes(id)))
    if (toBigInt(latestBlock) > 0n) {
      const revRegEntries = await this.getLogsRecursively(id, Number(toBigInt(latestBlock)))
      return revRegEntries
    }
    return []
  }

  private async getLogsRecursively(id: string, blockNum: number): Promise<string[]> {
    let entries: string[] = []

    if (blockNum > 0) {
      const latestBlock = await (this.signer?.provider ?? ethers.provider).getBlockNumber()
      const fromToBlock = Math.min(blockNum, latestBlock)

      const eventLogs = await this.instance.queryFilter(
        this.instance.filters.RevocationRegistryEntryCreated(keccak256(toUtf8Bytes(id))),
        fromToBlock,
        fromToBlock,
      )

      entries = eventLogs.map((log: RevocationRegistryEntryCreatedEvent.Log) =>
        toUtf8String(getBytes((log.args as RevocationRegistryEntryCreatedEvent.InputTuple)[3])),
      )

      const parentBlocks: number[] = eventLogs.map((log: RevocationRegistryEntryCreatedEvent.Log) =>
        Number(toBigInt((log.args as RevocationRegistryEntryCreatedEvent.InputTuple)[2])),
      )

      for (const parentBlock of parentBlocks) {
        if (parentBlock > 0) {
          const parentEntries = await this.getLogsRecursively(id, parentBlock)
          entries = entries.concat(parentEntries)
        }
      }
    }

    return entries
  }

  public signCreateRevRegDefEndorsementData(
    identity: string,
    privateKey: Uint8Array,
    id: string,
    credDefId: string,
    issuerId: string,
    revRegDef: string,
  ) {
    return this.signEndorsementData(
      privateKey,
      concat([
        identity,
        toUtf8Bytes('createRevocationRegistryDefinition'),
        getBytes(keccak256(toUtf8Bytes(id)), 'hex'),
        getBytes(keccak256(toUtf8Bytes(credDefId)), 'hex'),
        toUtf8Bytes(issuerId),
        getBytes(toUtf8Bytes(revRegDef), 'hex'),
      ]),
    )
  }

  public signCreateRevRegEntryEndorsementData(
    identity: string,
    privateKey: Uint8Array,
    revRegDefId: string,
    issuerId: string,
    revRegEntry: string,
  ) {
    return this.signEndorsementData(
      privateKey,
      concat([
        identity,
        toUtf8Bytes('createRevocationRegistryEntry'),
        getBytes(keccak256(toUtf8Bytes(revRegDefId)), 'hex'),
        toUtf8Bytes(issuerId),
        getBytes(toUtf8Bytes(revRegEntry), 'hex'),
      ]),
    )
  }
}
