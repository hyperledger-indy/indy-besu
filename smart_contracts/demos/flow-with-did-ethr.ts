/**
 * Copyright (c) 2024 DSR Corporation, Denver, Colorado.
 * https://www.dsr-corporation.com
 * SPDX-License-Identifier: Apache-2.0
 */

import environment from '../environment'
import { Actor } from './utils/actor'
import { ROLES } from '../contracts-ts'
import {
  createCredentialDefinitionObject,
  createRevocationRegistryDefinitionObject,
  createRevocationRegistryEntryObject,
  createSchemaObject,
} from '../utils'

async function demo() {
  let receipt: any

  const trustee = await new Actor(environment.accounts.account1).init()
  const faber = await new Actor().init()
  const alice = await new Actor().init()
  const unauthorized = await new Actor().init()

  console.log('1. Trustee assign ENDORSER role to Faber')
  receipt = await trustee.roleControl.assignRole(ROLES.ENDORSER, faber.address)
  console.log(`Role ${ROLES.ENDORSER} assigned to account ${faber.address}. Receipt: ${JSON.stringify(receipt)}`)

  console.log('2. Faber sets service attribute to DID document (Optional)')
  receipt = await faber.ethereumDIDRegistry.setAttribute(
    faber.address,
    'did/svc/did-communication',
    'https://example.com',
    86400,
  )
  console.log(`Attribute created for id ${faber.address}. Receipt: ${JSON.stringify(receipt)}`)

  console.log("3. Faber creates a Test Schema using the 'did:ethr' DID as the issuer")
  const { id: schemaId, schema } = createSchemaObject({ issuerId: faber.didEthr })
  receipt = await faber.schemaRegistry.createSchema(faber.address, schemaId, faber.didEthr, schema)
  console.log(`Schema created for id ${schemaId}. Receipt: ${JSON.stringify(receipt)}`)

  console.log('4. Faber resolves Test Schema to ensure its written')
  const resolvedSchema = await faber.schemaRegistry.resolveSchema(schemaId)
  console.log(`Schema resolved for ${schemaId}. Schema: ${resolvedSchema.schema}`)

  console.log("5. Faber create a Test Credential Definition using the 'did:ethr' DID as the issuer")
  const { id: credentialDefinitionId, credDef: credentialDefinition } = createCredentialDefinitionObject({
    issuerId: faber.didEthr,
    schemaId: schemaId,
  })
  receipt = await faber.credentialDefinitionRegistry.createCredentialDefinition(
    faber.address,
    credentialDefinitionId,
    faber.didEthr,
    schemaId,
    credentialDefinition,
  )
  console.log(`Credential Definition created for id ${credentialDefinitionId}. Receipt: ${JSON.stringify(receipt)}`)

  console.log('6. Faber resolves Test Credential Definition to ensure its written')
  const resolvedCredentialDefinition = await faber.credentialDefinitionRegistry.resolveCredentialDefinition(
    credentialDefinitionId,
  )
  console.log(
    `Credential Definition resolved for ${credentialDefinitionId}. Credential Definition: ${resolvedCredentialDefinition.credDef}`,
  )

  console.log('7. Alice resolves Test Schema')
  const testSchema = await alice.schemaRegistry.resolveSchema(schemaId)
  console.log(`Schema resolved for ${schemaId}. Schema: ${testSchema.schema}`)

  console.log('8. Alice resolves Test Credential Definition')
  const testCredentialDefinition = await alice.credentialDefinitionRegistry.resolveCredentialDefinition(
    credentialDefinitionId,
  )
  console.log(
    `Credential Definition resolved for ${credentialDefinitionId}. Credential Definition: ${testCredentialDefinition.credDef}`,
  )

  console.log("9. Faber create a Test Revocation Registry Definition using the 'did:ethr' DID as the issuer")
  const { id: revocationRegistryId, revRegDef: revocationRegistryDefinition } =
    createRevocationRegistryDefinitionObject({
      issuerId: faber.didEthr,
      credDefId: credentialDefinitionId,
    })

  receipt = await faber.revocationRegistry.createRevocationRegistryDefinition(
    faber.address,
    revocationRegistryId,
    credentialDefinitionId,
    faber.didEthr,
    revocationRegistryDefinition,
  )

  console.log(
    `Revocation Registry Definition created for id ${revocationRegistryId}. Receipt: ${JSON.stringify(receipt)}`,
  )

  console.log('10. Faber resolves Test Revocation Registry Definition to ensure its written')
  const resolvedRevocationRegistryDefinition = await faber.revocationRegistry.resolveRevocationRegistryDefinition(
    revocationRegistryId,
  )
  console.log(
    `Revocation Registry Definition resolved for ${revocationRegistryId}. Credential Definition: ${resolvedRevocationRegistryDefinition.revRegDef}`,
  )

  console.log("11. Faber create a Test Revocation Registry Entry using the 'did:ethr' DID as the issuer")
  const { revRegEntry: revocationRegistryEntryStruct } = createRevocationRegistryEntryObject({
    revRegDefId: revocationRegistryId,
    issued: [],
    revoked: [5, 6],
  })

  receipt = await faber.revocationRegistry.createRevocationRegistryEntry(
    faber.address,
    revocationRegistryId,
    faber.didEthr,
    revocationRegistryEntryStruct,
  )

  console.log(
    `Revocation Registry Entry created for Revocation Registry Definition id ${revocationRegistryId}. Receipt: ${JSON.stringify(
      receipt,
    )}`,
  )

  console.log("11-a. Faber create another Test Revocation Registry Entry using the 'did:ethr' DID as the issuer")
  const { revRegEntry: revocationRegistryEntryStruct1 } = createRevocationRegistryEntryObject({
    revRegDefId: revocationRegistryId,
    issued: [],
    revoked: [0, 1],
  })

  receipt = await faber.revocationRegistry.createRevocationRegistryEntry(
    faber.address,
    revocationRegistryId,
    faber.didEthr,
    revocationRegistryEntryStruct1,
  )

  console.log(
    `Revocation Registry Entry created for Revocation Registry Definition id ${revocationRegistryId}. Receipt: ${JSON.stringify(
      receipt,
    )}`,
  )

  console.log('12. Alice fetches Revocation Registry Entries associated with a Test Revocation Registry Definition')
  const revRegEntries = await alice.revocationRegistry.fetchAllRevocationEntries(revocationRegistryId)

  console.log(`All Revocation Registry Entries found for Revocation Registry Definition id ${revocationRegistryId}: `)
  console.log(revRegEntries)
}

if (require.main === module) {
  demo()
}

module.exports = exports = demo
