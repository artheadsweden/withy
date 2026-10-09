# WORK-0010 — Line history references

Status: PLANNED
Owner agent: Core Engineer
Milestone: M2
Branch: `work/0010-line-history-reference`

## Objective

Implement the approved OMVCS Line record and lifecycle for mutable references
to Revisions, without importing Git branch or ref semantics.

## Normative requirements

- Core Specification §§4.1, 16–17, 56–58, 74, and 82–83.
- Glossary: Line and Default Line.
- Core Invariants: INV-HIST-005, INV-HIST-010, INV-GC-001, and INV-UX-002.
- ADR-0016, ADR-0020, and ADR-0022.

## Dependencies

- WORK-0001 through WORK-0008 verified identifier and Revision APIs.
- WORK-0009 verified Revision graph traversal and admitted-Revision
  resolution.
- DG-0016 is resolved by ADR-0016 and corresponding Spec updates.
- DG-0021 is resolved by ADR-0021, which defines the shared, repository-
  authoritative Default Line preference and `SetDefaultLine`.
- ADR-0020 defines the interoperable Line generation domain and exhaustion
  behavior.
- ADR-0022 defines DeleteLine's expected-generation concurrency contract.
- ADR-0021 defines Default Line persistence, compare-and-swap updates, local
  selection separation, and deletion protection.
- DEC-CORE-008 separately governs automatic pin/retention after deletion; it
  does not authorize deleting historical objects as part of DeleteLine.
- DEC-INTERACTION-001 and DEC-INTERACTION-003 concern presentation terminology
  and numbering; this Core package does not define those presentation choices.

## Allowed scope

- `crates/omvcs-core/`
- Focused Line model, reference-validation, and lifecycle tests.

## Deliverables

- The closed five-member Line record with stable assigned UUIDv7 identity.
- CreateLine, MoveLine, and RenameLine with Project-scoped name uniqueness,
  atomic failures, and the specified generation/CAS behavior.
- Project-consistent admitted Revision target validation without requiring
  Resource-byte availability.
- DeleteLine with atomic `line_id` plus `expected_generation`; no
  historical-object deletion.
- Optional Project-scoped Default Line preference persisted as repository-
  owned operational metadata, plus atomic `SetDefaultLine`.

## Acceptance tests

- Tests reject missing or additional Line members and validate assigned
  UUIDv7 Line IDs independently of name and target.
- Tests enforce exact name equality and uniqueness within a Project, allow
  equal names across Projects, and allow multiple Lines to share one target.
- CreateLine starts at generation zero and is atomic; target metadata must be
  admitted and belong to the supplied Project without requiring Resource
  bytes.
- MoveLine tests stale expected-target and stale-generation conflicts,
  cross-Project rejection, arbitrary same-Project admitted movement,
  field preservation, exactly-one generation increments (including a
  successful move to the current target), and atomic failure.
- RenameLine tests identity/target/project preservation, conflict handling,
  exactly-one generation increments (including a request for the existing
  name), and atomic failure.
- Generation tests accept `0`, ordinary positive values, and
  `9007199254740991`; reject `9007199254740992`, negative, fractional, and
  string values; and establish canonical serialization stability.
- Increment tests prove exactly-one successful increments and atomic failure
  at the maximum with no wrapping, reset, silent saturation, or value reuse.
- DeleteLine succeeds with the current generation, conflicts on stale
  generation without changing the Line, and distinguishes a missing Line
  from a stale-generation conflict. Success removes only the Line record;
  referenced historical objects remain unaffected. Retry requires the
  caller to obtain and reason from current state.
- Default Line is optional, Project-scoped, and designates only an existing
  Line in an existing Project; same-Project validation, current-value CAS,
  clearing, stale conflict, atomic failure, and stored-reference admission
  are covered.
- Setting/clearing the Default Line does not change Line records,
  generations, Revision history, or roots; Line creation does not implicitly
  set it.
- Repository Home is authoritative; Platform mirrors and client-local
  current-Line selection cannot redefine the shared value.
- Deleting the current Default Line fails atomically; concurrent
  SetDefaultLine/DeleteLine decisions cannot leave a dangling reference.
- DEC-CORE-008 retention/pinning tests are excluded; no such policy is
  inferred by the DeleteLine tests.
- DEC-CORE-008 remains separate for automatic pin/retention; the Default
  Line is not an additional history root.
- No test or implementation assumes Git branch, force-push, reflog, or
  detached-HEAD behavior.

## Explicit non-goals

- Release semantics, Working State lifecycle, repository reachability, or
  garbage collection.
- Musician-facing terminology and display numbering.
- Automatic Line-deletion pin/retention behavior under DEC-CORE-008.

## Known Design Gaps

- DG-0016 is resolved by ADR-0016.
- DG-0021 is resolved by ADR-0021.
- ADR-0020 resolves generation numeric-domain-dependent behavior.
- ADR-0022 resolves DeleteLine concurrency semantics.
- DEC-CORE-008 separately governs Line-deletion automatic pin/retention.
- DEC-INTERACTION-001 and DEC-INTERACTION-003 remain presentation decisions;
  they are not prerequisites for the Core Line model.

## Implementation plan

1. Revalidate this package against ADR-0016, ADR-0020, ADR-0021, ADR-0022,
   and the updated Specs.
2. Implement the resolved Line record, operations, and separate repository-
   owned Default Line preference.
3. Add direct contract, CAS, default-preference, and atomicity tests.

## Verification requirements

The independent Verifier must check every approved member and operation rule,
CAS conflict behavior, target Project consistency, unresolved-gap boundaries,
and absence of inferred Git semantics.

## Completion criteria

Formatting, focused tests, coverage-map update, independent verification,
handover, and clean Git state.
