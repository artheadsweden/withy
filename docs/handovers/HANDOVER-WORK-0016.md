# Handover WORK-0016

From agent: OMVCS Lead
To agent: Integration/closeout owner
Date: 2026-10-10
Branch: `work/0016-replica-storage-map`
HEAD: `9870e0ae2c6e34b6282d654bd2e97e7e89f0b0e8` (feature branch; integrated at `ee8d77ddfc9859ee8c7bcae13018332632f6efa4`)

## Completed

WORK-0016's bounded Replica/representation/ProviderLocator model, Project
Storage Map, generation/CAS mechanics, and authoritative persisted-map
reconstruction are implemented. The Storage Engineer and independent Verifier
accepted the scoped implementation. Verified candidate promotion, locator
grammar beyond the approved opaque envelope, shared namespace semantics, and
deletion/GC/retention remain excluded.

## Specifications implemented

- Core §§8–8.2, 29–35, 47–51, 58.
- Storage Adapter §§29–36, 44–45, 49–51, 71–75, 124, 126, 161–164, 252, 261.
- Glossary: Replica Identifier, Resource Replica, Storage Location, Storage
  Map, Storage Map Generation.
- ADR-0032, ADR-0033, ADR-0034, ADR-0035.
- INV-RES-003/005/006, INV-STOR-003–005, INV-INT-001–003.

## Files changed

- `crates/omvcs-model/src/replica.rs` and `crates/omvcs-model/src/lib.rs`
- `crates/omvcs-model/src/canonical.rs`
- `crates/omvcs-model/tests/replica_model.rs`
- `crates/omvcs-core/src/storage_map.rs`, `crates/omvcs-core/src/lib.rs`,
  `crates/omvcs-core/Cargo.toml`
- `crates/omvcs-core/tests/storage_map_operations.rs`
- Relevant Specs, ADR-0032/0033, DG-0033/0034, WORK-0016, decision register,
  spec coverage, milestones, and project state.

## Tests added or changed

- Replica identity/cardinality, representation binding, manifest availability,
  locator envelope, canonical persistence bytes and round-trip.
- Storage Map generation domain/initialization, atomic multi-entry CAS,
  stale conflict/no retry, no-op, invalid mutation, concurrent writers,
  persisted-state validation/reconstruction, exhaustion, unsupported and
  provider failure cases.

## Commands run

- Focused `omvcs-core` Storage Map tests: 20 passed.
- Focused `omvcs-model` Replica tests: 11 passed.
- Post-merge `cargo test --locked -p omvcs-core -p omvcs-model`: passed.
- Post-merge `cargo test --workspace --locked`: passed.
- Post-merge `cargo fmt --all -- --check`: passed.
- Post-merge warnings-denied Clippy for `omvcs-core` and `omvcs-model`,
  all targets: passed.
- Post-merge `git diff --check`: passed before status reconciliation.
- Storage Engineer review: ACCEPT.
- Independent Verifier review: ACCEPT.

## Semantic decisions made beyond the specification

None in implementation. Human decision DG-0033 was recorded in ADR-0035:
authoritative persisted-map reconstruction establishes prior successful
registration; decoding alone does not register a candidate.

## Design Gaps discovered

- DG-0033 is resolved by ADR-0035.
- DG-0034 remains OPEN and blocks only syntax-specific ProviderLocator
  schema-identifier validation.

## Assumptions

Repository Home implementations uphold their normative persistence authority
and atomic conditional-write obligations. Provider tokens remain distinct from
the Core StorageMapGeneration.

## Known limitations

Concrete provider durability, restart behavior, provider secret rejection,
and provider-specific CAS mapping require Storage Adapter conformance. No
versioned schema-identifier lexical grammar is claimed. Verification evidence,
upload assurance, shared namespace, ownership, deletion, GC, and retention
are outside this work package.

## Remaining work

WORK-0016 is VERIFIED / INTEGRATED at the bounded approved scope. Keep
WORK-0017–0021 unstarted. Before implementing WORK-0017 verification and
verified promotion, resolve DEC-CORE-001 and DEC-STORAGE-002 jointly for
chunking, DEC-STORAGE-004 for the verification-strength taxonomy, and
DEC-STORAGE-005 for post-upload full-hash-versus-verified-reconstruction
assurance. WORK-0017 must also receive its normal implementation
authorization. DEC-STORAGE-011 remains OPEN; shared namespace claims,
deletion, GC, and retention remain excluded. DG-0034 still blocks only
syntax-specific ProviderLocator schema-identifier validation.

## Git state

Working tree: CLEAN after integration and post-merge validation; documentation
reconciliation was committed on the integration branch.
Remote publishing: ENABLED for `origin` per `docs/project-state.md`.
Feature-branch and integration-branch pushes performed: YES.
