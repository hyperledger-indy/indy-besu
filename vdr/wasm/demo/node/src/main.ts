/**
 * Copyright (c) 2024 DSR Corporation, Denver, Colorado.
 * https://www.dsr-corporation.com
 * SPDX-License-Identifier: Apache-2.0
 */

import fs from "fs";
import secp256k1 from "secp256k1";

import { readFileSync } from "fs";
import { resolve } from 'path'
import { LedgerClient, EthrDidRegistry, DidResolver, SchemaRegistry, Endorsement, Schema, CredentialDefinition, CredentialDefinitionRegistry, RevocationRegistryDefinition, RevocationRegistry, RevocationRegistryEntry } from "indy-besu-vdr";

const projectRootPath = resolve('../../../..')
const trustee = {
    address: '0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5',
    secret: Uint8Array.from([139, 187, 177, 179, 69, 175, 86, 181, 96, 165, 178, 11, 212, 176, 237, 28, 216, 204, 153, 88, 161, 98, 98, 188, 117, 17, 132, 83, 203, 84, 109, 247])
}
const identity = {
    address: '0xce70ce892768d46caf120b600dec29ed20198982',
    secret: Uint8Array.from([126, 218, 51, 235, 106, 56, 168, 226, 49, 234, 92, 61, 233, 13, 242, 75, 137, 130, 228, 222, 148, 239, 14, 63, 135, 13, 140, 163, 134, 166, 49, 50])
}
const network = 'test'

function sign(message: Uint8Array, key: Uint8Array) {
    let signature = secp256k1.ecdsaSign(message, key)
    return {
        recovery_id: signature.recid,
        signature: signature.signature
    }
}

function readJson(path: string) {
    const data = readFileSync(path, 'utf8')
    return JSON.parse(data)
}

async function demo() {
    console.log('1. Init client')
    const configPath = `${projectRootPath}/network/config.json`
    const config = readJson(configPath)
    const contractConfigs = [
        {
            address: config.contracts.ethereumDidRegistry.address as string,
            spec: readJson(`${projectRootPath}/${config.contracts.ethereumDidRegistry.specPath}`)
        },
        {
            address: config.contracts.schemaRegistry.address as string,
            spec: readJson(`${projectRootPath}/${config.contracts.schemaRegistry.specPath}`)
        },
        {
            address: config.contracts.credDefRegistry.address as string,
            spec: readJson(`${projectRootPath}/${config.contracts.credDefRegistry.specPath}`)
        },
        {
            address: config.contracts.revocationRegistry.address as string,
            spec: readJson(`${projectRootPath}/${config.contracts.revocationRegistry.specPath}`)
        }
    ]


    const client = new LedgerClient(config.chainId, config.nodeAddress, contractConfigs, network, null)
    const status = await client.ping()
    console.log('Status: ' + JSON.stringify(status, null, 2))

    console.log('2. Publish and Modify DID')
    const did = 'did:ethr:' + network + ":" + identity.address
    const serviceAttribute = { "serviceEndpoint": "http://10.0.0.2", "type": "LinkedDomains" }
    const validity = BigInt(1000)
    let endorsingData = await EthrDidRegistry.buildDidSetAttributeEndorsingData(client, did, serviceAttribute, validity)
    let authorSignature = sign(endorsingData.getSigningBytes(), identity.secret)
    endorsingData.setSignature(authorSignature)
    let transaction = await Endorsement.buildEndorsementTransaction(client, trustee.address, endorsingData)
    let transactionSignature = sign(transaction.getSigningBytes(), trustee.secret)
    transaction.setSignature(transactionSignature)
    let txnHash = await client.submitTransaction(transaction)
    let receipt = await client.getReceipt(txnHash)
    console.log('Transaction receipt: ' + receipt)

    console.log('3. Resolve DID Document')
    const didWithMeta = await DidResolver.resolveDid(client, did, null)
    console.log('Resolved DID Document: ' + JSON.stringify(didWithMeta, null, 2))

    console.log('4. Publish Schema')
    const name = (Math.random() + 1).toString(36).substring(7)
    let schema = new Schema(did, name, "1.0.0", ["First Name", "Last Name"])
    let schemaEndorsingData = await SchemaRegistry.buildCreateSchemaEndorsingData(client, schema)
    authorSignature = sign(schemaEndorsingData.getSigningBytes(), identity.secret)
    schemaEndorsingData.setSignature(authorSignature)
    transaction = await Endorsement.buildEndorsementTransaction(client, trustee.address, schemaEndorsingData)
    transactionSignature = sign(transaction.getSigningBytes(), trustee.secret)
    transaction.setSignature(transactionSignature)
    txnHash = await client.submitTransaction(transaction)
    receipt = await client.getReceipt(txnHash)
    console.log('   Schema Transaction receipt: ' + receipt)

    console.log('5. Resolve Schema')
    const resolvedSchema = await SchemaRegistry.resolveSchema(client, schema.getId())
    console.log('   Resolved Schema: ' + resolvedSchema.toString())

    console.log('6. Publish Credential Definition')
    const tag = (Math.random() + 1).toString(36).substring(7)
    let credDef = new CredentialDefinition(did, schema.getId(), tag,
    {
            "n": "779...397",
            "rctxt": "774...977",
            "s": "750..893",
            "z":"632...005"
        }
    )
    let credDefEndorsingData = await CredentialDefinitionRegistry.buildCreateCredentialDefinitionEndorsingData(client, credDef)
    authorSignature = sign(credDefEndorsingData.getSigningBytes(), identity.secret)
    credDefEndorsingData.setSignature(authorSignature)
    transaction = await Endorsement.buildEndorsementTransaction(client, trustee.address, credDefEndorsingData)
    transactionSignature = sign(transaction.getSigningBytes(), trustee.secret)
    transaction.setSignature(transactionSignature)
    txnHash = await client.submitTransaction(transaction)
    receipt = await client.getReceipt(txnHash)
    console.log('   Credential Definition Transaction receipt: ' + receipt)
    
    console.log('7. Resolve Credential Definition')
    const resolvedCredDef = await CredentialDefinitionRegistry.resolveCredentialDefinition(client, credDef.getId())
    console.log('   Resolved Credential Definition: ' + resolvedCredDef.toString())
    
    console.log('8. Publish Revocation Registry Definition')
    const revRegDefTag = (Math.random() + 1).toString(36).substring(7)
    let revRegDef = new RevocationRegistryDefinition(did, credDef.getId(), revRegDefTag, {
        "publicKeys": {
            "accumKey": {
                "z": "1 0BB...386"
            }
        },
        "maxCredNum": 50,
        "tailsLocation": "https://my.revocations.tails/tailsfile.txt",
        "tailsHash": "91zvq2cFmBZmHCcLqFyzv7bfehHH5rMhdAG5wTjqy2PE"
    })
    let revRegDefEndorsingData = await RevocationRegistry.buildCreateRevocationRegistryDefinitionEndorsingData(client, revRegDef)
    authorSignature = sign(revRegDefEndorsingData.getSigningBytes(), identity.secret)
    revRegDefEndorsingData.setSignature(authorSignature)
    transaction = await Endorsement.buildEndorsementTransaction(client, trustee.address, revRegDefEndorsingData)
    transactionSignature = sign(transaction.getSigningBytes(), trustee.secret)
    transaction.setSignature(transactionSignature)
    txnHash = await client.submitTransaction(transaction)
    receipt = await client.getReceipt(txnHash)
    console.log('   Revocation Registry Definition Transaction receipt: ' + receipt)

    console.log('9. Resolve Revocation Registry Definition')
    const resolvedRevRegDef = await RevocationRegistry.resolveRevocationRegistryDefinition(client, revRegDef.getId())
    console.log('   Resolved Revocation Registry Definition: ' + resolvedRevRegDef.toString())

    console.log('10. Publish Revocation Registry Entry')
    let regRegEntry = new RevocationRegistryEntry(
        did,
        revRegDef.getId(), 
        "1 0BB...386",
        undefined,
        undefined,      
        Uint32Array.from([1, 2, 3]),
    )
    console.log('   Revocation Registry Entry: ' + regRegEntry.toString())
    let revRegEntryEndorsingData = await RevocationRegistry.buildCreateRevocationRegistryEntryEndorsingData(client, regRegEntry)
    authorSignature = sign(revRegEntryEndorsingData.getSigningBytes(), identity.secret)
    revRegEntryEndorsingData.setSignature(authorSignature)
    transaction = await Endorsement.buildEndorsementTransaction(client, trustee.address, revRegEntryEndorsingData)
    transactionSignature = sign(transaction.getSigningBytes(), trustee.secret)
    transaction.setSignature(transactionSignature)
    txnHash = await client.submitTransaction(transaction)
    receipt = await client.getReceipt(txnHash)
    const pastEpochTime = Math.floor(Date.now() / 1000);
    console.log('   Revocation Registry Entry Transaction receipt: ' + receipt)

    console.log('10.a Publish Revocation Registry Entry')
    let regRegEntry1 = new RevocationRegistryEntry(
        did,
        revRegDef.getId(), 
        "1 0BB...387",
        "1 0BB...386",
        Uint32Array.from([2]),      
        Uint32Array.from([11, 12, 13]),
    )
    let revRegEntryEndorsingData1 = await RevocationRegistry.buildCreateRevocationRegistryEntryEndorsingData(client, regRegEntry1)
    authorSignature = sign(revRegEntryEndorsingData1.getSigningBytes(), identity.secret)
    revRegEntryEndorsingData1.setSignature(authorSignature)
    transaction = await Endorsement.buildEndorsementTransaction(client, trustee.address, revRegEntryEndorsingData1)
    transactionSignature = sign(transaction.getSigningBytes(), trustee.secret)
    transaction.setSignature(transactionSignature)
    txnHash = await client.submitTransaction(transaction)
    receipt = await client.getReceipt(txnHash)
    console.log('   Revocation Registry Entry Transaction receipt: ' + receipt)

    console.log('11. Retrieve Revocation Status List')
    const currentEpochTime = Math.floor(Date.now() / 1000);
    let statusList = await RevocationRegistry.resolveRevocationRegistryStatusList(client, revRegDef.getId(), BigInt(currentEpochTime))
    console.log('   Revocation Status List: ' + statusList.toString())
    
    console.log('11.a Retrieve Revocation Status List in the past')
    let statusList1 = await RevocationRegistry.resolveRevocationRegistryStatusList(client, revRegDef.getId(), BigInt(pastEpochTime))
    console.log('   Revocation Status List: ' + statusList1.toString())

}

async function main() {
    await demo()
}

main()
