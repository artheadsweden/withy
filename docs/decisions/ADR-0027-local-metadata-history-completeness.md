# ADR-0027 — Local metadata-history completeness

Status: ACCEPTED
Date: 2026-10-09
Decision owner: Human
Related Design Gap: DEC-CORE-004

## Context

Core 0.1 describes complete repository metadata while also permitting
resource-sparse local operation. It does not state whether all historical
metadata must be present in every local Repository, or how an implementation
may claim that its local history is complete.

## Decision

1. A local Repository MAY contain an intentionally incomplete subset of
   historical metadata. Local metadata completeness is operational state;
   it is not historical identity, provenance, Platform truth, or deletion
   authority.
2. Validation distinguishes:
   - `complete`: sufficient requested-scope coverage, all required
     references in that scope resolve, and no applicable declared boundary;
   - `declared_incomplete`: at least one required reference terminates at a
     matching declared history boundary;
   - `unresolved`: required metadata is absent without a matching declaration;
   - `not_assessed`: the requested scope or its providers do not permit a
     meaningful completeness determination.
3. A `complete` claim requires sufficient coverage of the requested scope
   and locally resolved required references. Incomplete coverage or the
   absence of observed errors in an incompletely assessed scope MUST NOT
   establish completeness.
4. A content-derived metadata Identifier is calculated and verified from
   the object's canonical body under its exact available schema. Local
   presence, resolution, or admission of referenced objects is not required
   for that body hash; their absence does not change the Identifier.
   Unknown/unavailable schemas or invalid object bodies still prevent
   establishing a valid Identifier.
5. An absent required target is not, by absence alone, proof of corruption.
   A matching declared boundary reports intentional incompleteness; without
   one, the reference is unresolved. Neither classification validates the
   missing target or waives existing historical-object admission rules. An
   object whose required target is absent remains unadmitted until the
   ordinary target-resolution checks pass, even though its body-derived
   Identifier remains valid.
6. A declared boundary is operational metadata and MUST NOT affect the
   identity or provenance of a historical object, Platform assertions, or
   deletion/retention authority.

## Rationale

This permits resource- and metadata-sparse local operation without
conflating missing metadata with corruption or asserting completeness from
partial observation. Keeping completeness operational preserves immutable
historical identities.

## Alternatives considered

- Require complete historical metadata in every local Repository. Rejected:
  local completeness is not mandatory.
- Infer intentional incompleteness whenever a target is absent. Rejected:
  an undeclared absence remains unresolved.
- Treat a declaration as validating or admitting the missing target.
  Rejected: a declaration describes an omission; it does not supply the
  target or establish its validity. Admission remains distinct from the
  body's independently verifiable content-derived Identifier.

## Specification impact

- `Specs/OMVCS Core Specification.md` §§2, 4, 10, 13–14, 18, 22, 29–30,
  55–56, 66–68, 82–83, and 89.
- `Specs/OMVCS Core Invariants Specification.md` INV-WORK-002,
  INV-RES-007, INV-REC-002–004, and INV-REC-006.
- `Specs/OMVCS Glossary.md`, Content-derived Identifier, Repository
  Metadata, and new completeness terms.
- `Specs/OMVCS Storage Adapter Specification.md` §127.

## Test impact

WORK-0014 must distinguish all four completeness states; verify that a
complete result requires adequate scope/provider coverage and resolved
references; show that missing metadata alone is not classified as
corruption; and verify body-derived Identifiers independently of reference
resolution while keeping admission strict.

## Implementation impact

WORK-0014 (M2) reports local completeness and exposes body-derived
Identifier verification independently of target resolution, while retaining
existing admission rules. M4 import/recovery work consumes the same
classification.

## Compatibility / migration impact

No historical object schema or identifier changes. Existing repositories
may need local operational completeness records and declared-boundary
records; their persistence and transport representation are not specified
here.

## Notes

This decision does not establish Contribution semantics, retention,
archival-pin policy, or Working State safety-reference roots.
