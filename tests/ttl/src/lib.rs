/// TTL Expiration and Restoration Boundary Tests
///
/// Comprehensive deterministic tests for TTL (Time-To-Live) boundaries across
/// protocol-config, issuer-registry, and proof-registry contracts.
///
/// Each test verifies behavior at exact TTL boundaries:
/// - pre_expiry: entry still valid (1 ledger before boundary)
/// - at_expiry: entry at boundary (inclusive validity)
/// - post_expiry: entry expired (1 ledger after boundary)
/// - restoration: entry restored after expiry
///
/// Soroban SDK 27.0.0 TTL Model:
/// - extend_ttl(threshold, extend_to): Extends TTL if current TTL <= threshold
/// - Expiry: entry is expired when ledger.sequence > expiry_ledger
/// - Boundary: inclusive (at expiry ledger = still valid)
mod harness;
mod issuer_registry_ttl;
mod proof_registry_ttl;
mod protocol_config_ttl;
