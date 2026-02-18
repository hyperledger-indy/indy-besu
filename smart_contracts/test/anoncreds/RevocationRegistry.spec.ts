/**
 * Copyright (c) 2024 DSR Corporation, Denver, Colorado.
 * https://www.dsr-corporation.com
 * SPDX-License-Identifier: Apache-2.0
 */

import { expect } from 'chai'
import { keccak256, toUtf8Bytes } from 'ethers'
import { IndyDidRegistry } from '../../contracts-ts'
import {
  CreateRevocationEntryParams,
  createRevocationRegistryDefinitionObject,
  createRevocationRegistryEntryObject,
} from '../../utils'
import {
  createCredentialDefinition,
  createCredentialDefinitionSigned,
  createRevocationRegistryDefinition,
  createRevocationRegistryDefinitionSigned,
  createSchema,
  createSchemaSigned,
  deployRevocationRegistry,
  TestableCredentialDefinitionRegistry,
  TestableRevocationRegistry,
  TestableRoleControl,
  TestableSchemaRegistry,
  testActorAddress,
  testActorPrivateKey,
} from '../utils/contract-helpers'
import { ClErrors, DidErrors } from '../utils/errors'
import { TestAccounts } from '../utils/test-entities'

describe('RevocationRegistry', function () {
  let didRegistry: IndyDidRegistry
  let schemaRegistry: TestableSchemaRegistry
  let credentialDefinitionRegistry: TestableCredentialDefinitionRegistry
  let revocationRegistry: TestableRevocationRegistry
  let roleControl: TestableRoleControl
  let testAccounts: TestAccounts
  let schemaId: string
  let credDefId: string
  let issuerAddress: string
  let issuerId: string
  let issuerIdSigned: string
  let schemaIdSigned: string
  let credDefIdSigned: string

  beforeEach(async function () {
    const {
      indyDidRegistry: didRegistryInit,
      schemaRegistry: schemaRegistryInit,
      credentialDefinitionRegistry: credentialDefinitionRegistryInit,
      roleControl: roleControlInit,
      testAccounts: testAccountsInit,
      revocationRegistry: revocationRegistryInit,
    } = await deployRevocationRegistry()

    didRegistryInit.connect(testAccountsInit.trustee.account)
    schemaRegistryInit.connect(testAccountsInit.trustee.account)
    credentialDefinitionRegistryInit.connect(testAccountsInit.trustee.account)
    revocationRegistryInit.connect(testAccountsInit.trustee.account)

    issuerAddress = testAccountsInit.trustee.account.address
    issuerId = `did:ethr:${testAccountsInit.trustee.account.address}`
    const { id: createdSchemaId } = await createSchema(schemaRegistryInit, issuerAddress, issuerId)

    schemaId = createdSchemaId

    const { id: createdCredDefId } = await createCredentialDefinition(
      credentialDefinitionRegistryInit,
      issuerAddress,
      issuerId,
      schemaId,
    )

    issuerIdSigned = `did:ethr:${testActorAddress}`

    const { id: createdSchemaIdSigned } = await createSchemaSigned(schemaRegistryInit, testActorAddress, issuerIdSigned)

    schemaIdSigned = createdSchemaIdSigned

    const { id: createdCredDefIdSigned } = await createCredentialDefinitionSigned(
      credentialDefinitionRegistryInit,
      testActorAddress,
      issuerIdSigned,
      schemaIdSigned,
    )

    didRegistry = didRegistryInit
    testAccounts = testAccountsInit
    schemaRegistry = schemaRegistryInit
    credentialDefinitionRegistry = credentialDefinitionRegistryInit
    revocationRegistry = revocationRegistryInit
    roleControl = roleControlInit
    credDefId = createdCredDefId
    credDefIdSigned = createdCredDefIdSigned
  })

  describe('Add/Resolve Revocation Registry Definition with did:ethr Issuer', function () {
    it('Should create and resolve Revocation Registry Definition', async function () {
      const ethrIssuerId = `did:ethr:${issuerAddress}`
      const { id, revRegDef } = createRevocationRegistryDefinitionObject({ issuerId: ethrIssuerId, credDefId })

      await revocationRegistry.createRevocationRegistryDefinition(issuerAddress, id, credDefId, ethrIssuerId, revRegDef)
      const result = await revocationRegistry.resolveRevocationRegistryDefinition(id)

      expect(result.revRegDef).to.be.deep.equal(revRegDef)
    })

    it('Should fail if Revocation Registry Definition is being created with not owned Issuer DID', async function () {
      const ethrIssuerId = `did:ethr:${testAccounts.trustee2.account.address}`
      const { id, revRegDef } = createRevocationRegistryDefinitionObject({ issuerId: ethrIssuerId, credDefId })

      await expect(
        revocationRegistry.createRevocationRegistryDefinition(
          testAccounts.trustee2.account.address,
          id,
          credDefId,
          ethrIssuerId,
          revRegDef,
        ),
      ).to.be.revertedWithCustomError(revocationRegistry.baseInstance, DidErrors.NotIdentityOwner)
    })

    it('Should fail if Revocation Registry Definition is being created with invalid Issuer ID', async function () {
      const invalidIssuerId = 'did:ethr:ab$ddfgh354345'
      const { id, revRegDef } = createRevocationRegistryDefinitionObject({ issuerId: invalidIssuerId, credDefId })

      await expect(
        revocationRegistry.createRevocationRegistryDefinition(issuerAddress, id, credDefId, invalidIssuerId, revRegDef),
      )
        .to.be.revertedWithCustomError(revocationRegistry.baseInstance, ClErrors.InvalidIssuerId)
        .withArgs(invalidIssuerId)
    })

    it('Should fail if RevRegDef is being created for nonexistent Credential Definition', async function () {
      const ethrIssuerId = `did:ethr:${issuerAddress}`
      const { id, revRegDef } = createRevocationRegistryDefinitionObject({ issuerId: ethrIssuerId, credDefId })

      const invalidCredDefId = 'invalid credDefId'

      await expect(
        revocationRegistry.createRevocationRegistryDefinition(
          issuerAddress,
          id,
          invalidCredDefId,
          ethrIssuerId,
          revRegDef,
        ),
      )
        .to.be.revertedWithCustomError(credentialDefinitionRegistry.baseInstance, ClErrors.CredentialDefinitionNotFound)
        .withArgs(keccak256(toUtf8Bytes(invalidCredDefId)))
    })
  })

  describe('Endorse/Resolve Revocation Registry Definition with did:ethr Issuer', function () {
    it('Should endorse and resolve Revocation Registry Definition with did:ethr', async function () {
      const { id, revRegDef } = createRevocationRegistryDefinitionObject({
        issuerId: issuerIdSigned,
        credDefId: credDefIdSigned,
      })

      const revRegSig = revocationRegistry.signCreateRevRegDefEndorsementData(
        testActorAddress,
        testActorPrivateKey,
        id,
        credDefIdSigned,
        issuerIdSigned,
        revRegDef,
      )

      await revocationRegistry.createRevocationRegistryDefinitionSigned(
        testActorAddress,
        id,
        credDefIdSigned,
        issuerIdSigned,
        revRegDef,
        revRegSig,
      )
      const result = await revocationRegistry.resolveRevocationRegistryDefinition(id)

      expect(result.revRegDef).to.be.deep.equal(revRegDef)
    })

    it('Should fail if Revocation Registry Definition is being endorsed with not owned Issuer DID', async function () {
      const { id, revRegDef } = createRevocationRegistryDefinitionObject({
        issuerId: issuerIdSigned,
        credDefId: credDefIdSigned,
      })

      const revRegSig = revocationRegistry.signCreateRevRegDefEndorsementData(
        testAccounts.trustee2.account.address,
        testActorPrivateKey,
        id,
        credDefIdSigned,
        issuerIdSigned,
        revRegDef,
      )

      await expect(
        revocationRegistry.createRevocationRegistryDefinitionSigned(
          testAccounts.trustee2.account.address,
          id,
          credDefIdSigned,
          issuerIdSigned,
          revRegDef,
          revRegSig,
        ),
      ).to.be.revertedWithCustomError(revocationRegistry.baseInstance, DidErrors.NotIdentityOwner)
    })

    it('Should fail if Revocation Registry Definition is being endorsed with invalid signature', async function () {
      const { id, revRegDef } = createRevocationRegistryDefinitionObject({
        issuerId: issuerIdSigned,
        credDefId: credDefIdSigned,
      })

      const revRegSig = revocationRegistry.signCreateRevRegDefEndorsementData(
        testAccounts.trustee2.account.address,
        testActorPrivateKey,
        'different id passed into signature',
        credDefIdSigned,
        issuerIdSigned,
        revRegDef,
      )

      await expect(
        revocationRegistry.createRevocationRegistryDefinitionSigned(
          testActorAddress,
          id,
          credDefIdSigned,
          issuerIdSigned,
          revRegDef,
          revRegSig,
        ),
      ).to.be.revertedWithCustomError(revocationRegistry.baseInstance, DidErrors.NotIdentityOwner)
    })
  })

  describe('Add/Resolve All Revocation Registry Entry with did:ethr Issuer', function () {
    it('Should successfully add Revocation Registry Entry', async function () {
      const ethrIssuerId = `did:ethr:${issuerAddress}`
      const { id, revRegDef } = await createRevocationRegistryDefinition(
        revocationRegistry,
        issuerAddress,
        ethrIssuerId,
        credDefId,
      )

      const { revRegEntry } = createRevocationRegistryEntryObject({
        revRegDefId: id,
        currentAccumulator: '0x20',
        issued: [2, 3],
        revoked: [0, 1],
      })

      await revocationRegistry.createRevocationRegistryEntry(issuerAddress, id, issuerId, revRegEntry)

      const result = await revocationRegistry.resolveRevocationRegistryDefinition(id)

      expect(result.revRegDef).to.be.deep.equal(revRegDef)

      const entries = await revocationRegistry.fetchAllRevocationEntries(id)
      expect(entries[0]).to.be.deep.equal(revRegEntry)
    })

    it('Should fail if attempting to create Revocation Registry Entry for nonexistent RevRegDef', async function () {
      const ethrIssuerId = `did:ethr:${issuerAddress}`
      const id = 'invalid revRegDefId'

      const { revRegEntry: revocationRegistryEntry } = createRevocationRegistryEntryObject({ revRegDefId: id })

      await expect(
        revocationRegistry.createRevocationRegistryEntry(issuerAddress, id, ethrIssuerId, revocationRegistryEntry),
      )
        .to.be.revertedWithCustomError(revocationRegistry.baseInstance, ClErrors.RevocationRegistryDefinitionNotFound)
        .withArgs(keccak256(toUtf8Bytes(id)))
    })
  })

  describe('Endorse/Resolve Revocation Registry Entry with did:ethr Issuer', function () {
    it('Should endorse and resolve Revocation Registry Entry with did:ethr', async function () {
      const { id } = await createRevocationRegistryDefinitionSigned(
        revocationRegistry,
        testActorAddress,
        issuerIdSigned,
        credDefIdSigned,
      )

      const revocationRegistryEntryParams: CreateRevocationEntryParams = {
        revRegDefId: id,
        currentAccumulator: '0x20',
        issued: [2, 3],
        revoked: [0, 1],
      }

      const { revRegEntry: revocationRegistryEntry } =
        createRevocationRegistryEntryObject(revocationRegistryEntryParams)

      const revRegEntrySignature = revocationRegistry.signCreateRevRegEntryEndorsementData(
        testActorAddress,
        testActorPrivateKey,
        id,
        issuerIdSigned,
        revocationRegistryEntry,
      )

      await revocationRegistry.createRevocationRegistryEntrySigned(
        testActorAddress,
        id,
        issuerIdSigned,
        revocationRegistryEntry,
        revRegEntrySignature,
      )

      const entries = await revocationRegistry.fetchAllRevocationEntries(id)
      expect(entries[0]).to.be.deep.equal(revocationRegistryEntry)
    })

    it('Should fail if RevRegEntry is being endorsed with not owned Issuer DID', async function () {
      const { id } = await createRevocationRegistryDefinitionSigned(
        revocationRegistry,
        testActorAddress,
        issuerIdSigned,
        credDefIdSigned,
      )

      const revocationRegistryEntryParams: CreateRevocationEntryParams = {
        revRegDefId: id,
        currentAccumulator: '0x20',
        issued: [2, 3],
        revoked: [0, 1],
      }

      const { revRegEntry: revocationRegistryEntry } =
        createRevocationRegistryEntryObject(revocationRegistryEntryParams)

      const revRegEntrySignature = revocationRegistry.signCreateRevRegEntryEndorsementData(
        testAccounts.trustee2.account.address,
        testActorPrivateKey,
        id,
        issuerIdSigned,
        revocationRegistryEntry,
      )

      await expect(
        revocationRegistry.createRevocationRegistryEntrySigned(
          testAccounts.trustee2.account.address,
          id,
          issuerIdSigned,
          revocationRegistryEntry,
          revRegEntrySignature,
        ),
      ).to.be.revertedWithCustomError(revocationRegistry.baseInstance, DidErrors.NotIdentityOwner)
    })

    it('Should fail if Revocation Registry Entry is being endorsed with invalid signature', async function () {
      const { id } = await createRevocationRegistryDefinitionSigned(
        revocationRegistry,
        testActorAddress,
        issuerIdSigned,
        credDefIdSigned,
      )

      const revocationRegistryEntryParams: CreateRevocationEntryParams = {
        revRegDefId: id,
        currentAccumulator: '0x20',
        issued: [2, 3],
        revoked: [0, 1],
      }

      const { revRegEntry: revocationRegistryEntry } =
        createRevocationRegistryEntryObject(revocationRegistryEntryParams)

      const revRegEntrySignature = revocationRegistry.signCreateRevRegEntryEndorsementData(
        testAccounts.trustee2.account.address,
        testActorPrivateKey,
        'invalid signature id',
        issuerIdSigned,
        revocationRegistryEntry,
      )

      await expect(
        revocationRegistry.createRevocationRegistryEntrySigned(
          testAccounts.trustee2.account.address,
          id,
          issuerIdSigned,
          revocationRegistryEntry,
          revRegEntrySignature,
        ),
      ).to.be.revertedWithCustomError(revocationRegistry.baseInstance, DidErrors.NotIdentityOwner)
    })
  })
})
