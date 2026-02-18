// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

import { RevocationRegistryDefinitionRecord } from "./RevocationRegistryTypes.sol";

interface RevocationRegistryInterface {
    /**
     * @dev Event that is sent when a Revocation Registry Definition is created
     *
     * @param revocationRegistryDefinitionId    Keccak hash of created revocation registry definition id
     * @param identity                          Issuer address
     */
    event RevocationRegistryDefinitionCreated(bytes32 revocationRegistryDefinitionId, address identity);

    /**
     * @dev Event that is sent when a Revocation Registry Entry (Delta) is created
     *
     * @param revocationRegistryDefinitionId    Keccak hash of created revocation registry definition id
     * @param timestamp                         Timestamp of the created Revocation Registry Entry
     * @param revRegEntry                       Revocation Registry Entry JSON as bytes
     */
    event RevocationRegistryEntryCreated(
        bytes32 indexed revocationRegistryDefinitionId,
        uint256 indexed timestamp,
        uint parentBlockNumber,
        bytes revRegEntry
    );

    /**
     * @dev Creates a new Revocation Registry Definition.
     *
     * Once the Revocation Registry Definition is created, this function emits a `RevocationRegistryDefinitionCreated` event
     * with the new Revocation Registry Definition's ID and issuer address.
     *
     * Restrictions:
     * - Only senders with either TRUSTEE or ENDORSER or STEWARD role are permitted to create new object;
     *
     * This function can revert with following errors:
     * - `RevocationRegistryDefinitionAlreadyExist`: Raised if Revocation Registry Definition with provided ID already exist.
     * - `CredentialDefinitionNotFound`: Raised if the associated Credential Definition doesn't exist.
     * - `IssuerNotFound`: Raised if the associated issuer doesn't exist.
     * - `InvalidIssuerId`: Raised if the provided issuer DID is invalid.
     * - `IssuerHasBeenDeactivated`: Raised if the associated issuer is not active.
     * - `NotIdentityOwner`: Raised when specified issuer DID is not owned by sender.
     *
     * @param identity  Account address of Revocation Registry Definition issuer.
     * @param id        Keccak hash of Revocation Registry Id to be created.
     * @param credDefId Keccak hash of Credential Definition Id.
     * @param issuerId  DID of Revocation Registry Definition issuer.
     * @param revRegDef AnonCreds Revocation Registry Definition JSON as bytes.
     */
    function createRevocationRegistryDefinition(
        address identity,
        bytes32 id,
        bytes32 credDefId,
        string calldata issuerId,
        bytes calldata revRegDef
    ) external;

    /**
     * @dev Endorse a new Revocation Registry Definition (off-chain author signature).
     *
     * Once the Revocation Registry Definition is created, this function emits a `RevocationRegistryDefinitionCreated` event
     * with the new Revocation Registry Definition's ID and issuer address.
     *
     * Restrictions:
     * - Only senders with either TRUSTEE or ENDORSER or STEWARD role are permitted to create new object;
     *
     * This function can revert with following errors:
     * - `RevocationRegistryDefinitionAlreadyExist`: Raised if Revocation Registry Definition with provided ID already exist.
     * - `CredentialDefinitionNotFound`: Raised if the associated Credential Definition doesn't exist.
     * - `IssuerNotFound`: Raised if the associated issuer doesn't exist.
     * - `InvalidIssuerId`: Raised if the provided issuer DID is invalid.
     * - `IssuerHasBeenDeactivated`: Raised if the associated issuer is not active.
     * - `NotIdentityOwner`: Raised when specified issuer DID is not owned by sender.
     *
     * @param identity  Account address of credential definition issuer.
     * @param sigR      Part of EcDSA signature.
     * @param sigV      Part of EcDSA signature.
     * @param sigS      Part of EcDSA signature.
     * @param id        Keccak hash of Credential Definition id to be created.
     * @param issuerId  DID of Revocation Registry Definition issuer.
     * @param credDefId Keccak hash of Credential Definition id.
     * @param revRegDef AnonCreds Revocation Registry Definition JSON as bytes.
     */
    function createRevocationRegistryDefinitionSigned(
        address identity,
        uint8 sigV,
        bytes32 sigR,
        bytes32 sigS,
        bytes32 id,
        bytes32 credDefId,
        string calldata issuerId,
        bytes calldata revRegDef
    ) external;

    /**
     * @dev Resolve the Revocation Registry Definition associated with the given ID.
     *
     * If no matching Revocation Registry Definition is found, the function revert with `RevocationRegistryDefinitionNotFound` error
     *
     * @param id Keccak hash of the Revocation Registry Definition to be resolved.
     *
     * @return revocationRegistryDefinitionRecord Returns the Revocation Registry Definition with metadata.
     */
    function resolveRevocationRegistryDefinition(
        bytes32 id
    ) external returns (RevocationRegistryDefinitionRecord memory revocationRegistryDefinitionRecord);

    /**
     * @dev get the last event block number (to get the event of the last revocation number)
     */
    function getLastEventBlockNumber(bytes32 id) external returns (uint);

    /**
     * @dev Creates a new Revocation Registry Entry (Delta).
     *
     * Once the Revocation Registry Entry is created, this function emits a `RevocationRegistryEntryCreated` event
     * with the Revocation Registry Definition's ID, timestamp and a struct containing the new accumulator and issued/revoked
     * credentials
     *
     * Restrictions:
     * - Only senders with either TRUSTEE or ENDORSER or STEWARD role are permitted to create new object;
     * - Only the issuer of the associated Revocation Registry Definition is permitted to create new object;
     *
     * This function can revert with following errors:
     * - `RevocationRegistryNotFound`: Raised if the associated Revocation Registry Definition doesn't exist.
     * - `IssuerNotFound`: Raised if the associated issuer doesn't exist.
     * - `InvalidIssuerId`: Raised if the provided issuer DID is invalid.
     * - `IssuerHasBeenDeactivated`: Raised if the associated issuer is not active.
     * - `NotIdentityOwner`: Raised when specified issuer DID is not owned by sender.
     * - `NotRevocationRegistryDefinitionIssuer`: Raised when trying to create object while not being issuer of associated Revocation Registry Definition.
     *
     * @param identity            Account address of Revocation Registry Definition issuer.
     * @param revRegDefId         Keccak hash of the associated Revocation Registry Id.
     * @param issuerId            DID of Revocation Registry Definition issuer.
     * @param revRegEntry         AnonCreds Revocation Registry Entry JSON as bytes.
     */
    function createRevocationRegistryEntry(
        address identity,
        bytes32 revRegDefId,
        string calldata issuerId,
        bytes calldata revRegEntry
    ) external;

    /**
     * @dev Endorse a new Revocation Registry Entry (off-chain author signature).
     *
     * Once the Revocation Registry Entry is created, this function emits a `RevocationRegistryEntryCreated` event
     * with the Revocation Registry Definition's ID, timestamp and a struct containing the new accumulator and issued/revoked
     * credentials
     *
     * Restrictions:
     * - Only senders with either TRUSTEE or ENDORSER or STEWARD role are permitted to create new object;
     * - Only the issuer of the associated Revocation Registry Definition is permitted to create new object;
     *
     * This function can revert with following errors:
     * - `RevocationRegistryNotFound`: Raised if the associated Revocation Registry Definition doesn't exist.
     * - `IssuerNotFound`: Raised if the associated issuer doesn't exist.
     * - `InvalidIssuerId`: Raised if the provided issuer DID is invalid.
     * - `IssuerHasBeenDeactivated`: Raised if the associated issuer is not active.
     * - `NotIdentityOwner`: Raised when specified issuer DID is not owned by sender.
     * - `NotRevocationRegistryDefinitionIssuer`: Raised when trying to create object while not being issuer of associated Revocation Registry Definition.
     *
     * @param identity  Account address of credential definition issuer.
     * @param sigR                Part of EcDSA signature.
     * @param sigV                Part of EcDSA signature.
     * @param sigS                Part of EcDSA signature.
     * @param revRegDefId         Keccak hash of the associated Revocation Registry Id.
     * @param issuerId            DID of Revocation Registry Definition issuer.
     * @param revRegEntry         AnonCreds Revocation Registry Entry JSON as bytes.
     */
    function createRevocationRegistryEntrySigned(
        address identity,
        uint8 sigV,
        bytes32 sigR,
        bytes32 sigS,
        bytes32 revRegDefId,
        string calldata issuerId,
        bytes calldata revRegEntry
    ) external;
}
