# WORK-0008 — Immutable Revision model

Status: BLOCKED ON DG-0003 AND DG-0004
Owner agent: Core Engineer
Milestone: M1
Branch: `work/0008-revision-model`

## Objective

Represent an immutable content-addressed Revision that identifies exactly one complete Project State and preserves its immutable parent Revision relationships.

## Normative requirements

- Glossary: Revision, Revision Identifier, Parent Revision, Revision Graph.
- Core Specification, sections 4–5, 14–15, and 55.
- Core Invariants: INV-HIST-001–004, INV-RES-002, INV-PROJ-001.

## Dependencies

- WORK-0001 through WORK-0003 and WORK-0007.
- DG-0003 must resolve ordering semantics for hashed parent/provenance collections.
- DG-0004 must resolve the Actor Identifier representation included in Revision identity.

## Allowed scope

- `crates/omvcs-model/`
- Focused Revision validation/identity tests and conformance vectors explicitly permitted by this package.

## Deliverables

- Immutable Revision referencing exactly one complete Project State.
- Explicit parent Revision references, author Actor reference, UTC RFC 3339 creation timestamp, message/description, and specified provenance metadata.
- Identity independent of platform, storage location, credentials, local paths, replica availability, and UI state.

## Acceptance tests

- Every Revision references exactly one complete Project State.
- Initial Revisions accept zero parents; parent relationships of a published Revision cannot be changed.
- A changed creative state requires a distinct Revision; operational storage movement does not.
- Timestamp order alone never establishes ancestry.
- Platform/storage/replica changes leave Revision identity unchanged.
- Multi-parent and provenance ordering tests remain blocked until DG-0003 is resolved.

## Explicit non-goals

- Lines, Releases, Working State, publication transactions, signatures, or authorization.
- Defining Actor Identifier format, authentication, or whether signing is mandatory.
- Permitting history rewriting or deriving ancestry from timestamps.

## Known Design Gaps

- DG-0003 — canonical order of hashed collection fields.
- DG-0004 — Actor Identifier representation in Revision identity.

## Implementation plan

1. Implement the immutable Revision structure after Project State and canonical identity contracts are resolved.
2. Hash only the specified immutable historical object body.
3. Add direct identity, ancestry immutability, and infrastructure-independence tests.

## Verification requirements

The Verifier must check complete-state references, immutable parentage, timestamp/ancestry separation, and identity independence against the original specifications.

## Completion criteria

Formatting, focused tests, coverage-map update, independent verification, handover, and clean Git state.
