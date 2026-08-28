# Testing

This document covers the test suites in this repository, and in detail the one that needs a procedure rather than just a command: the ledger snapshot regression fixtures.

## Running everything

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets
cargo test --workspace
cargo build --workspace
python3 scripts/check-doc-links.py
python3 scripts/generate-reference.py --check
```

Every suite runs offline. No test reaches the network, reads a system clock, or depends on a fixture it did not commit.

## Suites

| Crate | What it covers |
|---|---|
| `contracts/*` | Unit tests inside each contract |
| `tests/cross-contract` | Atomicity and authorization across contract boundaries |
| `tests/emergency` | Pause, unpause, and admin rotation under adversarial ordering |
| `tests/events` | Event shape, ordering, and indexer compatibility |
| `tests/event-fixtures` | Golden event fixtures under `tests/fixtures/events` |
| `tests/encoding` | Hashing and encoding vectors under `tests/fixtures/encoding` |
| `tests/time` | Ledger-time boundary behaviour |
| `tests/budgets` | Resource budget regressions (run in release) |
| `tests/ledger-snapshots` | Serialized ledger state and emitted events per lifecycle state |

---

## Ledger snapshot regression fixtures

### What they are for

A test written through a contract client checks what a call **returns**. It does not check what the call **left on the ledger**. A change to how a record serializes, a field that silently changes type, a key that moves, or an event that gains a topic will pass every assertion about return values and still break every indexer reading the chain.

The snapshot suite closes that gap. For five representative lifecycle states it builds a small synthetic deployment, renders every contract-owned ledger entry and every emitted event into normalized text, and compares the result against a committed fixture in [`tests/fixtures/ledger-snapshots/`](../tests/fixtures/ledger-snapshots).

### The five states

| Fixture | State |
|---|---|
| `initialized.snap` | All three contracts provisioned, one schema version approved, no issuer or proof records |
| `active.snap` | One issuer registered, one valid proof registered |
| `paused.snap` | As `active`, with the protocol pause flag engaged |
| `revoked.snap` | Proof revoked and issuer revoked, both terminal states |
| `expired.snap` | As `active`, with ledger time advanced past the proof expiration; the record is untouched |

`expired` is the state a verifier meets most often and the one most likely to be mishandled, which is why it gets a fixture of its own even though its storage bytes are identical to `active`.

### What a fixture contains

Four sections:

- **`[ledger]`** - the sequence and timestamp the snapshot was taken at. Every scenario sets both explicitly, so this is deterministic context, and it is what separates `expired` from `active`.
- **`[storage]`** - every entry each contract holds in instance, persistent, and temporary storage, sorted so host iteration order cannot move a line.
- **`[events]`** - every event emitted over the whole scenario, in emission order, in the XDR form an indexer receives. Order is part of the contract with indexers and is never sorted away.
- **`[verdicts]`** - what the read-only entry points return in this state. Storage records what happened; verdicts record what a verifier concludes from it. A change that left the bytes intact but flipped a verdict is exactly what a storage-only snapshot would miss.

### What normalization does and does not remove

The renderer in [`tests/ledger-snapshots/src/render.rs`](../tests/ledger-snapshots/src/render.rs) is the only path into a fixture. Two rules govern it.

**It excludes host metadata.** Host object handles, live-until ledgers, entry sizes, and budget counters describe the environment a call ran in, not the state the contract produced. They move for reasons unrelated to compatibility, and a fixture that churned on every unrelated change would stop being read. Values are rendered from `ScVal`, the serialized form, which carries none of it.

**It hides no contract state.** Every field of every record is rendered in full. The one substitution is the address alias table, which replaces a generated address with the role it plays (`addr:issuer`, `addr:protocol-config`). The substitution is total: an address with no alias renders as `addr:<UNALIASED>`, and a test rejects any fixture containing it.

Three tests keep the normalization honest:

- `rendering_is_deterministic` builds each scenario twice and requires identical output. Anything that varied would be metadata the renderer failed to exclude.
- `the_normalization_hides_no_stored_entry` walks the real storage of every contract and requires each entry to appear in the rendered body.
- `each_state_is_distinguishable_from_the_others` requires all five fixtures to differ. Five identical fixtures would pass every other test and detect nothing.

### No real addresses or production identifiers

Every address is generated by the test environment and appears in fixtures only as an alias. Identifier hashes are repeated single bytes (`0x11`, `0x33`) chosen so a reader can tell them apart at a glance. A test scans every fixture for anything shaped like a Stellar strkey - 56 uppercase base32 characters starting `G` or `C` - and fails if it finds one, whether or not it was ever real.

### Updating a fixture

A snapshot diff is a compatibility signal. Sometimes it is the intended one, and then the change needs an explanation a reviewer can act on.

```powershell
./scripts/update-ledger-snapshots.ps1 -Reason "revoked_at is now recorded on admin revocation"
```

The script requires the reason, passes it to the guarded regenerator, and re-runs the snapshot tests. Each fixture header then carries:

```text
# scenario: revoked
# revision: 2
# reason: revoked_at is now recorded on admin revocation
# body-digest: <sha256 of the body>
```

The digest is what makes the reason binding. A body cannot change without the digest changing, the digest is written only by the regenerator, and the regenerator refuses to run without a reason of at least twenty characters. An intended update therefore reaches review as a body diff, a bumped revision, and a written explanation, in one commit. A fixture edited by hand fails `every_fixture_header_is_well_formed` instead.

Put the same explanation in the pull request description. The header is for whoever reads the fixture in a year; the description is for whoever reviews it today.

### When a snapshot test fails unexpectedly

Read the diff before regenerating. The suite is designed so that the diff itself tells you what changed:

| Diff | Likely cause |
|---|---|
| A `[storage]` line changed shape | A record's serialization changed. Check `packages/shared` types. |
| A `[storage]` line appeared or vanished | A storage key was added, removed, or moved between durability classes. |
| An `[events]` line changed | An event gained, lost, or reordered a field. This breaks indexers. |
| An `[events]` line moved | Emission order changed. This also breaks indexers. |
| A `[verdicts]` line flipped | A read-only entry point now reaches a different conclusion from the same state. |
| `addr:<UNALIASED>` appeared | A scenario gained an address without registering an alias for it. |

Regenerate only once you can say which of these it is, and why the new content is correct.
