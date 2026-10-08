# WORK-0002 — Canonical metadata serialization

Status: PLANNED
Owner agent: Core Engineer
Milestone: M1
Branch: `work/0002-canonical-serialization`

## Objective

Provide the specified deterministic UTF-8 JSON Canonicalization Scheme representation for hashed OMVCS metadata objects without adding unspecified normalization rules.

## Normative requirements

- Core Specification, section 5.
- Core Invariants: INV-HIST-002, INV-RES-002.

## Dependencies

- WORK-0001 for identifier/schema references.
- ADR-0005 resolves hashed JSON object maps: RFC 8785 object-member ordering only, with duplicate-name rejection before canonical serialization/hashing and no additional entry sorting.

## Allowed scope

- `crates/omvcs-model/`
- Focused serialization tests and vectors explicitly permitted by this package.

## Deliverables

- Deterministic UTF-8 JSON serialization compatible with RFC 8785 semantics.
- A clearly defined boundary between the hashed canonical object body and external storage wrappers/headers.
- Schema-aware recursive normalization of ordered sequences and set-like collections; JSON object maps follow RFC 8785 member ordering and map values are normalized according to their schemas.

## Acceptance tests

- Equivalent JSON objects with different input member order serialize to identical canonical bytes.
- JSON object maps use RFC 8785 object-member ordering regardless of insertion order and receive no additional entry sorting.
- Raw JSON containing duplicate object member names is rejected before canonical serialization or hashing.
- Map values are normalized by their schemas, including set-like array normalization, without reordering map entries beyond RFC 8785.
- Canonical output has no semantically irrelevant whitespace and follows RFC 8785 string and number rules.
- Hashing input excludes storage wrappers, HTTP headers, database keys, and external signatures/timestamps unless specified as object fields.
- Independent canonicalization of the same supported object yields byte-identical output.
- Ordered sequences preserve order; reordered inputs produce distinct bytes where sequence order differs.
- Permutations of set-like elements produce identical canonical bytes; duplicate canonical elements are rejected.
- Nested array collection fields are normalized recursively and missing classifications are rejected.

Conformance vectors:

| Input JSON bytes | Expected canonical result |
|---|---|
| `{"z":0,"a":1}` | `{"a":1,"z":0}` |
| `{"a":1,"z":0}` | `{"a":1,"z":0}` |
| `{"b":"a","a":"z"}` | `{"a":"z","b":"a"}` |
| `{"a":1,"a":2}` | Reject before canonical serialization or hashing |

## Explicit non-goals

- Defining object schemas or optional/default field semantics not specified by the Specs.
- Inventing Unicode normalization or number behavior beyond RFC 8785 and the collection rules in Core Specification §5.1.
- Serializing provider, platform, or DAW-specific structures outside the M1 model.

## Implementation plan

1. Implement the RFC 8785-compatible canonical JSON boundary and reject duplicate member names before canonicalization.
2. Add published/independently checked serialization vectors for resolved object forms.
3. Apply the resolved collection classifications and normalization rules in Core Specification §5.1.

## Verification requirements

The Verifier must check byte-level determinism against RFC 8785 and Core Specification §5.1, including object insertion-order invariance, RFC 8785 map member ordering without additional entry sorting, duplicate-name rejection, array order preservation, set-like permutation invariance and duplicate rejection, and recursive schema-directed normalization.

## Completion criteria

Formatting, focused tests, coverage-map update, independent verification, handover, and clean Git state.
