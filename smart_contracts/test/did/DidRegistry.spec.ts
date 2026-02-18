/**
 * Copyright (c) 2024 DSR Corporation, Denver, Colorado.
 * https://www.dsr-corporation.com
 * SPDX-License-Identifier: Apache-2.0
 */

import { expect } from 'chai'
import { ROLES } from '../../contracts-ts'
import { createBaseDidDocument } from '../../utils/entity-factories'
import {
  createDidSigned,
  deployIndyDidRegistry,
  TestableIndyDidRegistry,
  TestableRoleControl,
  testActorAddress,
  testActorPrivateKey,
} from '../utils/contract-helpers'
import { AuthErrors, DidErrors } from '../utils/errors'
import { TestAccounts } from '../utils/test-entities'

describe('IndyDidRegistry', function () {
  let didRegistry: TestableIndyDidRegistry
  let roleControl: TestableRoleControl
  let testAccounts: TestAccounts
  let identity: string
  let did: string
  let didDocument: string

  beforeEach(async function () {
    const {
      roleControl: roleControlInit,
      indyDidRegistry: didRegistryInit,
      testAccounts: testAccountsInit,
    } = await deployIndyDidRegistry()

    didRegistry = didRegistryInit
    roleControl = roleControlInit
    testAccounts = testAccountsInit

    identity = testAccounts.trustee.account.address
    did = `did:indybesu:testnet:${testAccounts.trustee.account}`
    didDocument = createBaseDidDocument(did)

    didRegistry.connect(testAccounts.trustee.account)
  })

  describe('Create DID', function () {
    it('Should create and resolve DID document', async function () {
      await didRegistry.createDid(identity, didDocument)

      const { document } = await didRegistry.resolveDid(identity)

      expect(document).to.be.deep.equal(didDocument)
    })

    it('Should fail if resolving DID does not exist', async function () {
      await expect(didRegistry.resolveDid(identity))
        .to.revertedWithCustomError(didRegistry.baseInstance, DidErrors.DidNotFound)
        .withArgs(identity)
    })

    it('Should fail if the DID being created already exists', async function () {
      await didRegistry.createDid(identity, didDocument)

      await expect(didRegistry.createDid(identity, didDocument))
        .to.be.revertedWithCustomError(didRegistry.baseInstance, DidErrors.DidAlreadyExist)
        .withArgs(identity)
    })

    it('Should fail if the DID being created by identity without required role', async function () {
      didRegistry.connect(testAccounts.noRole.account)

      await expect(
        didRegistry.createDid(testAccounts.noRole.account.address, didDocument),
      ).to.be.revertedWithCustomError(roleControl.baseInstance, AuthErrors.Unauthorized)
    })

    it('Should update and deactivate DID by identity owner', async function () {
      // create DID document by trustee3
      didRegistry.connect(testAccounts.trustee3.account)
      await didRegistry.createDid(testAccounts.trustee3.account.address, didDocument)

      // remove role from trustee3
      didRegistry.connect(testAccounts.trustee.account)
      await roleControl.revokeRole(ROLES.TRUSTEE, testAccounts.trustee3.account.address)

      // update DID document and deactivate DID by trustee3
      didRegistry.connect(testAccounts.trustee3.account)

      const authorDid = `did:indybesu:${testActorAddress}`
      const updatedDidDocument = createBaseDidDocument(authorDid, {
        id: 'kid',
        type: 'Ed25519VerificationKey2018',
        controller: authorDid,
        publicKeyMultibase: 'key',
      })
      await didRegistry.updateDid(testAccounts.trustee3.account.address, updatedDidDocument)

      let didRecord = await didRegistry.resolveDid(testAccounts.trustee3.account.address)
      expect(didRecord.document).to.be.deep.equal(updatedDidDocument)

      await didRegistry.deactivateDid(testAccounts.trustee3.account.address)
      didRecord = await didRegistry.resolveDid(testAccounts.trustee3.account.address)
      expect(didRecord.metadata.deactivated).to.be.deep.equal(true)
    })
  })

  describe('Update DID', function () {
    it('Should update DID document', async function () {
      await didRegistry.createDid(identity, didDocument)

      await didRegistry.updateDid(identity, didDocument)

      const { document } = await didRegistry.resolveDid(identity)

      expect(document).to.be.deep.equal(didDocument)
    })

    it('Should fail if the DID creator is not an update txn sender', async function () {
      await didRegistry.createDid(identity, didDocument)

      didRegistry.connect(testAccounts.trustee2.account)
      await expect(didRegistry.updateDid(identity, didDocument)).to.revertedWithCustomError(
        didRegistry.baseInstance,
        DidErrors.NotIdentityOwner,
      )
    })

    it('Should fail if the DID being updated does not exists', async function () {
      await expect(didRegistry.updateDid(identity, didDocument))
        .to.revertedWithCustomError(didRegistry.baseInstance, DidErrors.DidNotFound)
        .withArgs(identity)
    })

    it('Should fail if the DID being updated is deactivated', async function () {
      await didRegistry.createDid(identity, didDocument)
      await didRegistry.deactivateDid(identity)

      await expect(didRegistry.updateDid(identity, didDocument))
        .to.revertedWithCustomError(didRegistry.baseInstance, DidErrors.DidHasBeenDeactivated)
        .withArgs(identity)
    })
  })

  describe('Deactivate DID', function () {
    it('Should deactivate DID document', async function () {
      await didRegistry.createDid(identity, didDocument)
      await didRegistry.deactivateDid(identity)

      const didStorage = await didRegistry.resolveDid(identity)

      expect(didStorage.metadata.deactivated).is.true
    })

    it('Should fail if the DID has already been deactivated', async function () {
      await didRegistry.createDid(identity, didDocument)
      await didRegistry.deactivateDid(identity)

      await expect(didRegistry.deactivateDid(identity))
        .to.revertedWithCustomError(didRegistry.baseInstance, DidErrors.DidHasBeenDeactivated)
        .withArgs(identity)
    })

    it('Should fail if the DID being deactivated does not exists', async function () {
      await expect(didRegistry.deactivateDid(identity))
        .to.revertedWithCustomError(didRegistry.baseInstance, DidErrors.DidNotFound)
        .withArgs(identity)
    })

    it('Should fail if the DID creator is not an deactivate txn sender', async function () {
      await didRegistry.createDid(identity, didDocument)

      didRegistry.connect(testAccounts.trustee2.account)
      await expect(didRegistry.deactivateDid(identity)).to.revertedWithCustomError(
        didRegistry.baseInstance,
        DidErrors.NotIdentityOwner,
      )
    })
  })

  describe('Endorse DID', function () {
    it('Should endorse and resolve DID document', async function () {
      const authorDid = `did:indybesu:testnet:${testActorAddress}`
      const authorDidDocument = createBaseDidDocument(authorDid)

      const sig = await didRegistry.signCreateDidEndorsementData(
        testActorAddress,
        testActorPrivateKey,
        authorDidDocument,
      )
      await didRegistry.createDidSigned(testActorAddress, authorDidDocument, sig)

      const didRecord = await didRegistry.resolveDid(testActorAddress)
      expect(didRecord.document).to.be.deep.equal(authorDidDocument)
    })

    it('Should endorse DID update', async function () {
      const authorDid = `did:indybesu:${testActorAddress}`
      await createDidSigned(didRegistry, testActorAddress, authorDid)

      const updatedDidDocument = createBaseDidDocument(authorDid, {
        id: 'kid',
        type: 'Ed25519VerificationKey2018',
        controller: authorDid,
        publicKeyMultibase: 'key',
      })

      const sig = await didRegistry.signUpdateDidEndorsementData(
        testActorAddress,
        testActorPrivateKey,
        updatedDidDocument,
      )
      await didRegistry.updateDidSigned(testActorAddress, updatedDidDocument, sig)

      const didRecord = await didRegistry.resolveDid(testActorAddress)
      expect(didRecord.document).to.be.deep.equal(updatedDidDocument)
    })
  })
})
