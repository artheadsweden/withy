# WORK-0002 — Canonical metadata serialization

Status: BLOCKED ON DG-0003
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
- DG-0003 must be resolved before canonical serialization of collection-valued M1 fields can be considered conformant.

## Allowed scope

- `crates/omvcs-model/`
- Focused serialization tests and vectors explicitly permitted by this package.

## Deliverables

- Deterministic UTF-8 JSON serialization compatible with RFC 8785 semantics.
- A clearly defined boundary between the hashed canonical object body and external storage wrappers/headers.
- Tests that do not assume unresolved ordering for set-like collections.

## Acceptance tests

- Equivalent JSON objects with different input member order serialize to identical canonical bytes.
- Canonical output has no semantically irrelevant whitespace and follows RFC 8785 string and number rules.
- Hashing input excludes storage wrappers, HTTP headers, database keys, and external signatures/timestamps unless specified as object fields.
- Independent canonicalization of the same supported object yields byte-identical output.
- Collection-order test vectors remain blocked until DG-0003 determines which arrays are ordered and which require sorting.

## Explicit non-goals

- Defining object schemas or optional/default field semantics not specified by the Specs.
- Inventing Unicode normalization, array sorting, or number behavior beyond RFC 8785.
- Serializing provider, platform, or DAW-specific structures outside the M1 model.

## Known Design Gaps

- DG-0003 — canonical order of hashed collection fields.

## Implementation plan

1. Implement the RFC 8785-compatible canonical JSON boundary.
2. Add published/independently checked serialization vectors for resolved object forms.
3. Leave affected collection rules blocked until the required decision is approved and Specs/tests are updated.

## Verification requirements

The Verifier must check byte-level determinism against RFC 8785 semantics and ensure no unapproved array-order rule was selected.

## Completion criteria

Formatting, focused tests, coverage-map update, independent verification, handover, and clean Git state.
