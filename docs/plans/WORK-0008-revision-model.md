# WORK-0008 — Immutable Revision model

Status: BLOCKED — DG-0015 leaves non-empty Revision provenance validation undefined; not yet implemented
Owner agent: Core Engineer
Milestone: M1
Branch: `work/0008-revision-model`

## Objective

Represent an immutable content-addressed Revision that identifies exactly one complete Project State and preserves its immutable parent Revision relationships.

## Normative requirements

- Glossary: Actor Identifier (ActorId), Revision, Revision Identifier, Parent Revision, Revision Graph.
- Core Specification, sections 4–5, 14–15, 23, 42–43, 55–56, 59–60, and 76–77.
- Core Invariants: INV-HIST-001–009, INV-COL-003–004, INV-RES-002, INV-PROJ-001.

## Dependencies

- WORK-0001 through WORK-0003 and WORK-0007.
- ADR-0010: Component identity is separate from Project State membership; a Revision continues to identify its Project State, which determines the participating Components and Component States.
- ADR-0011: Component State references consumed through Project State use the closed 0.1 body and canonical identity contract.
- ADR-0012: the Revision's Project State reference MUST resolve to a valid/admitted closed OMVCS 0.1 Project State whose identity hashes all five required historical members.
- ADR-0014: Revision uses the closed seven-member OMVCS 0.1 body, schema-owned provenance, canonical UTC nanosecond timestamp, same-Project admitted parent references, and the exact seven-member hash preimage.
- DG-0014 is resolved. Do not add a direct `project_id`, nested `author`, Line/Release fields, or generic provenance vocabulary.
- DG-0015 is open. Do not infer non-empty provenance entry shapes or operation-specific provenance requirements from examples; resolve this gap before handing off the complete package.

## Allowed scope

- `crates/omvcs-model/`
- Focused Revision validation/identity tests and conformance vectors explicitly permitted by this package.

## Deliverables

- Immutable Revision referencing exactly one complete Project State.
- Exact closed historical body containing `schema`, `project_state_id`, `parents`, `author_id`, `created_at`, `message`, and `provenance`, with no unknown top-level fields.
- Explicit parent Revision references, direct typed ActorId author, canonical UTC RFC 3339 nanosecond creation timestamp, required string message, and exact-schema-validated provenance objects.
- Identity independent of platform, storage location, credentials, local paths, replica availability, and UI state.

## Acceptance tests

- All seven required members are present; each omitted member and every unknown top-level member is rejected. Exact schema must be available and validate before admission.
- `project_state_id` is a typed identifier resolving to one valid/admitted Project State. Revision Project identity is obtained through that Project State; no direct `project_id` exists.
- `parents` is required and accepts an empty array, one parent, or multiple parents. Typed parent identifiers resolve to valid/admitted Revisions; each parent must have the same Project identity as the current Project State. Parent order does not affect identity, duplicate parents are rejected, and parent relationships are immutable after publication.
- Initial Revisions accept zero parents; normal derived Revisions generally have one; integration Revisions MAY have multiple. No generic first-parent, Line, branch/ref, or Release meaning is assigned to parent ordering. Timestamp order never establishes ancestry; ancestry remains a DAG.
- A changed creative state requires a distinct Revision; operational storage movement does not.
- `author_id` is direct typed ActorId in canonical lowercase UUIDv7 form. Profile, email, account, Platform, or signing-key changes (including rotation) do not affect authorship or identity; current account/profile/key availability is not required for admission.
- `created_at` requires exact `YYYY-MM-DDTHH:MM:SS.nnnnnnnnnZ` form; offsets, missing/fewer/more fraction digits, and finer-than-nanosecond inputs that cannot be represented exactly are rejected, never rounded/truncated. Timestamp is informational and participates in identity.
- `message` is required JSON string, accepts empty string, retains whitespace/case/Unicode exactly, and participates in identity.
- `provenance` is a required set-like array and accepts the valid empty array. Non-empty entry admission, nested-schema tests, and operation-specific provenance tests are blocked by DG-0015; do not invent test entries or infer a vocabulary from examples. Provenance is additional context, not a replacement for ancestry.
- Identical canonical Revision bodies produce identical IDs; valid changes to each settled canonical field change identity. Testing identity changes for non-empty provenance values is blocked by DG-0015. Operational, storage, Replica, Line/ref/branch, Release, Platform/account, credential, local DAW/runtime, signature/wrapper, validation-evidence, transport, and unknown fields are excluded.
- Metadata-object admission does not require local Resource-byte materialization where the corresponding admitted metadata objects are resolvable.

## Explicit non-goals

- Lines, Releases, Working State, publication transactions, signatures, or authorization.
- Defining Actor account-association proof, authentication, or whether signing is mandatory.
- Permitting history rewriting or deriving ancestry from timestamps.

## Known Design Gaps

- No direct ActorId representation gap remains; ADR-0002 resolves DG-0004.
- WORK-0007 is verified; DG-0012 is resolved by ADR-0012.
- DG-0014 is resolved by ADR-0014.
- DG-0015 blocks implementation and handoff of this complete package until the non-empty provenance schema and affected operation requirements are settled.

## Implementation plan

1. Begin only after DG-0015 is resolved and the package acceptance tests are executable; then implement the approved immutable Revision structure and canonical identity using WORK-0001 through WORK-0003 and admitted WORK-0007 APIs.
2. Add direct schema/admission, identity, ancestry, timestamp, provenance, and infrastructure-independence tests derived from ADR-0014 and the updated Specs.

## Verification requirements

The Verifier must check complete-state references, immutable parentage, timestamp/ancestry separation, and identity independence against the original specifications.

## Completion criteria

Formatting, focused tests, coverage-map update, independent verification, handover, and clean Git state.
