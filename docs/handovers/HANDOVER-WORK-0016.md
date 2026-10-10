# Handover WORK-0016

From agent: OMVCS Lead
To agent: Integration/closeout owner
Date: 2026-10-10
Branch: `work/0016-replica-storage-map`
HEAD: `327dfa7` (implementation commit; documentation closeout pending)

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
- `cargo test --workspace --locked`: passed.
- `cargo fmt --all -- --check`: passed.
- Warnings-denied Clippy for touched crates/all targets: passed.
- `git diff --check`: passed before the latest documentation reconciliation;
  rerun at closeout.
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

Run final closeout validation, inspect the complete diff for provenance and
scope, commit coherently, and integrate only through the authorized branch
workflow. Keep WORK-0017–0021 unstarted. Do not implement verified promotion
until WORK-0017's result contract and DEC-STORAGE-004/005 are approved.

## Git state

Working tree: DIRTY while this handover was prepared. The status inventory
contained only WORK-0016 production implementation, tests, and directly
related Spec/ADR/gap/plan/coverage/milestone/project-state/handover
documentation; no unrelated files were identified. The implementation has
been committed; documentation closeout is the remaining local commit.
Remote publishing: ENABLED for `origin` per `docs/project-state.md`.
Remote push performed: NO
