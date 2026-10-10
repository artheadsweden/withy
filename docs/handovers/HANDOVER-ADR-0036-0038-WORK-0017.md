# Handover — ADR-0036–0038 / WORK-0017 readiness

From agent: Spec Guardian
To agent: OMVCS Lead / next implementation owner
Date: 2026-10-10
Branch: `spec/0003-canonical-collection-order`
HEAD at decision/specification closeout:
`c9834059323fcbf83e7745fb4698aa6a26cebfe3`

## Completed

Resolved the three approved semantic gates for WORK-0017 in distinct ADRs:
the OMVCS 0.1 reference chunking policy, verification strength versus
method/evidence, and destination Resource assurance. Updated affected
normative specifications, invariants, glossary, decision register, coverage,
M3 preflight, project state, and downstream work packages. WORK-0017 is
READY / AUTHORIZED and not started. WORK-0020 and WORK-0021 status wording
now distinguishes implementation prerequisites from unresolved decisions.

## Specifications implemented

- Core Specification §§8–8.2, 32–35, 47–55, and 89.
- Storage Adapter Specification §§30–36, 54–58, 73–76, 109–112, 160,
  162, 183, 192, 204–205, 240–243, 263, and 270.
- Glossary: Chunk, Chunking Policy, Chunk Manifest, Resource Replica,
  Replica Addition, Content Verification, Verification Strength,
  Verification Method, Verification Evidence, Verification Result, and
  Corrupt Replica.
- Core Invariants: INV-RES-005, INV-STOR-005, and INV-INT-001–003.
- ADR-0007 Decision 6 and associated test wording were reconciled with the
  fixed OMVCS 0.1 chunk boundaries.

## Files changed

- `Specs/OMVCS Core Specification.md`
- `Specs/OMVCS Storage Adapter Specification.md`
- `Specs/OMVCS Glossary.md`
- `Specs/OMVCS Core Invariants Specification.md`
- `docs/decisions/ADR-0007-resource-reference-boundary.md`
- `docs/decisions/ADR-0036-reference-chunking-policy.md`
- `docs/decisions/ADR-0037-verification-strength-and-evidence.md`
- `docs/decisions/ADR-0038-destination-resource-assurance.md`
- `docs/decision-register.md`
- `docs/spec-coverage.md`
- `docs/milestones.md`
- `docs/project-state.md`
- `docs/plans/M3-STORAGE-PREFLIGHT.md`
- `docs/plans/WORK-0015-storage-primitives-contract.md`
- `docs/plans/WORK-0016-replica-storage-map.md`
- `docs/plans/WORK-0017-resource-chunk-verification.md`
- `docs/plans/WORK-0018-mock-storage-adapter.md`
- `docs/plans/WORK-0019-local-filesystem-storage.md`
- `docs/plans/WORK-0020-resource-replication.md`
- `docs/plans/WORK-0021-storage-migration.md`

## Tests added or changed

No executable tests were added or changed. WORK-0017's normative
acceptance/conformance cases were updated for exact chunk boundaries,
zero-byte representation, verification strength and evidence, destination
assurance, and promotion eligibility.

## Commands run

- Cross-Spec `rg` audits over all eight documents for chunk layout/policy,
  verification strengths, checksums, copy/promotion, and registration
  requirements: completed; no unresolved cross-Spec contradiction found.
- `git diff --check`: passed before commit.
- `git diff --cached --check`: passed before commit.
- Cargo tests, formatting, and Clippy were not run because this task changed
  only specifications and documentation.

## Semantic decisions made beyond the specification

`None`. The three semantic decisions were explicitly approved by the human
and recorded in ADR-0036/0037/0038 before updating normative text.

## Design Gaps discovered

No new Design Gap. DG-0034 remains OPEN and limited to syntax-specific
ProviderLocator schema-identifier validation; it does not block WORK-0017.

## Assumptions

None beyond the approved decisions.

## Known limitations

- WORK-0017's implementation and independent verification have not started.
- WORK-0020 awaits WORK-0017 verification implementation.
- WORK-0021 awaits WORK-0017 and WORK-0020 implementations.
- DG-0034 remains open; DEC-STORAGE-011 remains open.
- Deletion, GC, retention, and shared-namespace semantics remain excluded.
- No production code changed.

## Remaining work

Implement WORK-0017 from its updated work package, then obtain Storage
Engineer review and independent Verifier acceptance. Promotion integration
may be delivered in WORK-0017 or as a bounded WORK-0016 follow-up, without
changing the integrated WORK-0016 identity, Storage Map, or CAS contracts.

## Git state

Decision/specification commit: `c9834059323fcbf83e7745fb4698aa6a26cebfe3`.
Handover commit: `33226fbecee4738932dd8011e91690db38024763`. Remote
publishing is ENABLED for `origin`; both commits were pushed, local and
remote tips matched at the handover commit, and the working tree was clean.
