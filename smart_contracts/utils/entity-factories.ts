/**
 * Copyright (c) 2024 DSR Corporation, Denver, Colorado.
 * https://www.dsr-corporation.com
 * SPDX-License-Identifier: Apache-2.0
 */

export function createBaseDidDocument(did: string, key?: any) {
  const kid = `${did}#KEY-1`
  return JSON.stringify({
    '@context': ['https://www.w3.org/ns/did/v1'],
    id: did,
    controller: [did],
    verificationMethod: [
      key || {
        id: kid,
        type: 'Ed25519VerificationKey2018',
        controller: did,
        publicKeyMultibase: 'zAKJP3f7BD6W4iWEQ9jwndVTCBq8ua2Utt8EEjJ6Vxsf',
      },
    ],
    authentication: [kid],
    assertionMethod: [],
    capabilityInvocation: [],
    capabilityDelegation: [],
    keyAgreement: [],
    service: [],
    alsoKnownAs: [],
  })
}

interface CreateSchemaParams {
  issuerId: string
  name?: string
  version?: string
  attrNames?: string[]
}

export function createSchemaObject({
  issuerId,
  name = 'BasicIdentity',
  version = '1.0.0',
  attrNames = ['First Name', 'Last Name'],
}: CreateSchemaParams) {
  const id = `${issuerId}/anoncreds/v0/SCHEMA/${name}/${version}`
  return {
    id,
    schema: JSON.stringify({
      id,
      issuerId,
      name,
      version,
      attrNames,
    }),
  }
}

interface CreateCredentialDefinitionParams {
  issuerId: string
  schemaId: string
  credDefType?: string
  tag?: string
  value?: Record<string, string>
}

export function createCredentialDefinitionObject({
  issuerId,
  schemaId,
  credDefType = 'CL',
  tag = 'BasicIdentity',
  value = {
    n: '779...397',
    rctxt: '774...977',
    s: '750..893',
    z: '632...005',
  },
}: CreateCredentialDefinitionParams) {
  const id = `${issuerId}/anoncreds/v0/CLAIM_DEF/${schemaId}/${tag}`
  return {
    id,
    credDef: JSON.stringify({
      id,
      issuerId,
      schemaId,
      credDefType,
      tag,
      value,
    }),
  }
}

interface CreateRevocationRegistryDefinitionParams {
  issuerId: string
  credDefId: string
  revocDefType?: string
  tag?: string
  value?: Record<string, any>
}

export function createRevocationRegistryDefinitionObject({
  issuerId,
  credDefId,
  revocDefType = 'CL_ACCUM',
  tag = 'RevocationRegistry',
  value = {
    publicKeys: {
      accumKey: {
        z: '1 0BB...386',
      },
    },
    maxCredNum: 666,
    tailsLocation: 'https://my.revocations.tails/tailsfile.txt',
    tailsHash: '91zvq2cFmBZmHCcLqFyzv7bfehHH5rMhdAG5wTjqy2PE',
  },
}: CreateRevocationRegistryDefinitionParams) {
  const id = credDefId
    .split('/')
    .map((part, index) => (index === 0 ? issuerId : part))
    .join('/')
    .replace('/CLAIM_DEF/', '/REV_REG_DEF/')
    .concat(`/${tag}`)

  return {
    id,
    revRegDef: JSON.stringify({
      id,
      revocDefType,
      credDefId,
      issuerId,
      tag,
      value,
    }),
  }
}

export interface CreateRevocationEntryParams {
  revRegDefId: string
  currentAccumulator?: string
  issued?: number[]
  revoked?: number[]
}

export function createRevocationRegistryEntryObject({
  revRegDefId,
  currentAccumulator = '0x10',
  issued = [0, 1],
  revoked = [],
}: CreateRevocationEntryParams) {
  return {
    revRegEntry: JSON.stringify({
      revRegDefId,
      currentAccumulator,
      issued,
      revoked,
    }),
  }
}
