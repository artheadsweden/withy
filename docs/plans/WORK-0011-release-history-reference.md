# WORK-0011 — Immutable Release references

Status: VERIFIED — independently accepted; integrated at
`e179d27147ea6efcac3aa46fedcecd34ddd5725f`
Owner agent: Core Engineer
Milestone: M2
Branch: `work/0011-releases`

## Objective

Implement the approved immutable Release model and its association with an
admitted Revision, distinct from mutable Lines.

## Normative requirements

- Core Specification §§4.1–4.2, 18, 56, 62, 74, and 82–83.
- Glossary: Project, Revision, Line, Release, and Release Identifier.
- Core Invariants: INV-HIST-001–005, INV-HIST-009, INV-HIST-012,
  INV-GC-001, and INV-REC-002.

## Dependencies

- WORK-0001 through WORK-0008 verified identity and Revision APIs.
- DG-0017 is resolved by ADR-0017 and the corresponding Spec updates.
- DEC-CORE-002 remains open for separate Reference Render policy. Under
  ADR-0017, it does not block Release body or `CreateRelease` implementation;
  WORK-0011 MUST NOT add a render field, precondition, or render-dependent
  test.

## Allowed scope

- `crates/omvcs-model/` for the typed `ReleaseId` and immutable Release body.
- `crates/omvcs-core/` for atomic CreateRelease admission and its resolver/
  operation-boundary contracts.
- Focused Release identity, admission, uniqueness, and immutability tests.

## Deliverables

- The exact Release identity, body, required/mutable members, and name
  uniqueness rules selected by the approved decision.
- Creation/admission checks for the approved Release-to-Project/Revision
  association.
- Enforcement that an admitted Release continues to identify the same
  Revision.
- Exact Release-to-Revision root reference for later read-only reachability
  work; WORK-0011 does not implement graph traversal.

## Acceptance tests

- Tests cover the exact closed seven-member body, rejection of unknown
  members and `null` substitutions, exact schema availability/admission,
  canonical JCS bytes, and an exact canonical-body/ReleaseId golden vector.
- Identity changes when any of `schema`, `project_id`, `name`,
  `revision_id`, `created_at`, `creator_id`, or `description` changes.
- Project-scoped name uniqueness uses exact string/code-point equality;
  different Projects may reuse a name; no case folding, locale/filesystem
  normalization, Git normalization, or semver parsing occurs.
- Creation follows the approved preconditions and distinguishes
  idempotent exact duplicate success, name conflict, and identifier/body
  integrity violation. When one request simultaneously encounters an existing
  same-ReleaseId/different-canonical-body integrity violation and a
  Project/name binding conflict, tests MUST assert that Core returns the
  integrity violation, not the name conflict, and that failure is atomic
  with all stored objects and Project/name bindings unchanged. Compare
  before/after stored object bodies and binding values for this overlap.
- Failed creation leaves no partial Release and does not claim a name.
- The target Revision and its admitted Project State resolve to the same
  Project; cross-Project targets fail.
- An admitted Release cannot be moved to a different Revision.
- Resource-byte absence and lack of local materialisation do not block
  target validation.
- All body members are immutable; OMVCS 0.1 exposes no update/delete
  operation. No future-version deletion policy is inferred.
- No test or implementation equates Release with a Line, Git tag, or mutable
  reference.

## Explicit non-goals

- Line lifecycle, Working State, repository reachability, or garbage
  collection.
- Reference Render production or Adapter execution.
- Release reachability traversal (WORK-0013).
- Reference Render is wholly outside WORK-0011's implementation and tests;
  DEC-CORE-002 remains a separate open decision.

## Known Design Gaps

- DG-0017 is resolved by ADR-0017.
- DG-0023 is resolved by accepted, human-approved ADR-0023 and the Core §18 update. The
  integrity failure takes precedence over an overlapping Project/name
  conflict only for the exact simultaneous conditions recorded there;
  no general failure-precedence framework is defined.
- DEC-CORE-002 remains open but does not block the Release contract; no
  render-dependent Release behavior is in scope.

## Implementation plan

1. Revalidate this package against ADR-0017 and the updated Specs.
2. Implement the normative immutable Release object and admission rules.
3. Add closed-schema, naming, association, and immutability tests.

## Verification requirements

The independent Verifier must compare the implementation to the approved
Release contract, validate Project/Revision association, and confirm no
mutable-Line or Git-tag semantics were introduced.

Verifier acceptance: ACCEPT. No remaining actionable conformance findings.
The two test-coverage findings were resolved with direct missing-member and
resolver-identity-mismatch tests. Post-remediation locked workspace tests
passed (188 tests, 0 failures). ADR-0023's bounded `CreateRelease` overlap
precedence is now explicitly human-approved; this approval normalization
does not represent a new implementation audit or test run.

## Completion criteria

Formatting, focused and locked workspace tests, warnings-denied Clippy,
independent verification, coverage-map update, handover, feature-branch push,
no-fast-forward integration, and integration-branch push are complete.
