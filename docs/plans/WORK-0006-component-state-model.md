# WORK-0006 — Component State model

Status: IMPLEMENTED
Owner agent: Core Engineer
Milestone: M1
Branch: `work/0006-component-state`

## Objective

Represent an immutable, content-addressed state of one Creative Component, including its stable Component reference and Resource dependencies, with optional parentage that preserves known lineage without fabricating unknown ancestry.

## Normative requirements

- Glossary: Component State.
- Core Specification, sections 5.1, 7, 10–11, and 23.
- Core Invariants: INV-HIST-001–003, INV-HIST-006, INV-RES-004, INV-RES-008, INV-PROJ-002, INV-PROV-003, INV-DAW-004.
- ADR-0007 for the generic Resource Reference schema and identity boundary.
- ADR-0008 for the exact `byte_length` integer range and rejection rules.
- ADR-0009 for applicable schema/Adapter validation and historical admission of Resource Reference `properties`.
- ADR-0010 for the one-field Creative Component object and separation of Component State from Project State membership.
- ADR-0011 for the closed OMVCS 0.1 Component State body, schema-owned metadata contract, exact hash preimage, and Resource Reference property-authority binding.

## Dependencies

- WORK-0001 through WORK-0005.
- Hashed metadata maps follow RFC 8785 object-member ordering; duplicate member names are rejected before canonicalization/hashing (ADR-0005).

## Allowed scope

- `crates/omvcs-model/`
- Focused Component State validation/identity tests and conformance vectors explicitly permitted by this package.

## Deliverables

- Immutable Component State identity referencing its Creative Component and required Resource Objects.
- Optional parentage representation and validation; distinguish known initial state from unknown/unasserted lineage.
- Canonical identity tests use resolved array collection rules and RFC 8785 map handling.

## Acceptance tests

- A published Component State cannot be changed in place.
- Each Component State identifies the Creative Component whose state it represents by its typed `component_id`; Component State creative fields do not expand the generic Creative Component object.
- A changed Resource reference produces a distinct Component State identity.
- Resource references use immutable Resource identifiers.
- Each embedded Resource Reference has a typed Resource Identifier and a `byte_length` equal to the complete Resource's byte count, represented as an integer in `0 ..= 9007199254740991`; reject negative, fractional, greater-than-maximum, string, and other alternate representations (ADR-0008).
- Embedded Resource Reference length vectors accept `0`, `1`, `9007199254740991` and reject `-1`, `1.5`, `9007199254740992`, and `"1"`.
- Every embedded Resource Reference without `properties` passes generic Core validation; any present `properties`, including an empty map, is admitted only after validation under the exact versioned context governing its Component State use.
- Rejected, unknown, unavailable, or non-unique property validation context prevents admission of that Component State as valid history; preserved unchecked candidates cannot be hashed or committed as valid history.
- Component State identity contains property values after successful validation but not validation status/evidence; context-driven shapes, nested array declarations, and recursive duplicate-member rejection are enforced.
- The closed OMVCS 0.1 body requires `schema`, typed `component_id`, `resources`, and `metadata`; permits optional `parents`; accepts empty `resources` and `metadata`; and rejects omitted required members and all additional top-level members.
- Unknown or unavailable Component State schema versions may be preserved as uninterpreted candidates but cannot be admitted as valid history or assigned a valid Component State Identifier.
- Metadata keys, shapes, meanings, nested schemas, and array classifications are validated only under the exact versioned Component State schema; unknown/unpermitted keys or invalid shapes fail historical admission.
- Each permitted property-bearing Resource Reference context is bound to exactly one exact versioned authority by the Component State schema/Adapter contract; no latest-version, installed-preference, key-heuristic, ambiguous, or independent Resource Reference schema-ID behavior is allowed.
- The hash preimage contains exactly `schema`, `component_id`, `resources`, `metadata`, and `parents` only when present; changes to any present body field, including `schema`, change identity; validation evidence and operational/presentation/storage/transport fields are absent.
- Changing a canonical Resource Reference field value changes Component State identity but does not change Resource Identifier for unchanged raw bytes; reordering `properties` map insertion does not change identity.
- An initial state with zero parents is valid; known derived states SHOULD record one or more parents.
- An explicit empty `parents` array identifies a known initial state; an omitted field does not imply initial state.
- Unknown historical lineage is never fabricated.
- An operation/provenance rule requiring derivation preservation MUST enforce parent recording.
- Parent/resource set-like permutations produce identical identities; duplicate set elements are rejected.
- Metadata map insertion-order permutations produce identical canonical bytes/identities; duplicate member names are rejected; map entries receive no additional element-byte sorting.

## Explicit non-goals

- Defining provenance beyond the specified Component State lineage relation.
- Adapter-specific interpretation, Component mapping, or DAW-state operations.
- Storage chunking, replica tracking, or publication transactions.

## Known Design Gaps

- DG-0007 and DG-0008 are resolved by ADR-0007 and ADR-0008; apply the same Resource Reference schema and safe-integer validation to embedded references.
- DG-0009 is resolved by ADR-0009; apply the same exact-context properties validation and unchecked-to-historical admission boundary to every embedded Resource Reference.
- DG-0011 is resolved by ADR-0011; implement its closed field set, schema-owned metadata semantics, exact canonical body, and property-authority binding without reopening the approved rules.

## Implementation plan

1. Implement the closed body and required/optional member validation from ADR-0011, preserving ADR-0003 parentage semantics.
2. Model the schema-owned metadata and exact Resource Reference validation-authority context without adding generic Core property meanings.
3. Model admitted Component State references and the exact immutable hash preimage.
4. Add identity, requiredness, rejection, reference-integrity, lineage, and permutation tests from the approved rules.

## Verification requirements

The Verifier must reject any parentage or collection-order behavior not authorized by resolved Specs and must test immutability and Resource identity references.

## Completion criteria

Formatting, focused tests, coverage-map update, independent verification, handover, and clean Git state.

## Independent verification — 2026-10-08

Result: **REJECTED pending implementation remediation**. Status remains
`IMPLEMENTED`, not verified. Reviewed implementation
`298e5d0198e9b125d8f9b258800ed4f2620ec196` against base
`0ff182ab005fff7d58cebdea73dcb36e82c33b42`.

Blocking finding W6-V001: embedded Resource References accept positional JSON
arrays without the required named fields. Both `[resource_id, 3]` and
`[resource_id, 3, "primary", "audio/wav", {}]` decode through JSON text and value
APIs, then admit as valid Component State history. The inherited
`ResourceReferenceCandidate` decoder delegates to a derived wire struct without
an object-only gate. This violates Core §§7, 10 and ADR-0007/0011; it is an
implementation defect, not an unresolved semantic question.

Retained `tests/component_state_acceptance.rs`: 12 independent tests, including
two failing regressions for this defect. The other ten pass, covering embedded
exact byte-length forms and lossy/alternate-type rejection, raw duplicate names,
canonical-equivalent Resource duplicate rejection, explicit JCS/hash preimage,
576 combined parent/resource permutations, strict field shapes, schema fallback
rejection, mismatched property binding, and incomplete metadata schema rejection.

The original 21 Component State tests pass. The full model gate with
`--no-fail-fast` completes all targets: **95 passed, 2 failed**, including six
passing compile-fail doctests. Formatting, locked strict Clippy and whitespace
checks pass. See the handover for commands and evidence.

No production code or Specs changed, no test weakened, and no new Design Gap or
unapproved metadata vocabulary introduced. The Core Engineer must remediate
W6-V001 under bounded scope and request another independent gate before
integration. WORK-0007/0008 remain unstarted; no push or integration performed.
