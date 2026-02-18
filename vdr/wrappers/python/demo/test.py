# Copyright (c) 2024 DSR Corporation, Denver, Colorado.
# https://www.dsr-corporation.com
# SPDX-License-Identifier: Apache-2.0

import asyncio
import json
import secrets
import string
import os
import time  # Add this import at the top of the file if not already present

from eth_keys import keys
from indy_besu_vdr import *

# Account address to use for sending transactions
trustee = {
    "address": '0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5',
    "secret": '8bbbb1b345af56b560a5b20bd4b0ed1cd8cc9958a16262bc75118453cb546df7'
}
identity = {
    "address": '0xce70ce892768d46caf120b600dec29ed20198982',
    "secret": '7eda33eb6a38a8e231ea5c3de90df24b8982e4de94ef0e3f870d8ca386a63132'
}
network = 'test'
project_root = f"{os.getcwd()}/../../.."


def sign(secret: str, data: bytes):
    signature = keys.PrivateKey(bytearray.fromhex(secret)).sign_msg_hash(data)
    rec_id = int(signature[-1:][0])
    sig = signature[0:-1]
    return SignatureData(recovery_id = rec_id, signature = sig)


def read_config():
    with open(f"{project_root}/network/config.json") as f:
        return json.loads(f.read())


async def demo():
    print("1. Init client")
    config = read_config()

    did_registry_contract = config["contracts"]["ethereumDidRegistry"]
    did_contract_address = did_registry_contract["address"]
    did_contract_spec_path = "{}/{}".format(project_root, did_registry_contract["specPath"])

    schema_registry_contract = config["contracts"]["schemaRegistry"]
    schema_contract_address = schema_registry_contract["address"]
    schema_contract_spec_path = "{}/{}".format(project_root, schema_registry_contract["specPath"])
    
    credential_definition_registry_contract = config["contracts"]["credDefRegistry"]
    credential_definition_contract_address = credential_definition_registry_contract["address"]
    credential_definition_contract_spec_path = "{}/{}".format(project_root, credential_definition_registry_contract["specPath"])

    revocation_registry_contract = config["contracts"]["revocationRegistry"]
    revocation_registry_contract_address = revocation_registry_contract["address"]
    revocation_registry_contract_spec_path = "{}/{}".format(project_root, revocation_registry_contract["specPath"])

    contract_configs = [
        ContractConfig(address = did_contract_address, spec_path = did_contract_spec_path, spec = None),
        ContractConfig(address = schema_contract_address, spec_path = schema_contract_spec_path, spec = None),
        ContractConfig(address = credential_definition_contract_address, spec_path = credential_definition_contract_spec_path, spec = None),
        ContractConfig(address = revocation_registry_contract_address, spec_path = revocation_registry_contract_spec_path, spec = None)
    ]
    client = LedgerClient(config["chainId"], config["nodeAddress"], contract_configs, network, None)
    status = await client.ping()
    print(' Status: ' + str(status))

    print("2. Publish DID")
    did = 'did:ethr:' + identity['address']
    service_attribute = {"serviceEndpoint": "http://10.0.0.2", "type": "LinkedDomains"}
    endorsing_data = await DidEthrRegistry.build_did_set_attribute_endorsing_data(client, did, service_attribute, 1000)
    identity_signature = sign(identity["secret"], endorsing_data.get_signing_bytes())
    endorsing_data.set_signature(identity_signature)
    transaction = await Endorsement.build_endorsement_transaction(client, trustee["address"], endorsing_data)
    trustee_signature = sign(trustee["secret"], transaction.get_signing_bytes())
    transaction.set_signature(trustee_signature)
    txn_hash = await client.submit_transaction(transaction)
    print(' Transaction hash: ' + bytes(txn_hash).hex())
    receipt = await client.get_receipt(txn_hash)
    print(' Transaction receipt: ' + receipt)

    print("3. Resolve DID Document")
    resolved_did_doc = await DidResolver.resolve_did(client, did, None)
    print(' Resolved DID Document:' + resolved_did_doc)

    print("4. Publish Schema")
    name = ''.join(secrets.choice(string.ascii_uppercase + string.digits) for _ in range(6))
    schema = Schema(did, name, "1.0.0", ["First Name", "Last Name"])
    endorsing_data = await SchemaRegistry.build_create_schema_endorsing_data(client, schema)
    identity_signature = sign(identity["secret"], endorsing_data.get_signing_bytes())
    endorsing_data.set_signature(identity_signature)

    # Author: serialize endorsement data into JSON string and pass it to trustee
    endorsing_data_json = endorsing_data.to_string()
    print('  Schema transaction endorsement data: ' + endorsing_data_json)
    # Trustee: deserialize endorsement data form json
    schema_endorsing_data = TransactionEndorsingData.from_string(endorsing_data_json)
    transaction = await Endorsement.build_endorsement_transaction(client, trustee["address"], schema_endorsing_data)
    trustee_signature = sign(trustee["secret"], transaction.get_signing_bytes())
    transaction.set_signature(trustee_signature)
    txn_hash = await client.submit_transaction(transaction)
    print(' Transaction hash: ' + bytes(txn_hash).hex())
    receipt = await client.get_receipt(txn_hash)
    print(' Transaction receipt: ' + receipt)

    print("5. Resolve Schema")
    resolved_schema = await SchemaRegistry.resolve_schema(client, schema.id)
    print(' Resolved Schema:' + resolved_schema.to_string())

    print("6. Publish Credential Definition")
    cred_def = CredentialDefinition(did, schema.id, "CL", "cred_def_tag", 
        "{"
        "    \"n\": \"779...397\","
        "    \"rctxt\": \"774...977\","
        "    \"s\": \"750..893\","
        "    \"z\":\"632...005\""
        "}"
    )    
    cred_def_endorsing_data = await CredentialDefinitionRegistry.build_create_credential_definition_endorsing_data(client, cred_def)
    identity_signature = sign(identity["secret"], cred_def_endorsing_data.get_signing_bytes())
    cred_def_endorsing_data.set_signature(identity_signature)
    transaction = await Endorsement.build_endorsement_transaction(client, trustee["address"], cred_def_endorsing_data)
    trustee_signature = sign(trustee["secret"], transaction.get_signing_bytes())
    transaction.set_signature(trustee_signature)
    txn_hash = await client.submit_transaction(transaction)
    print(' Transaction hash: ' + bytes(txn_hash).hex())
    receipt = await client.get_receipt(txn_hash)
    print(' Transaction receipt: ' + receipt)

    print("7. Resolve Credential Definition")
    resolved_cred_def = await CredentialDefinitionRegistry.resolve_credential_definition(client, cred_def.id)
    print(' Resolved Credential Definition:' + resolved_cred_def.to_string())

    print("8. Publish Revocation Registry Definition")
    revocation_registry = RevocationRegistryDefinition(issuer_id = did, cred_def_id = cred_def.id, revoc_def_type = "CL_ACCUM", tag = "rev_reg_def_tag", 
        value = "{"
        "    \"publicKeys\": {"
            "    \"accumKey\": {"
            "        \"z\": \"1 0BB...386\""
            "    }"
        "    },"
        "    \"maxCredNum\": 50,"
        "    \"tailsLocation\": \"https://my.revocations.tails/tailsfile.txt\","
        "    \"tailsHash\": \"91zvq2cFmBZmHCcLqFyzv7bfehHH5rMhdAG5wTjqy2PE\""
        "}"
    )
    revocation_registry_endorsing_data = await RevocationRegistry.build_create_revocation_registry_definition_endorsing_data(client, revocation_registry)
    identity_signature = sign(identity["secret"], revocation_registry_endorsing_data.get_signing_bytes())
    revocation_registry_endorsing_data.set_signature(identity_signature)
    transaction = await Endorsement.build_endorsement_transaction(client, trustee["address"], revocation_registry_endorsing_data)
    trustee_signature = sign(trustee["secret"], transaction.get_signing_bytes())
    transaction.set_signature(trustee_signature)
    txn_hash = await client.submit_transaction(transaction)
    print(' Transaction hash: ' + bytes(txn_hash).hex())
    receipt = await client.get_receipt(txn_hash)
    print(' Transaction receipt: ' + receipt)

    print("9. Resolve Revocation Registry Definition")
    resolved_revocation_registry = await RevocationRegistry.resolve_revocation_registry_definition(client, revocation_registry.id)
    print(' Resolved Revocation Registry Definition:' + resolved_revocation_registry.to_string())
    
    print("10. Publish Revocation Registry Entry")
    
    revocation_registry_entry = RevocationRegistryEntry(did, revocation_registry.id, 
        "{"
        "    \"currentAccumulator\": \"1 0BB...386\","
#        "    \"prevAccumulator\": \"0\","
        "    \"revoked\": ["
        "        1, 2, 3"
        "    ],"
        "   \"issued\": []"
        "}"
    )
    print(' Revocation Registry Entry: ' + revocation_registry_entry.to_string())
    revocation_registry_entry_endorsing_data = await RevocationRegistry.build_create_revocation_registry_entry_endorsing_data(client, revocation_registry_entry)
    identity_signature = sign(identity["secret"], revocation_registry_entry_endorsing_data.get_signing_bytes())
    revocation_registry_entry_endorsing_data.set_signature(identity_signature)
    transaction = await Endorsement.build_endorsement_transaction(client, trustee["address"], revocation_registry_entry_endorsing_data)
    trustee_signature = sign(trustee["secret"], transaction.get_signing_bytes())
    transaction.set_signature(trustee_signature)
    txn_hash = await client.submit_transaction(transaction)
    print(' Transaction hash: ' + bytes(txn_hash).hex())
    receipt = await client.get_receipt(txn_hash)
    print(' Transaction receipt: ' + receipt)
    past_timestamp = int(time.time())
    
    print("10.a Publish Revocation Registry Entry")
    revocation_registry_entry = RevocationRegistryEntry(did, revocation_registry.id, 
        "{"
        "    \"currentAccumulator\": \"1 0BB...386\","
        "    \"prevAccumulator\": \"1 0BB...386\","
        "    \"revoked\": ["
        "        11, 12, 13"
        "    ],"
        "   \"issued\": []"
        "}"
    )
    print(' Revocation Registry Entry: ' + revocation_registry_entry.to_string())
    revocation_registry_entry_endorsing_data = await RevocationRegistry.build_create_revocation_registry_entry_endorsing_data(client, revocation_registry_entry)
    identity_signature = sign(identity["secret"], revocation_registry_entry_endorsing_data.get_signing_bytes())
    revocation_registry_entry_endorsing_data.set_signature(identity_signature)
    transaction = await Endorsement.build_endorsement_transaction(client, trustee["address"], revocation_registry_entry_endorsing_data)
    trustee_signature = sign(trustee["secret"], transaction.get_signing_bytes())
    transaction.set_signature(trustee_signature)
    txn_hash = await client.submit_transaction(transaction)
    print(' Transaction hash: ' + bytes(txn_hash).hex())
    receipt = await client.get_receipt(txn_hash)
    print(' Transaction receipt: ' + receipt)

    print("11. Retrieve Revocation Status List")
    current_timestamp = int(time.time())
    status_list = await RevocationRegistry.resolve_revocation_registry_status_list_full(client, revocation_registry.id, current_timestamp)
    print(' Revocation Status List:' + status_list.to_string())
    
    print("11.a Retrieve Revocation Status List in the past")
    status_list = await RevocationRegistry.resolve_revocation_registry_status_list_full(client, revocation_registry.id, past_timestamp)
    print(' Revocation Status List:' + status_list.to_string())

if __name__ == "__main__":
    asyncio.get_event_loop().run_until_complete(demo())
