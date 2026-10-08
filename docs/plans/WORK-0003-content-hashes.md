# WORK-0003 — Content-derived object hashes

Status: PLANNED
Owner agent: Core Engineer
Milestone: M1
Branch: `work/0003-content-hashes`

## Objective

Implement the specified SHA-256 identities for raw Resource bytes and canonical metadata object bodies.

## Normative requirements

- Core Specification, sections 4–6 and 55.
- Core Invariants: INV-HIST-002, INV-RES-001–004.

## Dependencies

- WORK-0001 for typed digest identifiers.
- WORK-0002 for canonical metadata bytes.
- Metadata collection vectors are additionally dependent on DG-0003.

## Allowed scope

- `crates/omvcs-model/`
- Focused hash/identity tests and conformance vectors explicitly permitted by this package.

## Deliverables

- Raw Resource identity calculated directly over complete Resource bytes using SHA-256.
- Metadata identity calculated over the canonical serialized object body using SHA-256.
- Clear treatment of object-type namespace in identifier formatting without asserting an unspecified alternative hash preimage.

## Acceptance tests

- Standard SHA-256 vectors pass.
- Identical raw byte sequences produce identical Resource identifiers regardless of filename or physical location.
- A one-byte Resource change produces a different Resource identifier.
- Metadata hashing consumes canonical object bytes and excludes external wrappers.
- Moving a Resource between locations or changing replica availability leaves Resource/Project/Revision identities unchanged.
- Object-type namespaces remain distinct even when digest text is equal.
- Property tests cover byte sequences and demonstrate deterministic raw Resource hashing.

## Explicit non-goals

- Adding hash algorithms not specified for OMVCS 0.1.
- Chunk hashes, chunk manifests, or storage layouts.
- Defining domain-separation bytes not required by the identifier format.
- Freezing hashes for unresolved collection order.

## Known Design Gaps

- DG-0003 affects metadata objects with collection-valued fields.

## Implementation plan

1. Add raw Resource hashing and its property tests.
2. Add metadata hashing over the canonical body from WORK-0002.
3. Add identity-independence tests for location and wrapper changes.

## Verification requirements

The Verifier must compare hash preimages and output identifiers to Core Specification sections 4–6 and 55 and confirm that no provider or platform value enters identity.

## Completion criteria

Formatting, focused tests, coverage-map update, independent verification, handover, and clean Git state.
