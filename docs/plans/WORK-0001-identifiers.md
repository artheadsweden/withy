# WORK-0001 — Normative identifier types

Status: PLANNED
Owner agent: Core Engineer
Milestone: M1
Branch: `work/0001-identifiers`

## Objective

Model the assigned and content-derived identifier classes explicitly, preserving the distinctions defined by OMVCS 0.1.

## Normative requirements

- Core Specification, sections 4, 6, 9–14.
- Glossary: Project Identifier, Creative Component, Resource Identifier, Revision Identifier.
- Core Invariants: INV-HIST-002, INV-RES-002, INV-RES-003, INV-PROJ-001–003.

## Dependencies

- Work is limited to identifier definitions whose formats are explicit in the cited requirements.
- Metadata-object digest inputs depend on WORK-0002 and unresolved DG-0003.
- Known open decisions are tracked in `docs/decision-register.md`; do not invent formats for unresolved identifiers.

## Allowed scope

- `crates/omvcs-model/`
- Focused model tests and conformance vectors explicitly permitted by this package.

## Deliverables

- Types distinguishing UUIDv7 assigned identifiers from typed SHA-256 content identifiers.
- Canonical lowercase UUID text and typed `omvcs:<object-type>:sha256:<digest>` formatting/parsing for specified object classes.
- No reliance on filenames, paths, storage endpoints, platform URLs, or credentials for creative identity.

## Acceptance tests

- Generated assigned identifiers use UUID version 7 and canonical lowercase textual form.
- Parsing rejects malformed UUID/content-identifier forms and preserves object-type namespace distinctions.
- Equal Resource bytes yield equal Resource identifiers; a changed byte yields a different identifier.
- Storage/path/platform/replica metadata changes cannot alter assigned Project/Component identity or content identifiers.
- Tests cover format boundaries and invalid digest encodings without silently accepting malformed IDs.

## Explicit non-goals

- Choosing formats for Actor, Line, Release, or other identifiers not settled by M1 requirements.
- Implementing canonical serialization, metadata hashing, storage locations, or repository operations.
- Introducing implementation-specific identifier semantics as OMVCS requirements.

## Known Design Gaps

- None directly. Metadata object identity remains dependent on DG-0003.

## Implementation plan

1. Map each in-scope normative identifier class to a distinct model type.
2. Implement only the explicit UUIDv7 and typed SHA-256 rules.
3. Add round-trip, namespace-separation, and invalid-input tests.

## Verification requirements

The Verifier must compare supported identifier classes and formatting against the cited Core sections and invariants, and check that no unassigned identifier format was invented.

## Completion criteria

Formatting, focused tests, coverage-map update, independent verification, handover, and clean Git state.
