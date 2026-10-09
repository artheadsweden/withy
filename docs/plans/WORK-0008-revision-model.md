# WORK-0008 — Immutable Revision model

Status: BLOCKED — DG-0014 Revision schema and hash preimage unresolved
Owner agent: Core Engineer
Milestone: M1
Branch: `work/0008-revision-model`

## Objective

Represent an immutable content-addressed Revision that identifies exactly one complete Project State and preserves its immutable parent Revision relationships.

## Normative requirements

- Glossary: Actor Identifier (ActorId), Revision, Revision Identifier, Parent Revision, Revision Graph.
- Core Specification, sections 4–5, 14–15, 43, 55–56, 59–60, and 76–77.
- Core Invariants: INV-HIST-001–007, INV-RES-002, INV-PROJ-001.

## Dependencies

- WORK-0001 through WORK-0003 and WORK-0007.
- ADR-0010: Component identity is separate from Project State membership; a Revision continues to identify its Project State, which determines the participating Components and Component States.
- ADR-0011: Component State references consumed through Project State use the closed 0.1 body and canonical identity contract.
- ADR-0012: the Revision's Project State reference MUST resolve to a valid/admitted closed OMVCS 0.1 Project State whose identity hashes all five required historical members.
- DG-0014 blocks implementation until the exact Revision member set, field representations, provenance schema/requiredness, extension policy, and hash preimage are approved. Do not infer `project_id` membership from the conceptual example.

## Allowed scope

- `crates/omvcs-model/`
- Focused Revision validation/identity tests and conformance vectors explicitly permitted by this package.

## Deliverables

- Immutable Revision referencing exactly one complete Project State.
- Explicit parent Revision references, author Actor reference, UTC RFC 3339 creation timestamp, message/description, and specified provenance metadata.
- Identity independent of platform, storage location, credentials, local paths, replica availability, and UI state.

## Acceptance tests

- Every Revision references exactly one complete Project State.
- The referenced Project State is resolvable and valid/admitted under ADR-0012; invalid or unchecked Project State candidates cannot be referenced as valid historical state.
- Initial Revisions accept zero parents; parent relationships of a published Revision cannot be changed.
- A changed creative state requires a distinct Revision; operational storage movement does not.
- Timestamp order alone never establishes ancestry.
- Platform/storage/replica changes leave Revision identity unchanged.
- Revision author is a canonical lowercase UUIDv7 ActorId; profile/account/signing-key changes, including key rotation, do not change the ActorId or rewrite historical authorship.
- Parent permutations produce identical identities and duplicate parents are rejected under Core Specification §5.1. Provenance canonicalization/duplicate tests must follow the exact presence and entry schema approved through DG-0014; do not infer them from the array classification alone.

## Explicit non-goals

- Lines, Releases, Working State, publication transactions, signatures, or authorization.
- Defining Actor account-association proof, authentication, or whether signing is mandatory.
- Permitting history rewriting or deriving ancestry from timestamps.

## Known Design Gaps

- No direct ActorId representation gap remains; ADR-0002 resolves DG-0004.
- WORK-0007 is verified; DG-0012 is resolved by ADR-0012.
- DG-0014 is OPEN and BLOCKS-MILESTONE. Revision implementation and final conformance tests must not begin until the human decision is recorded in an ADR and affected Specs/test plans are updated.

## Implementation plan

1. Await resolution of DG-0014; do not implement Revision semantics while its member set, provenance contract, and preimage are unsettled.
2. After the approved ADR/Spec update, revalidate this work package and implement only the approved immutable Revision structure and canonical identity.
3. Add direct schema/admission, identity, ancestry, and infrastructure-independence tests derived from the resolved normative contract.

## Verification requirements

The Verifier must check complete-state references, immutable parentage, timestamp/ancestry separation, and identity independence against the original specifications.

## Completion criteria

Formatting, focused tests, coverage-map update, independent verification, handover, and clean Git state.
