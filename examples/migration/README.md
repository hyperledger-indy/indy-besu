Sample script demonstration migration path described at the [design document](../../docs/migration/migration.md).

> NOTE: Under development

### Flow

The scripts consist of the following steps:
1. Setup actors: Trustee, Issuer, Verifier, Holder
2. Prepare Issuer data (DID, Schema, Credential Definition) and publish it to [Indy-Node](https://github.com/hyperledger/indy-node.git)
3. Issuer issue a credential to Holder and verifier request proof
4. Issuer migrate data to Besu Ledger
5. Verify previously issued holder credential using Besu Ledger as the data registry

### Requirements

* [Genesis transactions](./indy-genesis.txn) of a running Indy ledger

### Run

```
cargo run
```