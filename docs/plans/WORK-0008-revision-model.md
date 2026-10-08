# WORK-0008 — Immutable Revision model

Status: BLOCKED ON WORK-0007
Owner agent: Core Engineer
Milestone: M1
Branch: `work/0008-revision-model`

## Objective

Represent an immutable content-addressed Revision that identifies exactly one complete Project State and preserves its immutable parent Revision relationships.

## Normative requirements

- Glossary: Actor Identifier (ActorId), Revision, Revision Identifier, Parent Revision, Revision Graph.
- Core Specification, sections 4–5, 14–15, 55, and 59.
- Core Invariants: INV-HIST-001–004, INV-HIST-007, INV-RES-002, INV-PROJ-001.

## Dependencies

- WORK-0001 through WORK-0003 and WORK-0007.
- ADR-0010: Component identity is separate from Project State membership; a Revision continues to identify its Project State, which determines the participating Components and Component States.
- ADR-0011: Component State references consumed through Project State use the closed 0.1 body and canonical identity contract.

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
- Revision author is a canonical lowercase UUIDv7 ActorId; profile/account/signing-key changes, including key rotation, do not change the ActorId or rewrite historical authorship.
- Parent/provenance permutations produce identical identities; duplicate set elements are rejected under Core Specification §5.1.

## Explicit non-goals

- Lines, Releases, Working State, publication transactions, signatures, or authorization.
- Defining Actor account-association proof, authentication, or whether signing is mandatory.
- Permitting history rewriting or deriving ancestry from timestamps.

## Known Design Gaps

- No direct ActorId representation gap remains; ADR-0002 resolves DG-0004.
- Upstream WORK-0007 must be completed before Revision implementation begins; no unresolved DG-0001 through DG-0005 blocks its design.

## Implementation plan

1. Implement the immutable Revision structure after Project State and canonical identity contracts are resolved.
2. Hash only the specified immutable historical object body.
3. Add direct identity, ancestry immutability, and infrastructure-independence tests.

## Verification requirements

The Verifier must check complete-state references, immutable parentage, timestamp/ancestry separation, and identity independence against the original specifications.

## Completion criteria

Formatting, focused tests, coverage-map update, independent verification, handover, and clean Git state.
