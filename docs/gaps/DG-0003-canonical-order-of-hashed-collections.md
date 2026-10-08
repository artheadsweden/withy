# DG-0003 — Canonical order of hashed collection fields

Status: RESOLVED
Classification: BLOCKS-MILESTONE
Discovered by: OMVCS Lead
Discovered during: M0
Date: 2026-10-08

## Relevant specifications

- `Specs/OMVCS Core Specification.md`, sections 5, 10–14.
- `Specs/OMVCS Core Invariants Specification.md`, INV-HIST-002 and INV-HIST-004.

## Problem

OMVCS 0.1 canonicalizes metadata objects using JSON Canonicalization Scheme semantics. That determines object-member ordering but preserves array order. Hashed M1 objects contain collection-valued fields, including Component State `parents` and `resources`, and Revision `parents` and `provenance`. The specifications do not state whether each collection is an ordered sequence or an unordered set, nor define an ordering rule for values whose order is not semantically meaningful.

## Why the current specifications are insufficient

For a Revision with multiple parents, two conforming implementations could serialize the same parent relationships in different array orders and compute different Revision identifiers. The current specifications establish immutable parent relationships and deterministic JSON serialization, but do not define the semantic ordering or canonical array construction needed for equivalent objects to have identical identifiers.

## Affected work

- M1 canonical serialization and hashing.
- M1 Component State and Revision schemas and conformance vectors.

## Can unaffected work continue?

Yes. Raw Resource hashing and canonicalization of objects without ambiguous collection ordering can proceed. Final identity conformance for affected hashed metadata objects must wait.

## Candidate directions

Non-normative options include defining particular fields as ordered sequences, or defining them as sets with a specified canonical sort key. No rule is selected here.

## Required decision

For each collection-valued field in hashed M1 objects, is order semantically significant? For unordered collections, what canonical ordering rule must every implementation apply before serialization?

## Resolution

Resolved for array-valued collections by [ADR-0001](../decisions/ADR-0001-hashed-collection-ordering.md), approved by the human decision owner on 2026-10-08. The follow-on question of hashed JSON object map ordering was recorded separately in DG-0005 and resolved by ADR-0005.

- Ordered sequences preserve semantic order and are never reordered during canonicalization.
- Set-like collections are sorted ascending by each element's canonical serialized bytes before containing-object serialization; duplicate canonical elements are invalid.
- Every hashed array-valued collection field explicitly declares its semantics, and nested collections are normalized recursively.
- JSON object maps use RFC 8785 member ordering solely; ADR-0001's set-like array element-byte sorting does not apply, and duplicate member names are rejected before hashing/canonicalization (ADR-0005).

The affected Core, Core Invariants, Glossary, DAW Adapter, Storage Adapter, and Ardour reference-design specifications have been updated. The array-ordering rules are no longer blocked by DG-0003, and hashed JSON object map handling is resolved separately by ADR-0005. No M1 blocking design gaps remain from DG-0001 through DG-0005.
