/**
 * Copyright (c) 2024 DSR Corporation, Denver, Colorado.
 * https://www.dsr-corporation.com
 * SPDX-License-Identifier: Apache-2.0
 */

import { expect } from 'chai'
import { keccak256, toUtf8Bytes } from 'ethers'
import { IndyDidRegistry, SchemaRegistry } from '../../contracts-ts'
import { createSchemaObject } from '../../utils'
import {
  createDid,
  createDidSigned,
  deploySchemaRegistry,
  TestableRoleControl,
  TestableSchemaRegistry,
  testActorAddress,
  testActorPrivateKey,
} from '../utils/contract-helpers'
import { AuthErrors, ClErrors, DidErrors } from '../utils/errors'
import { TestAccounts } from '../utils/test-entities'

describe('SchemaRegistry', function () {
  let didRegistry: IndyDidRegistry
  let schemaRegistry: TestableSchemaRegistry
  let roleControl: TestableRoleControl
  let testAccounts: TestAccounts
  let issuerAddress: string
  let issuerId: string

  beforeEach(async function () {
    const {
      indyDidRegistry: didRegistryInit,
      roleControl: roleControlInit,
      schemaRegistry: schemaRegistryInit,
      testAccounts: testAccountsInit,
    } = await deploySchemaRegistry()

    didRegistryInit.connect(testAccountsInit.trustee.account)
    schemaRegistryInit.connect(testAccountsInit.trustee.account)

    issuerAddress = testAccountsInit.trustee.account.address
    issuerId = `did:indybesu:mainnet:${testAccountsInit.trustee.account.address}`
    await createDid(didRegistryInit, issuerAddress, issuerId)

    didRegistry = didRegistryInit
    testAccounts = testAccountsInit
    schemaRegistry = schemaRegistryInit
    roleControl = roleControlInit
  })

  describe('Add/Resolve Schema', function () {
    it('Should create and resolve Schema', async function () {
      const { id, schema } = createSchemaObject({ issuerId })

      await schemaRegistry.createSchema(issuerAddress, id, issuerId, schema)
      const result = await schemaRegistry.resolveSchema(id)

      expect(result.schema).to.be.equal(schema)
    })

    it('Should fail if resolving a non-existing schema', async function () {
      const { id } = createSchemaObject({ issuerId })

      await expect(schemaRegistry.resolveSchema(id))
        .to.be.revertedWithCustomError(schemaRegistry.baseInstance, ClErrors.SchemaNotFound)
        .withArgs(keccak256(toUtf8Bytes(id)))
    })

    it('Should fail if Schema is being created already exists', async function () {
      const { id, schema } = createSchemaObject({ issuerId })

      await schemaRegistry.createSchema(issuerAddress, id, issuerId, schema)

      await expect(schemaRegistry.createSchema(issuerAddress, id, issuerId, schema))
        .to.be.revertedWithCustomError(schemaRegistry.baseInstance, ClErrors.SchemaAlreadyExist)
        .withArgs(keccak256(toUtf8Bytes(id)))
    })

    it('Should fail if Schema is being created with non-existing Issuer', async function () {
      const identity = testAccounts.noRole.account.address
      const unknownIssuerId = `did:indybesu:mainnet:${identity}`
      const { id, schema } = createSchemaObject({ issuerId: unknownIssuerId })

      await expect(schemaRegistry.createSchema(issuerAddress, id, unknownIssuerId, schema))
        .to.be.revertedWithCustomError(schemaRegistry.baseInstance, ClErrors.IssuerNotFound)
        .withArgs(unknownIssuerId)
    })

    it('Should fail if Schema is being created with inactive Issuer', async function () {
      await didRegistry.deactivateDid(issuerAddress)

      const { id, schema } = createSchemaObject({ issuerId })

      await expect(schemaRegistry.createSchema(issuerAddress, id, issuerId, schema))
        .to.be.revertedWithCustomError(schemaRegistry.baseInstance, ClErrors.IssuerHasBeenDeactivated)
        .withArgs(issuerId)
    })

    it('Should fail if Schema is being created with not owned Issuer DID', async function () {
      const issuerId2 = `did:indybesu:mainnet:${testAccounts.trustee2.account}`
      const { id, schema } = createSchemaObject({ issuerId })

      didRegistry.connect(testAccounts.trustee2.account)
      schemaRegistry.connect(testAccounts.trustee2.account)

      await createDid(didRegistry, testAccounts.trustee2.account.address, issuerId2)
      await expect(
        schemaRegistry.createSchema(testAccounts.trustee2.account.address, id, issuerId, schema),
      ).to.be.revertedWithCustomError(schemaRegistry.baseInstance, DidErrors.NotIdentityOwner)
    })

    it('Should fail if the Schema being created by identity without required role', async function () {
      const { id, schema } = createSchemaObject({ issuerId })

      schemaRegistry.connect(testAccounts.noRole.account)

      await expect(schemaRegistry.createSchema(issuerAddress, id, issuerId, schema)).to.be.revertedWithCustomError(
        roleControl.baseInstance,
        AuthErrors.Unauthorized,
      )
    })
  })

  describe('Add/Resolve Schema with did:ethr Issuer', function () {
    it('Should create and resolve Schema', async function () {
      const ethrIssuerId = `did:ethr:${testAccounts.trustee.account.address}`

      const { id, schema } = createSchemaObject({ issuerId: ethrIssuerId })

      await schemaRegistry.createSchema(issuerAddress, id, ethrIssuerId, schema)
      const result = await schemaRegistry.resolveSchema(id)

      expect(result.schema).to.be.deep.equal(schema)
    })

    it('Should fail if Schema is being created with not owned Issuer DID', async function () {
      const ethrIssuerId = `did:ethr:${testAccounts.trustee2.account.address}`

      const { id, schema } = createSchemaObject({ issuerId: ethrIssuerId })

      await expect(schemaRegistry.createSchema(issuerAddress, id, ethrIssuerId, schema)).to.be.revertedWithCustomError(
        schemaRegistry.baseInstance,
        DidErrors.NotIdentityOwner,
      )
    })

    it('Should fail if Schema is being created with invalid Issuer ID', async function () {
      const invalidIssuerId = 'did:ethr:ab$ddfgh354345'
      const { id, schema } = createSchemaObject({ issuerId: invalidIssuerId })

      await expect(
        schemaRegistry.createSchema(issuerAddress, id, invalidIssuerId, schema),
      ).to.be.revertedWithCustomError(schemaRegistry.baseInstance, ClErrors.InvalidIssuerId)
    })
  })

  describe('Endorse/Resolve Schema with did:ethr Issuer', function () {
    it('Should endorse Schema with did:ethr', async function () {
      const authorDid = `did:ethr:${testActorAddress}`
      const { id, schema } = createSchemaObject({ issuerId: authorDid })

      const sig = await schemaRegistry.signCreateSchemaEndorsementData(
        testActorAddress,
        testActorPrivateKey,
        id,
        authorDid,
        schema,
      )

      await schemaRegistry.createSchemaSigned(testActorAddress, id, authorDid, schema, sig)
      const result = await schemaRegistry.resolveSchema(id)

      expect(result.schema).to.be.equal(schema)
    })

    it('Should endorse Schema with did:indybesu', async function () {
      const authorDid = `did:indybesu:${testActorAddress}`
      await createDidSigned(didRegistry, testActorAddress, authorDid)
      const { id, schema } = createSchemaObject({ issuerId: authorDid })

      const sig = await schemaRegistry.signCreateSchemaEndorsementData(
        testActorAddress,
        testActorPrivateKey,
        id,
        authorDid,
        schema,
      )
      await schemaRegistry.createSchemaSigned(testActorAddress, id, authorDid, schema, sig)
      const result = await schemaRegistry.resolveSchema(id)

      expect(result.schema).to.be.equal(schema)
    })

    it('Should fail if Schema is being endorsed with not owned Issuer DID', async function () {
      const authorDid = `did:ethr:${testAccounts.trustee2.account.address}`
      const { id, schema } = createSchemaObject({ issuerId: authorDid })

      const sig = await schemaRegistry.signCreateSchemaEndorsementData(
        testAccounts.trustee2.account.address,
        testActorPrivateKey,
        id,
        authorDid,
        schema,
      )
      await expect(
        schemaRegistry.createSchemaSigned(testAccounts.trustee2.account.address, id, authorDid, schema, sig),
      ).to.be.revertedWithCustomError(schemaRegistry.baseInstance, DidErrors.NotIdentityOwner)
    })

    it('Should fail if Schema is being endorsed with invalid signature', async function () {
      const authorDid = `did:ethr:${testActorAddress}`
      const { id, schema } = createSchemaObject({ issuerId: authorDid })

      const sig = await schemaRegistry.signCreateSchemaEndorsementData(
        testActorAddress,
        testActorPrivateKey,
        'different id passed into signature',
        authorDid,
        schema,
      )
      await expect(
        schemaRegistry.createSchemaSigned(testActorAddress, id, authorDid, schema, sig),
      ).to.be.revertedWithCustomError(schemaRegistry.baseInstance, DidErrors.NotIdentityOwner)
    })
  })
})
