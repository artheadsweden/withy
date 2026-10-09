# OMVCS Implementation Milestones

These are development milestones, not OMVCS specifications.

## M0 — Repository/bootstrap/tooling

- validate local Rust workspace;
- validate Copilot agents/instructions/skills;
- extract unresolved 0.1 decisions from all Specs;
- classify decision dependencies;
- create initial spec-coverage map;
- identify contradictions that block M1;
- prepare bounded M1 work packages.

## M1 — Normative data model

Status: COMPLETE. WORK-0001 through WORK-0008 are verified and integrated at `abd0807ed2152aa3e331c6ca058796eb70cf4d4a`.

- identifiers;
- canonical serialization;
- hashes;
- Resource model;
- Creative Component model;
- Component State;
- Project State;
- Revision.

## M2 — Local repository history

Status: IN PROGRESS. WORK-0009 is verified and integrated at
`3139bc5eeb4e29ce931e217b6e815a3a88b75772`. DG-0016, DG-0020, DG-0021,
and DG-0022 are resolved by ADR-0016, ADR-0020, ADR-0021, and ADR-0022.
WORK-0010 is independently verified and integrated at
`ef1937dcef4079cd6bd863c584464691e1ac72e6`. DEC-CORE-008 remains separate for automatic deletion
retention/pinning, and DEC-INTERACTION-001/003 remain presentation decisions,
not blockers for the Core Line package.
ADR-0017 resolves the Release object and admission contract; WORK-0011 is
independently verified and integrated at
`e179d27147ea6efcac3aa46fedcecd34ddd5725f` on
`spec/0003-canonical-collection-order`. Its feature branch was pushed to
`origin/work/0011-releases`. Accepted, human-approved ADR-0023 resolves DG-0023 only for the
exact `CreateRelease` integrity/name-conflict overlap, with atomic failure
and unchanged stored objects/bindings; it defines no general failure precedence.
DEC-CORE-002 does not block the Release contract. Human-approved ADR-0018
resolves DG-0018's Working State semantics. Human-approved ADR-0024–ADR-0026
resolve DG-0024–DG-0026 respectively: destructive replacement
authorization, AdapterWorkingStateRef persistence/recovery, and operation
failure/retry/idempotency contracts. DEC-INTERACTION-004 remains separate
for temporary checkpoints and does not block WORK-0012. WORK-0012 is independently verified and integrated at
`fd68d969da671546ec1e00f5362a75c0a52ed61f`; its feature branch is pushed to
`origin/work/0012-working-state-lifecycle`. WORK-0013 is independently verified and integrated at
`a67e1811ce0e7e15001f8517d379d7da607d3bdf` for the explicitly partial
Line/Release root subset on `spec/0003-canonical-collection-order`; its
feature branch is pushed to `origin/work/0013-repository-reachability`.
DG-0027 is resolved by ADR-0030: the current persisted Working State's
present Base Revision and present component-source Component States are
safety-reference roots. ADR-0031 resolves DG-0028: missing Working State
safety-reference targets are `unresolved` without a declared-boundary
lookup. The WORK-0013/0014 implementation extensions are independently
accepted with no findings and await integration on
`work/0013-working-state-roots`. Contributions, configured archival pins,
and pending publication transactions remain required by Core §62 but are
outside this extension. Reachability remains partial and cannot claim global
unreachable status.
AdapterWorkingStateRef is not itself a reachability root. ADR-0027–0029
resolve DEC-CORE-004, DEC-CORE-009, and DG-0019. WORK-0014 was independently
accepted and integrated at `3f37fa1a981fa06408616b080edb6c56355bc1b0` for
the partial Line/Release scope. The ADR-0030/0031 Working State-root
extension has independently passed implementation verification and awaits
integration. Contributions, configured archival pins, and pending
publication transactions remain outside the M2 root set.
DEC-CORE-005/008, DEC-INTERACTION-004, and DG-0015 remain separate. M2
remains IN PROGRESS. No M3 work has started.

- Lines;
- Releases;
- Working State;
- history graph;
- reachability;
- repository validation.

## M3 — Storage abstraction

- generic storage contract;
- mock storage;
- local filesystem storage;
- Replicas;
- verification;
- replication;
- migration.

## M4 — Publication and recovery transactions

- publication state machine;
- idempotency;
- interruption/retry;
- repository operation log;
- recovery scenarios.

## M5 — Executable DAW Adapter contract

- generic language-neutral contract;
- mock DAW Adapter;
- capture/restore test scenarios;
- dependency/reproducibility primitives.

## M6 — Contributions and integration model

- Contributions;
- Contribution head movement;
- provenance;
- Creative Integration;
- Selective Integration model.

## M7 — Stable C ABI

- coarse public FFI;
- opaque handles where necessary;
- C header generation/validation;
- language-neutral compatibility tests.

## M8 — Ardour reconnaissance and first vertical slice

- exact source mapping;
- whole-session capture;
- Resource enumeration;
- restore;
- Version 1 -> Version 2 -> restore both;
- no rich selective integration yet.

Later milestones are defined only after the preceding contracts have stabilized.
