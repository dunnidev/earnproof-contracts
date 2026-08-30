# Deployment verification

## What already exists

Issue #96 asks for a deployment verification script that validates contract
deployment, initialization, and basic functionality. That script already
exists — as `scripts/verify-manifest.ps1 -Live`, delivered by
[#7](https://github.com/veridatum-labs/earnproof-contracts/issues/7)
("Add live on-chain state checks to deployment verification").

Checked against #96's acceptance criteria directly:

| #96 requirement | Covered by `verify-manifest.ps1 -Live` |
|---|---|
| Verifies contract IDs in deployment manifest | Yes — offline shape checks (`Assert-ContractId`) run before any network call, in both modes |
| Validates admin addresses configured correctly | Yes — confirms `get_admin` on all three contracts matches the manifest's `admin` field |
| Tests basic operations (issuer lookup, schema check, pause state) | Yes — `is_paused`, `is_schema_version_approved` for every listed schema version, and `get_issuer_status` for the manifest's `initialIssuer` |
| Verifies cross-contract wiring | Yes — confirms `proof-registry`'s `get_issuer_registry`/`get_protocol_config` references match the manifest's contract IDs |
| Outputs clear success/failure messages | Yes — pass/fail per check, with expected-vs-actual values on mismatch |
| Never modifies state (read-only verification) | Yes — every call in `-Live` mode goes through `Invoke-StellarRead`, which only ever runs `stellar contract invoke -- <read-only-function>`; no signing key or seed phrase is required |
| Works with both testnet and sandbox | Yes — `-Network` is an explicit parameter, defaulting to the manifest's declared network; `scripts/deployment-manifest.testnet.json` is the checked-in example |

There is no gap here to duplicate. A second script (`scripts/verify-deployment.ps1`, matching #96's literal suggested filename) implementing the same checks against the same manifest shape would be **redundant with `verify-manifest.ps1 -Live`, not additive** — it would either drift out of sync with it over time, or the two would need to be kept manually consistent forever. Neither is better than having one verification path.

## Verification procedure

1. **Deploy**, recording the manifest (contract IDs, WASM hashes, build
   metadata) per [`docs/compatibility.md`](compatibility.md)'s release
   requirements. Copy
   [`scripts/deployment-manifest.example.json`](../scripts/deployment-manifest.example.json)
   as a starting point.

2. **Offline validation** — always run this first, requires no network
   access:

   ```powershell
   pwsh -File scripts/verify-manifest.ps1 -Manifest <path-to-manifest>
   ```

   Confirms the manifest itself is well-formed: contract IDs and WASM
   hashes look like real Stellar values (not leftover placeholders),
   required fields are present, and — when `-Release` is also supplied — the
   release note's declared metadata matches the manifest.

3. **Live on-chain verification** — confirms the deployment actually came up
   correctly, not just that the manifest describing it is well-formed:

   ```powershell
   pwsh -File scripts/verify-manifest.ps1 -Manifest <path-to-manifest> -Live
   ```

   This is the read-only check #96 asks for. Example against the checked-in
   testnet manifest:

   ```powershell
   pwsh -File scripts/verify-manifest.ps1 `
     -Manifest scripts/deployment-manifest.testnet.json `
     -Live
   ```

   Optional parameters (all have sane defaults):
   - `-Network <name>` — overrides the manifest's declared network.
   - `-Source <identity>` — a read-only source identity for the CLI call; no
     signing key is required since every call is a read-only entry point.
   - `-CliPath <path>` — path to the `stellar` CLI binary, if not on `PATH`.
   - `-TimeoutSeconds <n>` / `-MaxRetries <n>` — transient-RPC-failure
     handling (connection resets, `502`/`503`/`504`) is retried automatically
     up to this many times before the check is reported as failed.

4. **On a mismatch**, the script reports the expected value (from the
   manifest) and the actual on-chain value side by side, and exits non-zero
   — safe to wire into a CI/CD gate that blocks promoting a deployment until
   verification passes.

## Sandbox vs. testnet

The same script and manifest shape work for both — `-Network` accepts
whatever network alias the `stellar` CLI has configured locally (e.g. a
local sandbox network started via
[`scripts/local-sandbox/`](../scripts/local-sandbox/), or `testnet`). There
is no sandbox-specific verification path, since the checks themselves (an
admin address, a pause flag, a schema approval, a cross-contract reference)
mean the same thing regardless of which network they're read from.
