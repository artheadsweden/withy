# DG-0005 — Canonical ordering of hashed JSON object maps

Status: RESOLVED
Classification: BLOCKS-MILESTONE
Discovered by: Spec Guardian
Discovered during: ADR-0001 impact analysis
Date: 2026-10-08

## Relevant specifications

- `Specs/OMVCS Core Specification.md` §§5, 5.1, 10, 12–13.
- `Specs/OMVCS Core Invariants Specification.md`, INV-HIST-006.
- `Specs/OMVCS DAW Adapter Specification.md` §§20–22.
- `Specs/Ardour Reference Adapter Design.md` §§23, 31.
- `docs/decisions/ADR-0001-hashed-collection-ordering.md`.

## Problem

ADR-0001 resolves ordered sequences and set-like array collections: set-like array elements are sorted by canonical serialized element bytes and duplicates are rejected. M1 hashed schemas also contain JSON object maps, including Component State metadata, Adapter State bindings/metadata, Project State components/metadata, and Resource Reference properties. RFC 8785 orders JSON object members by member name. The approved set-like element sorting rule does not state whether map entries are within its scope or how it interacts with RFC 8785 member ordering.

## Why the previous specifications were insufficient

The previous rules did not explicitly state whether ADR-0001's array-only collection rule governed maps, leaving room for inconsistent interpretation alongside RFC 8785 object-member ordering.

## Affected work

- M1 canonical serialization and metadata hashing (WORK-0002 and WORK-0003).
- M1 Component State and Project State schemas containing hashed maps (WORK-0006 and WORK-0007).
- Conformance vectors for map-key ordering and set-like permutation equivalence.

## Can unaffected work continue?

The question is resolved by ADR-0005. M1 canonical map identity conformance may proceed under RFC 8785 object-member ordering and explicit duplicate-name rejection.

## Decision

Hashed JSON object maps are governed solely by RFC 8785 JSON Canonicalization Scheme object-member ordering. Map insertion order has no semantic significance. ADR-0001's canonical element-byte sorting applies only to set-like JSON arrays and MUST NOT apply to map entries. Duplicate object member names are invalid and MUST be rejected before hashing or canonical serialization. Values are normalized according to their own schemas.

## Resolution

Resolved by [ADR-0005](../decisions/ADR-0005-json-object-map-canonicalization.md). The Core Specification, Core Invariants, and DAW Adapter Specification have been updated, along with ADR-0001's explicit array-only scope. M1 conformance tests cover insertion-order invariance, RFC 8785 member ordering without additional entry sorting, duplicate-name rejection, and schema-directed normalization of map values.
