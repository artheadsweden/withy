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

Status: PREFLIGHT COMPLETE. WORK-0009 is the first planned, unblocked package; other feature packages await the decisions and gap resolutions identified in the M2 preflight. Production implementation has not started.

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
