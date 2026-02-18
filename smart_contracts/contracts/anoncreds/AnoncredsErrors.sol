// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

// Issuer errors

/**
 * @notice Error that occurs when the provided issuer is not found.
 * @param did   Issuer DID.
 */
error IssuerNotFound(string did);

/**
 * @notice Error that occurs when the provided issuer ID is invalid.
 * @param did   Issuer DID.
 */
error InvalidIssuerId(string did);

/**
 * @notice Error that occurs when attempting to perform an operation on behalf of a deactivated issuer.
 * @param did   Issuer DID.
 */
error IssuerHasBeenDeactivated(string did);

// Schema errors

/**
 * @notice Error that occurs when trying to create a schema with already existing schema identifier.
 * @param id    Keccak hash of Schema ID.
 */
error SchemaAlreadyExist(bytes32 id);

/**
 * @notice Error that occurs when the specified schema is not found.
 * @param id    Keccak hash of Schema ID.
 */
error SchemaNotFound(bytes32 id);

// CredDef errors

/**
 * @notice Error that occurs when trying to create a credential definition with already existing identifier.
 * @param id     Keccak hash of Credential definition ID.
 */
error CredentialDefinitionAlreadyExist(bytes32 id);

/**
 * @notice Error that occurs when the specified credential definition is not found.
 * @param id    Keccak hash of Credential definition ID.
 */
error CredentialDefinitionNotFound(bytes32 id);

// RevReg errors

/**
 * @notice Error that occurs when the new previous Accumulator in Revocation Registry Entry does not match current Ledger Accumulator
 * @param accumulator    Current Ledger Accumulator.
 */
error AccumulatorMismatch(bytes accumulator);

/**
 * @notice Error that occurs when attempting to perform an operation on Revocation Registries while not being the Revocation Registry Definition Iusser.
 * @param did   Issuer DID.
 */
error NotRevocationRegistryDefinitionIssuer(string did);

/**
 * @notice Error that occurs when trying to create a revocation registry definition with already existing identifier.
 * @param id     Keccak hash of Credential definition ID.
 */
error RevocationRegistryDefinitionAlreadyExist(bytes32 id);

/**
 * @notice Error that occurs when the specified revocation registry definition is not found.
 * @param id    Keccak hash of Revocation Registry definition ID.
 */
error RevocationRegistryDefinitionNotFound(bytes32 id);
