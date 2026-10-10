# ADR-0007 — Resource Reference and Resource Manifest boundary

Status: ACCEPTED
Date: 2026-10-08
Decision owner: Human
Related Design Gap: DG-0007

## Context

The Draft 0.1 Specs described Resource References and Resource Manifests without defining their relationship, their fields, or which fields participate in historical object identity. This left WORK-0004 unable to define a conforming Resource model.

## Decision

1. OMVCS 0.1 does not define Resource Manifest as a separate content-addressed historical object. Historical OMVCS objects refer directly to immutable Resource Objects using an embedded `ResourceReference`.
2. The generic Resource Reference contains:
   - `resource_id`: REQUIRED typed Resource Identifier;
   - `byte_length`: REQUIRED number of bytes in the complete Resource (non-negative);
   - `role`: OPTIONAL semantic role in the containing historical state;
   - `media_type`: OPTIONAL intended media/content type;
   - `properties`: OPTIONAL canonical JSON object map containing immutable interpretation metadata explicitly supplied by the relevant schema or Adapter.
3. Every field present in a Resource Reference is part of the containing historical object's canonical body and contributes to that object's identity. No Resource Reference field changes the Resource Identifier, which remains SHA-256 of the complete raw Resource bytes.
4. `logical_name`, Friendly Name, and filename are excluded from the generic historical Resource Reference in OMVCS 0.1. Renaming a Resource for presentation or local working purposes MUST NOT by itself alter historical creative state. If a DAW requires naming information for exact native reconstruction, that information belongs in Adapter State.
5. Chunk structure, Chunk IDs, Chunk Manifest information, Storage Endpoint, Storage Location, Replica information, credentials, provider metadata, and other physical reconstruction or storage details MUST NOT appear in the historical Resource Reference and MUST NOT affect historical object identity.
6. Chunk Manifests remain operational physical-reconstruction information.
   Replicas or Endpoints MAY use different provider-internal physical
   segmentation for the same Resource without changing the Resource
   Identifier or creative history. For OMVCS 0.1, every chunked OMVCS
   representation uses the fixed boundaries in ADR-0036; different
   OMVCS-level Chunking policies require explicit identification/versioning
   in a future protocol version.
7. In OMVCS 0.1, Resource Manifest is descriptive terminology for a Resource-oriented metadata/reconstruction view that may combine historical Resource Reference information with operational reconstruction information. It has no independent content-derived historical identifier. A future formal Resource Manifest object requires an explicit schema/version decision and MUST NOT change existing Resource Identifiers.

## Rationale

The Resource Identifier represents only the complete immutable bytes. A containing historical object's Resource Reference records the schema-approved facts needed to interpret those bytes; because it is embedded in that historical body, those facts participate in that object's identity. Presentation names and physical reconstruction choices are not generic creative-state references and must remain independent of Resource and historical object identity.

## Alternatives considered

- Define Resource Manifest as a separate content-addressed historical object: rejected for OMVCS 0.1.
- Treat Resource Manifest as the Resource Reference schema: rejected; Resource Manifest remains descriptive terminology, while Resource Reference is the embedded historical structure.
- Include filenames, chunk layouts, or physical storage details in the generic Resource Reference: rejected because these values are presentation or operational details and must not affect historical object identity.

## Specification impact

- `Specs/OMVCS Glossary.md`: Resource Manifest and Friendly Name definitions.
- `Specs/OMVCS Core Specification.md`: §§5.1, 6–8, 10, 13, and 56.
- `Specs/OMVCS Core Invariants Specification.md`: INV-RES-004 and INV-PROJ-003.
- `Specs/OMVCS Storage Adapter Specification.md`: §§7 and 162.
- `Specs/OMVCS DAW Adapter Specification.md`: §16.
- `Specs/Ardour Reference Adapter Design.md`: §§15 and 31.

## Test impact

WORK-0004 and dependent model acceptance tests/test plans MUST cover:

- required typed `resource_id` and non-negative complete-resource `byte_length`, plus optional `role`, `media_type`, and schema/Adapter-supplied `properties`;
- changes to the canonical value of a present Resource Reference field change the containing historical object's identity, while Resource Identifier remains determined only by raw Resource bytes; reordering `properties` map insertion does not change canonical identity;
- rejection/exclusion of logical/friendly filename and physical storage/reconstruction fields from generic Resource References;
- presentation/local renaming, storage location, Replica changes, and
  provider-internal physical segmentation do not change Resource or
  historical identity; OMVCS Chunk boundaries follow the applicable
  versioned policy;
- required DAW reconstruction naming is represented in Adapter State rather than generic Resource Reference;
- Resource Manifest has no separate content-derived historical identifier in 0.1.

Existing RFC 8785 object-map ordering and duplicate-name rejection for `properties` remain governed by ADR-0005; array ordering remains governed by ADR-0001.

## Implementation impact

- WORK-0004 owns the logical Resource and Resource Reference model.
- WORK-0005 consumes the Resource/Component distinction.
- WORK-0006 embeds Resource References in Component State.
- WORK-0007 embeds Resource References within Adapter State resources.
- WORK-0003 remains limited to the already specified raw-byte and canonical-body hash preimages.

No production code is part of this ADR task.

## Compatibility / migration impact

No production objects or implementations are known in this workspace. Draft 0.1 examples containing `logical_name` in generic Resource References are corrected by the accompanying Spec update. Any future formal Resource Manifest schema must preserve existing Resource Identifier semantics.

## Notes

This ADR resolves DG-0007 only. The exact `byte_length` integer range and invalid forms are specified by ADR-0008. This ADR does not define chunking algorithms, Chunk Manifest identity, storage metadata schemas, Adapter State naming schemas, or additional Resource Reference properties.
