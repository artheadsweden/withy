# DG-0006 — Adapter State component-bindings representation

Status: OPEN
Classification: BLOCKS-FEATURE
Discovered by: OMVCS Lead
Discovered during: M1 revalidation and planning
Date: 2026-10-08

## Relevant specifications

- `Specs/OMVCS Core Specification.md` §5.1, Adapter State `component_bindings` row.
- `Specs/OMVCS DAW Adapter Specification.md` §§20–22.
- `Specs/Ardour Reference Adapter Design.md` §§23, 31.
- ADR-0001 and ADR-0005.

## Problem

The generic Core and DAW Adapter specifications describe Adapter State `component_bindings` as a JSON object map keyed by Creative Component Identifier. The Ardour reference design instead depicts `component_bindings` as an array of binding records, each carrying a `component_id` and one or more native IDs. Both representations carry similar information but have different canonical schemas and hashed object identities.

## Why the current specifications are insufficient

The generic contract explicitly selects a map, while the Ardour reference design gives a different concrete representation without explaining whether it is an exception, a namespaced extension, or an example that must conform to the generic map. Choosing between those schema shapes changes canonical Adapter State bytes and identifiers. The generic map rule is clear for the M1 Core model, but the concrete Ardour schema requires reconciliation before implementing this feature.

## Affected work

- Ardour Adapter component-binding schema and its M8 implementation/reconnaissance.
- Any conformance vectors for the concrete Ardour Adapter State schema.
- WORK-0007 only if its scope expands from the generic canonical Adapter State contract to define the Ardour-specific binding schema.

## Can unaffected work continue?

Yes. M1 may implement the generic Adapter State metadata-object contract and the map canonicalization required by Core Specification §5.1. WORK-0007 MUST NOT define or implement an Ardour-specific schema. Unrelated M8 work may continue, but the concrete Ardour component-binding schema must not be treated as settled.

## Candidate directions

The following are discussion material only and are NOT approved decisions:

- Represent bindings as a JSON object map keyed by Component Identifier, with binding records as values.
- Retain an array of binding records and explicitly approve/amend the generic contract to accommodate that representation.

## Required decision

Should the Ardour Adapter State encode `component_bindings` using the generic Component-Identifier-keyed map, or should the generic contract be changed through an approved specification decision to permit the array-of-records representation?

## Resolution

UNRESOLVED
