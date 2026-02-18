/**
 * Copyright (c) 2024 DSR Corporation, Denver, Colorado.
 * https://www.dsr-corporation.com
 * SPDX-License-Identifier: Apache-2.0
 */

import { loadFixture } from '@nomicfoundation/hardhat-network-helpers'
import { expect } from 'chai'
import { createBaseDidDocument } from '../../utils'
import { deployUniversalDidResolver, TestableUniversalDidResolver } from '../utils/contract-helpers'
import { DidErrors } from '../utils/errors'
import { TestAccounts } from '../utils/test-entities'

describe('UniversalDidResolver', function () {
  let did: string
  let identity: string
  let indybesuDidDocument: string

  let universalDidResolver: TestableUniversalDidResolver
  let testAccounts: TestAccounts

  async function deployUniversalDidResolverFixture() {
    const {
      universalDidResolver: universalDidReolverInit,
      indyDidRegistry,
      testAccounts: testAccountsInit,
    } = await deployUniversalDidResolver()

    identity = testAccountsInit.trustee.account.address
    did = `did:indybesu:testnet:${identity}`
    indybesuDidDocument = createBaseDidDocument(did)

    indyDidRegistry.connect(testAccountsInit.trustee.account)
    await indyDidRegistry.createDid(identity, indybesuDidDocument)

    return { universalDidReolverInit, testAccountsInit }
  }

  beforeEach(async function () {
    const { universalDidReolverInit, testAccountsInit } = await loadFixture(deployUniversalDidResolverFixture)

    universalDidResolver = universalDidReolverInit
    testAccounts = testAccountsInit

    universalDidResolver.connect(testAccounts.trustee.account)
  })

  describe('Resolve did:indybesu', function () {
    it('Should resolve DID document', async function () {
      const document = await universalDidResolver.resolveDocument(did)

      expect(document).to.be.deep.equal(indybesuDidDocument)
    })

    it('Should resolve DID metadata', async function () {
      const metadata = await universalDidResolver.resolveMetadata(did)

      expect(metadata).to.contain({
        owner: testAccounts.trustee.account.address,
        deactivated: false,
      })
    })
  })

  describe('Resolve did:ethr', function () {
    it('Should resolve DID metadata', async function () {
      const didEthr = `did:ethr:${testAccounts.trustee.account.address}`
      const metadata = await universalDidResolver.resolveMetadata(didEthr)

      expect(metadata).to.contain({
        owner: testAccounts.trustee.account.address,
        deactivated: false,
      })
    })

    it('Should fail if an incorrect DID method-specific-id is provided', async function () {
      const incorrectDid = 'did:ethr:ab$ddfgh354345'

      await expect(universalDidResolver.resolveMetadata(incorrectDid))
        .revertedWithCustomError(universalDidResolver.baseInstance, DidErrors.IncorrectDid)
        .withArgs(incorrectDid)
    })
  })
})
