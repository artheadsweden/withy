# Handover M0 — Repository bootstrap and M1 planning

From agent: OMVCS Lead
To agent: OMVCS Lead / M1 owners after blocking decisions are resolved
Date: 2026-10-08
Branch: `work/m0-repository-bootstrap`
HEAD: `398761f71bbe2fa3bb6cb72aa4e293127236da85` (base before the M0 documentation commit)

## Completed

- Read `AGENTS.md`, development workflow, milestone/project-state documents, and all eight specifications.
- Audited the Rust workspace and Copilot customization structure.
- Had the Spec Guardian extract and classify all expressly listed unresolved 0.1 decisions: 92 entries across six specifications. No unresolved-decision lists were found in the Glossary or Core Invariants.
- Added four M1-blocking Design Gaps and cross-referenced applicable listed decisions.
- Populated M1 spec coverage and created eight bounded M1 model work packages.
- No M1 implementation was started; Specs remain unchanged.

## Specifications implemented

None. This is M0 planning and documentation only.

M1 package traceability is recorded in `docs/plans/WORK-0001-identifiers.md` through `WORK-0008-revision-model.md` and `docs/spec-coverage.md`.

## Files changed

- `docs/decision-register.md`
- `docs/spec-coverage.md`
- `docs/gaps/DG-0001-adapter-state-representation.md`
- `docs/gaps/DG-0002-component-state-parentage.md`
- `docs/gaps/DG-0003-canonical-order-of-hashed-collections.md`
- `docs/gaps/DG-0004-actor-identifier-representation.md`
- `docs/plans/WORK-0001-identifiers.md` through `WORK-0008-revision-model.md`
- `docs/handovers/HANDOVER-M0.md`

`docs/project-state.md` has a pre-existing worktree edit enabling publishing; it was not authored, staged, or included in the M0 documentation commit.

## Tests added or changed

None. Acceptance tests are specified in the M1 work packages and remain unimplemented.

## Commands run

- `cargo metadata --no-deps --format-version 1` — passed; nine workspace members.
- `cargo check --workspace` — passed.
- `cargo test --workspace` — passed; current bootstrap crates contain zero tests.
- `cargo fmt --all -- --check` — passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — passed.
- `git diff --check` — passed.
- Copilot agent, instruction, skill, workflow, and VS Code customization inventory — all expected paths present.
- `git remote -v` — `origin` fetch/push URL present and verified.

## Semantic decisions made beyond the specification

None. Milestone classifications were reviewed to assign chunking decisions to M3 and publication signing to M4; no normative behaviour was selected.

## Design Gaps discovered

- DG-0001 — Adapter State representation in Project State.
- DG-0002 — Component State parentage requirement.
- DG-0003 — canonical order of hashed collection fields.
- DG-0004 — Actor Identifier representation in Revision identity.

## Assumptions

- The work packages restrict scope to fields and behavior already explicit in the Specs.
- Licensing fields are excluded from M1 objects pending DEC-PLATFORM-016.
- Resource chunk layout remains outside M1 because Resource identity is defined over complete raw bytes.

## Known limitations

- The M1 data-model milestone cannot be completed until DG-0001 through DG-0004 are resolved through the ADR/Spec-update workflow.
- DEC-PLATFORM-016 also remains open; if licensing metadata is to be part of M1 Project/Revision objects, resolve it before adding those fields.
- The 92 listed 0.1 decisions remain OPEN; the register's `Blocks` field records the earliest affected milestone.
- The Rust workspace is bootstrap-only; successful tests currently execute zero test cases.

## Remaining work

1. Obtain human decisions for DG-0001 through DG-0004.
2. For each approved decision, record an ADR, update affected Specs and test plans, and update the affected work packages/coverage map.
3. Begin M1 only after its blocking gaps are resolved. Start with unblocked packages only if the Lead confirms they do not pre-empt the unresolved semantics.
4. Keep DEC-PLATFORM-016 out of the M1 schema unless explicitly resolved.

## Git state

Working tree: DIRTY — M0 documentation changes are committed; the pre-existing `docs/project-state.md` edit remains untouched.
Remote push performed: YES — the M0 branch was pushed to the verified `origin` remote after the documentation commit. Remote publishing was ENABLED in `docs/project-state.md`.
