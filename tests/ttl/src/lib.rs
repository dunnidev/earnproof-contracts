#![no_std]

//! Deterministic TTL expiration, restoration, and missing-state boundary tests.
//!
//! Every test in this crate drives the ledger sequence explicitly. No test
//! depends on wall-clock time, on the order in which other tests run, or on an
//! undocumented default. The ledger numbers used here are chosen so that each
//! assertion sits exactly on a documented boundary:
//!
//! * `TTL_EXTEND_TO_LEDGERS` ledgers after a write, the entry is on its final
//!   live ledger.
//! * One ledger later, the entry is archived and the host auto-restores it on
//!   the next access.
//! * `TTL_THRESHOLD_LEDGERS` ledgers of remaining life is the exact point at
//!   which an extension call starts having an effect.
//!
//! The operator-facing consequences of these boundaries are written up in
//! [`docs/storage-ttl.md`](../../../docs/storage-ttl.md).

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod fixture;

#[cfg(test)]
mod extension;

#[cfg(test)]
mod expiry;

#[cfg(test)]
mod restoration;

#[cfg(test)]
mod missing_state;
