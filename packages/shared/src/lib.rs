#![no_std]

use soroban_sdk::{contracttype, Address, BytesN};

// ── Storage TTL Configuration ──────────────────────────────────────────────────
// These constants control how long contract data persists on the Soroban ledger.
// All contracts use these shared values to ensure consistent expiration behavior.

/// Minimum ledgers before TTL entries are considered at risk. Used to trigger
/// preemptive TTL extension before entries expire.
pub const TTL_THRESHOLD_LEDGERS: u32 = 50_000;

/// Target ledgers for extended TTL after triggering a preemptive extension.
pub const TTL_EXTEND_TO_LEDGERS: u32 = 500_000;

// ── Numeric Input Bounds ───────────────────────────────────────────────────────
// These constants define the valid ranges for all numeric inputs across contracts.
// Every public numeric input must stay within these documented bounds or face
// explicit rejection with a panic guard.

/// Minimum valid schema version. Schema version 0 is reserved and invalid.
/// All schema versions must be > 0 to be approved or queried.
pub const MIN_SCHEMA_VERSION: u32 = 1;

/// Proof expiration timestamp must be strictly greater than the current ledger
/// timestamp. This ensures proofs always have a future validity window.
/// The maximum value is constrained by u64::MAX, but operationally should be
/// reasonable relative to ledger time (e.g., not decades in the future).
pub const MIN_EXPIRATION_OFFSET_FROM_NOW: u64 = 1;

/// Minimum valid contract version. Contract versions start at 1 and only increase.
/// Version 0 is reserved as "uninitialized" state.
pub const MIN_CONTRACT_VERSION: u32 = 1;

/// Maximum valid config version before overflow risk. ConfigVersion is incremented
/// on each config mutation via bump_config_version(). While u32::MAX is technically
/// possible, this constant documents the intended operational limit.
/// If ConfigVersion reaches this value, further mutations will be blocked.
pub const MAX_CONFIG_VERSION: u32 = u32::MAX - 1;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IssuerStatus {
    Active,
    Suspended,
    Revoked,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProofStatus {
    Active,
    Revoked,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IssuerRecord {
    pub issuer_id_hash: BytesN<32>,
    pub issuer_address: Address,
    pub metadata_hash: BytesN<32>,
    pub status: IssuerStatus,
    pub created_at: u64,
    pub updated_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofRecord {
    pub proof_id_hash: BytesN<32>,
    pub commitment_hash: BytesN<32>,
    pub issuer_address: Address,
    pub status: ProofStatus,
    pub schema_version: u32,
    pub expires_at: u64,
    pub created_at: u64,
    pub revoked_at: u64,
}
