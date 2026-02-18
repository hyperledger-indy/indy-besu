/**
 * Copyright (c) 2024 DSR Corporation, Denver, Colorado.
 * https://www.dsr-corporation.com
 * SPDX-License-Identifier: Apache-2.0
 */

import { expect } from 'chai'
import { keccak256, toUtf8Bytes } from 'ethers'
import { IndyDidRegistry } from '../../contracts-ts'
import { createCredentialDefinitionObject, createSchemaObject } from '../../utils'
import {
  createDid,
  createDidSigned,
  createSchema,
  createSchemaSigned,
  deployCredentialDefinitionRegistry,
  TestableCredentialDefinitionRegistry,
  TestableRoleControl,
  TestableSchemaRegistry,
  testActorAddress,
  testActorPrivateKey,
} from '../utils/contract-helpers'
import { AuthErrors, ClErrors, DidErrors } from '../utils/errors'
import { TestAccounts } from '../utils/test-entities'

describe('CredentialDefinitionRegistry', function () {
  let didRegistry: IndyDidRegistry
  let schemaRegistry: TestableSchemaRegistry
  let credentialDefinitionRegistry: TestableCredentialDefinitionRegistry
  let roleControl: TestableRoleControl
  let testAccounts: TestAccounts
  let schemaId: string
  let issuerAddress: string
  let issuerId: string

  beforeEach(async function () {
    const {
      indyDidRegistry: didRegistryInit,
      schemaRegistry: schemaRegistryInit,
      credentialDefinitionRegistry: credentialDefinitionRegistryInit,
      roleControl: roleControlInit,
      testAccounts: testAccountsInit,
    } = await deployCredentialDefinitionRegistry()

    didRegistryInit.connect(testAccountsInit.trustee.account)
    schemaRegistryInit.connect(testAccountsInit.trustee.account)
    credentialDefinitionRegistryInit.connect(testAccountsInit.trustee.account)

    issuerAddress = testAccountsInit.trustee.account.address
    issuerId = `did:indybesu:mainnet:${testAccountsInit.trustee.account.address}`
    await createDid(didRegistryInit, issuerAddress, issuerId)
    const { id } = await createSchema(schemaRegistryInit, issuerAddress, issuerId)

    didRegistry = didRegistryInit
    testAccounts = testAccountsInit
    schemaRegistry = schemaRegistryInit
    credentialDefinitionRegistry = credentialDefinitionRegistryInit
    roleControl = roleControlInit
    schemaId = id
  })

  describe('Add/Resolve Credential Definition', function () {
    it('Should create and resolve Credential Definition', async function () {
      const { id, credDef } = createCredentialDefinitionObject({ issuerId, schemaId })

      await credentialDefinitionRegistry.createCredentialDefinition(issuerAddress, id, issuerId, schemaId, credDef)
      const result = await credentialDefinitionRegistry.resolveCredentialDefinition(id)

      expect(result.credDef).to.be.deep.equal(credDef)
    })

    it('Should fail if resolving Credential Definition does not exist', async function () {
      const { id } = createCredentialDefinitionObject({ issuerId, schemaId })

      await expect(credentialDefinitionRegistry.resolveCredentialDefinition(id))
        .to.be.revertedWithCustomError(credentialDefinitionRegistry.baseInstance, ClErrors.CredentialDefinitionNotFound)
        .withArgs(keccak256(toUtf8Bytes(id)))
    })

    it('Should fail if Credential Definition is being already exists', async function () {
      const { id, credDef } = createCredentialDefinitionObject({ issuerId, schemaId })

      await credentialDefinitionRegistry.createCredentialDefinition(issuerAddress, id, issuerId, schemaId, credDef)

      await expect(
        credentialDefinitionRegistry.createCredentialDefinition(issuerAddress, id, issuerId, schemaId, credDef),
      )
        .to.be.revertedWithCustomError(
          credentialDefinitionRegistry.baseInstance,
          ClErrors.CredentialDefinitionAlreadyExist,
        )
        .withArgs(keccak256(toUtf8Bytes(id)))
    })

    it('Should fail if Credential Definition is being created with non-existing Issuer', async function () {
      const unknownIssuerId = `did:indybesu:mainnet:${testAccounts.noRole.account.address}`
      const { id, credDef } = createCredentialDefinitionObject({ issuerId: unknownIssuerId, schemaId })

      await expect(
        credentialDefinitionRegistry.createCredentialDefinition(issuerAddress, id, unknownIssuerId, schemaId, credDef),
      )
        .to.be.revertedWithCustomError(credentialDefinitionRegistry.baseInstance, ClErrors.IssuerNotFound)
        .withArgs(unknownIssuerId)
    })

    it('Should fail if Credential Definition is being created with inactive Issuer', async function () {
      await didRegistry.deactivateDid(issuerAddress)

      const { id, credDef } = createCredentialDefinitionObject({ issuerId, schemaId })

      await expect(
        credentialDefinitionRegistry.createCredentialDefinition(issuerAddress, id, issuerId, schemaId, credDef),
      )
        .to.be.revertedWithCustomError(credentialDefinitionRegistry.baseInstance, ClErrors.IssuerHasBeenDeactivated)
        .withArgs(issuerId)
    })

    it('Should fail if Credential Definition is being created with non-existing Schema', async function () {
      const unknownSchemaId = `${issuerId}/anoncreds/v0/SCHEMA/Test/1.0.0`
      const { id, credDef } = createCredentialDefinitionObject({ issuerId, schemaId: unknownSchemaId })

      await expect(
        credentialDefinitionRegistry.createCredentialDefinition(issuerAddress, id, issuerId, unknownSchemaId, credDef),
      )
        .to.be.revertedWithCustomError(schemaRegistry.baseInstance, ClErrors.SchemaNotFound)
        .withArgs(keccak256(toUtf8Bytes(unknownSchemaId)))
    })

    it('Should fail if Credential Definition is being created with not owned Issuer DID', async function () {
      const issuerId2 = `did:indybesu:mainnet:${testAccounts.trustee2.account}`
      const { id, credDef } = createCredentialDefinitionObject({ issuerId, schemaId })

      didRegistry.connect(testAccounts.trustee2.account)
      credentialDefinitionRegistry.connect(testAccounts.trustee2.account)

      await createDid(didRegistry, testAccounts.trustee2.account.address, issuerId2)
      await expect(
        credentialDefinitionRegistry.createCredentialDefinition(
          testAccounts.trustee2.account.address,
          id,
          issuerId,
          schemaId,
          credDef,
        ),
      ).to.be.revertedWithCustomError(credentialDefinitionRegistry.baseInstance, DidErrors.NotIdentityOwner)
    })

    it('Should fail if the Credential Definition being created by identity without required role', async function () {
      const { id, credDef } = createCredentialDefinitionObject({ issuerId, schemaId })

      credentialDefinitionRegistry.connect(testAccounts.noRole.account)

      await expect(
        credentialDefinitionRegistry.createCredentialDefinition(issuerAddress, id, issuerId, schemaId, credDef),
      ).to.be.revertedWithCustomError(roleControl.baseInstance, AuthErrors.Unauthorized)
    })
  })

  describe('Add/Resolve Credential Definition with did:ethr Issuer', function () {
    it('Should create and resolve Credential Definition', async function () {
      const ethrIssuerId = `did:ethr:${issuerAddress}`
      const { id, credDef } = createCredentialDefinitionObject({ issuerId: ethrIssuerId, schemaId })

      await credentialDefinitionRegistry.createCredentialDefinition(issuerAddress, id, ethrIssuerId, schemaId, credDef)
      const result = await credentialDefinitionRegistry.resolveCredentialDefinition(id)

      expect(result.credDef).to.be.deep.equal(credDef)
    })

    it('Should fail if Credential Definition is being created with not owned Issuer DID', async function () {
      const ethrIssuerId = `did:ethr:${testAccounts.trustee2.account.address}`
      const { id, credDef } = createCredentialDefinitionObject({ issuerId: ethrIssuerId, schemaId })

      await expect(
        credentialDefinitionRegistry.createCredentialDefinition(
          testAccounts.trustee2.account.address,
          id,
          ethrIssuerId,
          schemaId,
          credDef,
        ),
      ).to.be.revertedWithCustomError(credentialDefinitionRegistry.baseInstance, DidErrors.NotIdentityOwner)
    })

    it('Should fail if Credential Definition is being created with invalid Issuer ID', async function () {
      const invalidIssuerId = 'did:ethr:ab$ddfgh354345'
      const { id, credDef } = createCredentialDefinitionObject({ issuerId: invalidIssuerId, schemaId })

      await expect(
        credentialDefinitionRegistry.createCredentialDefinition(issuerAddress, id, invalidIssuerId, schemaId, credDef),
      )
        .to.be.revertedWithCustomError(schemaRegistry.baseInstance, ClErrors.InvalidIssuerId)
        .withArgs(invalidIssuerId)
    })
  })

  describe('Endorse/Resolve Credential Definition with did:ethr Issuer', function () {
    it('Should endorse and resolve Credential Definition with did:ethr', async function () {
      const authorDid = `did:ethr:${testActorAddress}`
      const { id: ethSchemaId } = await createSchemaSigned(schemaRegistry, testActorAddress, authorDid)

      const { id, credDef } = createCredentialDefinitionObject({ issuerId: authorDid, schemaId: ethSchemaId })
      const signature = await credentialDefinitionRegistry.signCreateCredDefEndorsementData(
        testActorAddress,
        testActorPrivateKey,
        id,
        authorDid,
        ethSchemaId,
        credDef,
      )

      await credentialDefinitionRegistry.createCredentialDefinitionSigned(
        testActorAddress,
        id,
        authorDid,
        ethSchemaId,
        credDef,
        signature,
      )

      const result = await credentialDefinitionRegistry.resolveCredentialDefinition(id)
      expect(result.credDef).to.be.deep.equal(credDef)
    })

    it('Should endorse and resolve Credential Definition with did:indybesu', async function () {
      const authorDid = `did:indybesu:${testActorAddress}`
      await createDidSigned(didRegistry, testActorAddress, authorDid)
      const { id: ethSchemaId } = await createSchemaSigned(schemaRegistry, testActorAddress, authorDid)

      const { id, credDef } = createCredentialDefinitionObject({ issuerId: authorDid, schemaId: ethSchemaId })
      const signature = await credentialDefinitionRegistry.signCreateCredDefEndorsementData(
        testActorAddress,
        testActorPrivateKey,
        id,
        authorDid,
        ethSchemaId,
        credDef,
      )

      await credentialDefinitionRegistry.createCredentialDefinitionSigned(
        testActorAddress,
        id,
        authorDid,
        ethSchemaId,
        credDef,
        signature,
      )

      const result = await credentialDefinitionRegistry.resolveCredentialDefinition(id)
      expect(result.credDef).to.be.deep.equal(credDef)
    })

    it('Should fail if Credential Definition is being endorsed with not owned Issuer DID', async function () {
      const authorDid = `did:ethr:${testActorAddress}`
      const { id: ethSchemaId } = await createSchemaSigned(schemaRegistry, testActorAddress, authorDid)

      const { id, credDef } = createCredentialDefinitionObject({ issuerId: authorDid, schemaId: ethSchemaId })

      const signature = await credentialDefinitionRegistry.signCreateCredDefEndorsementData(
        testAccounts.trustee2.account.address,
        testActorPrivateKey,
        id,
        authorDid,
        ethSchemaId,
        credDef,
      )
      await expect(
        credentialDefinitionRegistry.createCredentialDefinitionSigned(
          testAccounts.trustee2.account.address,
          id,
          authorDid,
          ethSchemaId,
          credDef,
          signature,
        ),
      ).to.be.revertedWithCustomError(schemaRegistry.baseInstance, DidErrors.NotIdentityOwner)
    })

    it('Should fail if Schema is being endorsed with invalid signature', async function () {
      const authorDid = `did:ethr:${testActorAddress}`
      const { id: ethSchemaId } = await createSchemaSigned(schemaRegistry, testActorAddress, authorDid)

      const { id, credDef } = createCredentialDefinitionObject({ issuerId: authorDid, schemaId: ethSchemaId })

      const signature = await credentialDefinitionRegistry.signCreateCredDefEndorsementData(
        testActorAddress,
        testActorPrivateKey,
        'different id passed into signature',
        authorDid,
        ethSchemaId,
        credDef,
      )
      await expect(
        credentialDefinitionRegistry.createCredentialDefinitionSigned(
          testActorAddress,
          id,
          authorDid,
          schemaId,
          credDef,
          signature,
        ),
      ).to.be.revertedWithCustomError(schemaRegistry.baseInstance, DidErrors.NotIdentityOwner)
    })
  })
})
