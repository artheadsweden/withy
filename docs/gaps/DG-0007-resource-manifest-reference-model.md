# DG-0007 — Resource Manifest and Resource Reference model boundary

Status: OPEN
Classification: BLOCKS-FEATURE
Discovered by: OMVCS Lead
Discovered during: WORK-0004 Resource model revalidation
Date: 2026-10-08

## Relevant specifications

- `Specs/OMVCS Glossary.md`, Resource, Resource Object, Resource Identifier, Resource Manifest, Friendly Name, Chunk, and Chunk Manifest entries.
- `Specs/OMVCS Core Specification.md` §§5–8, especially §5.1 and §7–8.2.
- `Specs/OMVCS Core Invariants Specification.md`, INV-RES-001–007 and INV-PROJ-003.
- `Specs/OMVCS Storage Adapter Specification.md` §§6–8, 21–25, 33–35, 74–75, and 162.
- ADR-0001 and ADR-0005 for canonical treatment of hashed arrays and maps.
- WORK-0004 — Resource and Resource Reference model.

## Problem

The specifications distinguish a logical Resource, its immutable content-addressed Resource Object, a historical Resource Reference, a Resource Manifest, and a physical Chunk Manifest, but do not unambiguously define how the Resource Manifest relates to the Resource Reference in the M1 Core model.

Core Specification §7 gives a conceptual Resource Reference with `resource_id`, `byte_length`, `media_type`, `logical_name`, `role`, and `properties`, and says only `resource_id` identifies the Resource Object while the other fields describe its intended interpretation. Core Specification §5.1 classifies `Resource Reference.properties` as a map. The Glossary separately defines a Resource Manifest as immutable descriptive metadata that MAY include Resource Identifier, content length, media type, chunk structure, friendly filename, technical format, sample rate, channel count, and other format-specific metadata. It does not say whether that Manifest is a distinct hashed object, the schema of a Resource Reference, or descriptive metadata embedded in a containing hashed object.

The Core and Storage Adapter specifications define Chunk Manifests as physical reconstruction information, but do not settle whether any such fields or a Resource Manifest are part of the WORK-0004 logical model or a hashed historical object.

## Why the current specifications are insufficient

The listed terms and examples establish content identity and location independence, but do not establish a single schema boundary between Resource Reference and Resource Manifest. Treating the Manifest as a separate content-addressed object, treating it as a Resource Reference, or embedding its fields in a parent hashed object creates different object graphs and hash inputs.

The Glossary's MAY-listed fields do not specify which fields are present or required in the M1 historical model. Core §5 establishes that fields included in a hashed object body contribute to its canonical bytes and identity; Core §7 does not specify which Resource Manifest fields, if any, are fields of a Resource Reference included in a containing Component State. Core §5.1 classifies the `properties` map but does not define the missing object/schema relationship or field inclusion boundary.

Choosing a schema or identity boundary would therefore add normative model semantics rather than merely implement the existing content-hash and storage-independence rules.

## Affected work

- WORK-0004 Resource and Resource Reference model, specifically Resource Manifest inclusion, fields, and identity participation.
- Related model conformance tests and `docs/spec-coverage.md` Resource-model coverage.
- Downstream WORK-0005 and WORK-0006 only where their model fields or hashed references depend on the selected Resource Reference schema.

## Can unaffected work continue?

Other already completed work remains unaffected. WORK-0004 MUST NOT define or implement the disputed Resource Manifest/Reference schema or hash participation. Because WORK-0004 is the current M1 package and WORK-0005/0006 depend on it, do not begin those packages until the decision is resolved and affected plans are updated.

## Candidate directions

The following are discussion material only and are NOT approved decisions:

- Define Resource Manifest as a distinct immutable metadata object with a specified identity and relationship to historical Resource References.
- Define Resource Manifest as descriptive terminology for fields on a Resource Reference, with the containing historical object determining whether those fields enter its hash.
- Keep Resource Manifest outside M1 Core history and define the exact Resource Reference fields and hash participation separately.

## Required decision

For OMVCS 0.1, is Resource Manifest a distinct object, the schema of Resource Reference, or a separate non-historical description? Which Resource/Resource Reference fields are included in a containing historical object body, given that included fields contribute to that object's canonical identity? Are chunk structure and friendly filename excluded from that historical model or represented elsewhere?

## Resolution

UNRESOLVED
