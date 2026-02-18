// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

import { UniversalDidResolverInterface } from "../did/UniversalDidResolverInterface.sol";
import { ControlledUpgradeable } from "../upgrade/ControlledUpgradeable.sol";

import { RevocationRegistryDefinitionRecord } from "./RevocationRegistryTypes.sol";
import { CredentialDefinitionRecord } from "./CredentialDefinitionTypes.sol";
import { RevocationRegistryInterface } from "./RevocationRegistryInterface.sol";
import { NotRevocationRegistryDefinitionIssuer, RevocationRegistryDefinitionAlreadyExist, RevocationRegistryDefinitionNotFound, AccumulatorMismatch } from "./AnoncredsErrors.sol";
import { CredentialDefinitionRegistryInterface } from "./CredentialDefinitionRegistryInterface.sol";
import { RoleControlInterface } from "../auth/RoleControl.sol";
import { AnoncredsRegistry } from "./AnoncredsRegistry.sol";
import { StringUtils } from "../utils/StringUtils.sol";

import { Errors } from "../utils/Errors.sol";

contract RevocationRegistry is RevocationRegistryInterface, ControlledUpgradeable, AnoncredsRegistry {
    /**
     * @dev Reference to the contract that manages AnonCreds Credential Definitions
     */
    CredentialDefinitionRegistryInterface private _credentialDefinitionRegistry;

    /**
     * Mapping Revocation Registry  Definition ID to its Revocation Registry Definition Details and Metadata.
     */
    mapping(bytes32 id => RevocationRegistryDefinitionRecord revocationRegistryDefinitionRecord) private _revRegDefs;

    /**
     * Mapping Revocation Registry Definition ID to the latest block number of a Revocation Registry Entry event
     */
    mapping(bytes32 id => uint blockIdMaps) private _lastEventBlockNumbers;

    /**
     * Checks that the Credential Definition exist
     */
    modifier _credentialDefinitionExists(bytes32 id) {
        _credentialDefinitionRegistry.resolveCredentialDefinition(id);
        _;
    }

    /**
     * Checks the uniqueness of the revocation registry definition ID
     */
    modifier _uniqueRevRegDefId(bytes32 id) {
        if (_revRegDefs[id].metadata.created != 0) revert RevocationRegistryDefinitionAlreadyExist(id);
        _;
    }

    /**
     * Checks that the revocation registry definition exist
     */
    modifier _revRecDefExist(bytes32 id) {
        if (_revRegDefs[id].metadata.created == 0) revert RevocationRegistryDefinitionNotFound(id);
        _;
    }

    /**
     * Checks that the provided issuerId is the original publisher of the Revocation Registry Definition
     */
    modifier _checkIssuer(string memory issuerId, bytes32 revRegDefId) {
        if (!StringUtils.equals(_revRegDefs[revRegDefId].metadata.issuerId, issuerId)) {
            revert NotRevocationRegistryDefinitionIssuer(issuerId);
        }
        _;
    }

    function initialize(
        address upgradeControlAddress,
        address didResolverAddress,
        address credentialDefinitionRegistry,
        address roleControlContractAddress
    ) public reinitializer(1) {
        _initializeUpgradeControl(upgradeControlAddress);
        _didResolver = UniversalDidResolverInterface(didResolverAddress);
        _credentialDefinitionRegistry = CredentialDefinitionRegistryInterface(credentialDefinitionRegistry);
        _roleControl = RoleControlInterface(roleControlContractAddress);
    }

    /// @inheritdoc RevocationRegistryInterface
    function createRevocationRegistryDefinition(
        address identity,
        bytes32 id,
        bytes32 credDefId,
        string calldata issuerId,
        bytes calldata revRegDef
    ) external override {
        _createRevocationRegistryDefinition(identity, msg.sender, id, credDefId, issuerId, revRegDef);
    }

    /// @inheritdoc RevocationRegistryInterface
    function createRevocationRegistryDefinitionSigned(
        address identity,
        uint8 sigV,
        bytes32 sigR,
        bytes32 sigS,
        bytes32 id,
        bytes32 credDefId,
        string calldata issuerId,
        bytes calldata revRegDef
    ) public virtual {
        bytes32 hash = keccak256(
            abi.encodePacked(
                bytes1(0x19),
                bytes1(0),
                address(this),
                identity,
                "createRevocationRegistryDefinition",
                id,
                credDefId,
                issuerId,
                revRegDef
            )
        );
        _createRevocationRegistryDefinition(
            identity,
            ecrecover(hash, sigV, sigR, sigS),
            id,
            credDefId,
            issuerId,
            revRegDef
        );
    }

    /// @inheritdoc RevocationRegistryInterface
    function resolveRevocationRegistryDefinition(
        bytes32 id
    )
        public
        view
        override
        _revRecDefExist(id)
        returns (RevocationRegistryDefinitionRecord memory revocationRegistryDefinitionRecord)
    {
        return _revRegDefs[id];
    }

    /// @inheritdoc RevocationRegistryInterface
    function getLastEventBlockNumber(bytes32 id) public view _revRecDefExist(id) returns (uint) {
        return _lastEventBlockNumbers[id];
    }

    /// @inheritdoc RevocationRegistryInterface
    function createRevocationRegistryEntry(
        address identity,
        bytes32 revRegDefId,
        string calldata issuerId,
        bytes calldata revRegEntry
    ) external override {
        _createRevocationRegistryEntry(identity, msg.sender, revRegDefId, issuerId, revRegEntry);
    }

    /// @inheritdoc RevocationRegistryInterface
    function createRevocationRegistryEntrySigned(
        address identity,
        uint8 sigV,
        bytes32 sigR,
        bytes32 sigS,
        bytes32 revRegDefId,
        string calldata issuerId,
        bytes calldata revRegEntry
    ) public override {
        bytes32 hash = keccak256(
            abi.encodePacked(
                bytes1(0x19),
                bytes1(0),
                address(this),
                identity,
                "createRevocationRegistryEntry",
                revRegDefId,
                issuerId,
                revRegEntry
            )
        );
        _createRevocationRegistryEntry(identity, ecrecover(hash, sigV, sigR, sigS), revRegDefId, issuerId, revRegEntry);
    }

    function _createRevocationRegistryDefinition(
        address identity,
        address actor,
        bytes32 id,
        bytes32 credDefId,
        string calldata issuerId,
        bytes calldata revRegDef
    )
        internal
        _senderIsTrusteeOrEndorserOrSteward
        _uniqueRevRegDefId(id)
        _validIssuer(issuerId, identity, actor)
        _credentialDefinitionExists(credDefId)
    {
        _revRegDefs[id].revRegDef = revRegDef;
        _revRegDefs[id].metadata.created = block.timestamp;
        _revRegDefs[id].metadata.issuerId = issuerId;
        _lastEventBlockNumbers[id] = 0;

        emit RevocationRegistryDefinitionCreated(id, identity);
    }

    function _createRevocationRegistryEntry(
        address identity,
        address actor,
        bytes32 revRegDefId,
        string calldata issuerId,
        bytes calldata revRegEntry
    )
        internal
        _senderIsTrusteeOrEndorserOrSteward
        _revRecDefExist(revRegDefId)
        _validIssuer(issuerId, identity, actor)
        _checkIssuer(issuerId, revRegDefId)
    {
        emit RevocationRegistryEntryCreated(
            revRegDefId,
            block.timestamp,
            _lastEventBlockNumbers[revRegDefId],
            revRegEntry
        );
        _lastEventBlockNumbers[revRegDefId] = block.number;
    }
}
