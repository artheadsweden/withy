# WORK-0003 — Content-derived object hashes

Status: COMPLETE
Owner agent: Core Engineer
Milestone: M1
Branch: `work/0003-hashes`

## Objective

Implement the specified SHA-256 identities for raw Resource bytes and canonical metadata object bodies.

## Normative requirements

- Core Specification, sections 4–6 and 55.
- Core Invariants: INV-HIST-002, INV-RES-001–004.
- ADR-0001 and ADR-0005 govern the canonical metadata byte input consumed from WORK-0002; they do not change the hash preimages defined by Core Specification §§5–6 and 55.

## Dependencies

- WORK-0001 for typed digest identifiers.
- WORK-0002 for canonical metadata bytes.
- WORK-0001 and WORK-0002 are complete and integrated into the current M1 base branch.
- Hashed metadata bytes already follow Core Specification §5.1, ADR-0001, and ADR-0005 through WORK-0002; this package consumes those canonical bytes and does not repeat serialization.

## Hash input boundary

- Resource Identifier digest input is exactly the complete raw Resource byte sequence: `SHA256(resource_bytes)` (Core Specification §6).
- Metadata object digest input is exactly the canonical serialized historical object body supplied by WORK-0002: `SHA256(canonical_metadata)` (Core Specification §§5, 55).
- The `omvcs:<object-type>:sha256:` text is the typed identifier namespace around the digest. Do not prepend the object type, schema, or any domain-separation bytes to either digest input; the specified hash equations define the preimage.
- WORK-0003 owns only these digest calculations and construction of the corresponding typed content identifiers. It does not add object-schema or historical-model semantics.

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
- Resource hashing uses exactly the raw bytes as its input; metadata hashing uses exactly the canonical bytes produced by WORK-0002. Expected digest vectors confirm neither path prepends object-type/domain-separation data.
- Metadata hashing excludes external wrappers; a wrapper change that leaves the canonical object body unchanged leaves the metadata digest unchanged.
- Maps supplied with different insertion orders produce the same canonical metadata hash; duplicate member names are rejected before hashing.
- Hashed JSON object-map identities consume the RFC 8785 canonical bytes, with duplicate member names rejected before hashing.
- Moving a Resource between locations or changing replica availability leaves Resource/Project/Revision identities unchanged.
- Object-type namespaces remain distinct even when digest text is equal.
- Property tests cover byte sequences and demonstrate deterministic raw Resource hashing.
- If a streaming hash interface is implemented or supported, hashing an identical byte sequence in multiple chunk partitions MUST produce the same digest as whole-buffer hashing; this verifies SHA-256 streaming equivalence and does not define storage chunk semantics.

## Known Design Gaps

- None affecting this work package. DG-0003 and DG-0005 are resolved by ADR-0001 and ADR-0005. Metadata object hashing consumes the completed serialization contract in WORK-0002.

## Explicit non-goals

- Adding hash algorithms not specified for OMVCS 0.1.
- Chunk hashes, chunk manifests, or storage layouts.
- Adding object-type prefixes, schema labels, or domain-separation bytes to hash inputs beyond the exact Core Specification equations.
- Defining schemas, model semantics, provider/platform metadata, or storage behavior.

## Implementation plan

1. Implement SHA-256 over complete raw Resource bytes and verify published SHA-256 vectors.
2. Implement SHA-256 over WORK-0002 canonical object-body bytes and construct the matching typed identifiers.
3. Test input-boundary, typed namespace, infrastructure-independence, and (if provided) streaming/whole-buffer equivalence.

## Verification requirements

The Verifier must compare exact hash preimages and output identifiers to Core Specification §§4–6 and 55; confirm the typed object prefix is outside the digest input; verify streaming/whole-buffer equivalence if supported; and confirm that no filename, provider, location, Platform, credential, or replica value enters identity.

## Completion criteria

Formatting, focused tests, coverage-map update, independent verification, handover, and clean Git state.
