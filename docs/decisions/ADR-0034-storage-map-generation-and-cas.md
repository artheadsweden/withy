# ADR-0034 — Storage Map generation and compare-and-swap contract

Status: ACCEPTED
Date: 2026-10-10
Decision owner: Human
Related Design Gap: DG-0032

## Context

Core §34 required guarded Storage Map updates and atomicity but left the
generation domain, initial state, transition, and request/result contract
unspecified. Generic Storage Adapter conditional writes did not establish
the portable Core generation or atomicity boundary. Core §58 also contained
an example allowing two same-generation Replica additions to commute.

## Decision

1. Each Project has one `StorageMapGeneration` guarding its complete logical
   Storage Map. It does not scope to an individual Resource, Replica,
   Endpoint, or entry. It is operational concurrency metadata, not content
   identity, historical identity, provenance, or verification evidence.
2. `StorageMapGeneration` is an unsigned integer in the inclusive domain
   `0 ..= 9007199254740991` (`2^53 - 1`). Its canonical JSON representation
   is an exact JSON integer serialized under Core §5 (RFC 8785/JCS).
   Negative values, fractions, strings,
   floating-point encodings, and alternate textual forms (including
   negative zero, leading zeroes, decimal points, or exponent notation) are
   invalid. Its canonical decimal form is `0` or a nonzero digit followed by
   zero or more decimal digits. Implementations must preserve the exact
   value independently of native integer width.
3. A newly initialized empty Storage Map has generation `0`. This means an
   existing empty map, not an absent generation. Repository Home
   operational-metadata initialization establishes this state. An absent
   persisted map outside that boundary is missing/incomplete metadata and
   MUST NOT be treated as generation zero.
4. Core exposes an operation equivalent to
   `ApplyStorageMapMutation(project_id, expected_generation, mutation)`.
   A mutation is one logical request containing one or more changes already
   permitted by the Specs; this decision adds no Replica lifecycle
   operation.
5. The expected-generation comparison and validation of all requested
   changes against the same pre-mutation map state form one atomic decision.
   A stale expected generation returns a typed conflict, performs no
   mutation, and leaves all entries and generation unchanged. The result
   SHOULD include the observed current generation when available. Core does
   not retry, merge, or rebase; the caller rereads and explicitly
   reevaluates.
6. If the generation matches but any requested change or expected-generation
    representation is invalid, the whole request fails without changing any
    entry or the generation.
7. After generation match and successful validation, a request that leaves
    the logical map unchanged returns a typed `unchanged` result with the
    current generation and does not advance it. It cannot be used to bump
    the generation.
8. A state-changing request at the maximum generation fails atomically with
    a typed exhausted/representation-limit result. It does not mutate, wrap,
    saturate, or switch representation. A no-op at the maximum may still
    return `unchanged`.
9. The logical Storage Map and its generation are one guarded persistent
    state boundary. If the Adapter/Repository Home cannot provide the
    required conditional and atomic write, Core reports unsupported or an
    applicable provider failure rather than success. A provider ETag,
    object version, or conditional-write token may implement the guard
    internally but is not the portable Core generation.
10. Two state-changing requests for one Project based on the same
    generation conflict after the first state-changing request succeeds,
    even if their changes could commute. The second caller may reread and
    explicitly resubmit if its request is still valid.
11. This is Storage-Map-specific and does not inherit or modify Line
    generation semantics. It does not alter ADR-0032/0033 Replica identity,
    representation, locator, verification, or shared-namespace decisions.

## Rationale

A single Project-scoped generation gives callers one coherent version of
the complete mutable map and prevents partial visibility when a logical
mutation touches multiple Resources or Replicas. Keeping the Core generation
separate from provider concurrency tokens preserves portable observable
semantics across storage systems.

## Alternatives considered

- Per-Resource or per-entry generations: rejected because callers would not
  receive one coherent Project Storage Map snapshot and multi-entry atomic
  mutations would require a different concurrency contract.
- Treating a missing persisted record as generation zero: rejected because
  absence after initialization may signal missing/incomplete operational
  metadata, not an existing empty map.
- Automatically merging or retrying stale changes: rejected because it
  would apply a request against state the caller did not evaluate.
- Using provider ETag/version values as the Core generation: rejected
  because provider tokens can change for provider reasons and are not
  portable Core state.
- Reusing Line generation semantics: rejected because this is a
  Storage-Map-specific contract.

## Specification impact

- `Specs/OMVCS Core Specification.md` §§33–34 and 58.
- `Specs/OMVCS Storage Adapter Specification.md` §§44–45, 50–51, 124,
  126, and 261.
- `Specs/OMVCS Core Invariants Specification.md`, INV-INT-003.
- `Specs/OMVCS Glossary.md`, Storage Map Generation.
- `docs/gaps/DG-0032-storage-map-generation-and-cas.md`.

## Test impact

WORK-0016 must test exact generation parsing/serialization at zero, ordinary
values, and the maximum; rejection of negative, fractional, string,
floating-point, and alternate forms; initial empty-map generation zero
versus absent map state; Project-wide scope; successful multi-entry atomic
mutation with exactly one increment; no-op unchanged without increment;
stale conflict with all entries and generation unchanged and no implicit
retry; validation-failure atomicity; generation exhaustion; unsupported and
provider failure without success-shaped fallback; and separation of Core
generation from provider tokens.

## Implementation impact

WORK-0016 may implement the Replica/representation/locator models, Storage
Map structure, generation type, and guarded/CAS mutation contract. Verified
promotion/registration remains gated by the approved WORK-0017 result and
DEC-STORAGE-004/005. DEC-STORAGE-011 shared namespace semantics, deletion,
GC, and retention remain excluded.

WORK-0017–WORK-0021 may use the specified guarded mutation mechanics where
their approved package scope requires them; their verification, chunking,
layout, and deletion decision gates are unchanged.

## Compatibility / migration impact

No production code or persisted Storage Map record is changed by this ADR.
The new representation is an exact bounded JSON integer. Provider tokens
remain implementation details; no token-to-generation migration rule is
introduced.

## Notes

No semantic decision beyond the human-approved decision is made here.
