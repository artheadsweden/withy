# ADR-0041 — Filesystem Repository Home marker and discovery

Status: ACCEPTED
Date: 2026-10-10
Decision owner: Human
Related decision: DEC-STORAGE-014

## Context

Repository Home bootstrap and discovery examples did not establish a
portable filesystem root marker, supported layout version, or explicit
root-selection behavior.

## Decision

1. A filesystem-backed Repository Home stores its marker at
   `.omvcs/repository.json` relative to the explicitly configured root.
   `.omvcs` is reserved for OMVCS repository metadata at that root.
2. The marker is a closed canonical JSON object containing exactly:
   `schema`, `project_id`, and `layout`, with values:
   - `schema`: `omvcs.repository-home/0.1`
   - `project_id`: an existing canonical ProjectId
   - `layout`: `omvcs.storage-layout/0.1`
3. Use Core §5 canonical serialization and existing ProjectId
   serialization. No separate RepositoryId is added. The marker contains
   no repository-format field, namespace, generation, credentials, or
   additional members.
4. The marker establishes intentional repository-root status, marker
   schema, Project identity, and storage-layout version only. It does not
   establish completeness, Resource verification, Platform registration,
   publication state, or operation-log presence.
5. Discovery uses only a caller-selected candidate root and MUST NOT search
   parent directories. Missing marker yields `not_repository` or an
   equivalent typed result. Malformed/invalid marker yields invalid
   repository metadata. Valid but unsupported schema/layout yields an
   explicit unsupported-version result. A mismatch with a requested
   Project yields an explicit identity-mismatch result. Only a supported,
   valid marker with matching ProjectId permits Home interpretation.
6. Bootstrap requires an explicitly selected new or empty root. A
   populated unmarked directory MUST NOT be silently adopted. Marker
   creation is atomic and MUST NOT replace an existing incompatible
   marker. Bootstrap initializes the empty Storage Map and
   `StorageMapGeneration` 0 at the existing operational-metadata
   initialization boundary; the marker and map/generation remain distinct
   records. Failure to establish the root, marker, or required operational
   state MUST be explicit.
7. Unknown future layout versions fail explicitly. No best-effort layout
   interpretation or automatic layout migration is defined.

## Rationale

The marker makes filesystem Home selection explicit and rejects accidental
binding to an ancestor or unrelated populated directory while remaining
separate from repository completeness and operational authority.

## Alternatives considered

- Search ancestor directories for a marker: rejected because the caller
  must not be bound to an unintended parent repository.
- Infer repository identity from directory name: rejected; ProjectId is
  the existing authoritative identity.
- Put generation, namespace, or completeness in the marker: rejected;
  these are separate operational facts and namespace semantics remain
  unresolved.
- Best-effort interpretation of newer layouts: rejected; unsupported
  versions must fail explicitly.

## Specification impact

- `Specs/OMVCS Storage Adapter Specification.md` §§10, 124–126, 240, 243,
  and decision list §270.
- `Specs/OMVCS Core Specification.md` §29.
- `Specs/OMVCS Glossary.md`, Repository Home Marker.

## Test impact

Tests MUST cover exact canonical marker encoding and closed fields,
missing/malformed/unsupported/mismatched markers, explicit-root-only
discovery with no upward search, safe empty-root bootstrap, atomic
nonreplacement, and separation of marker from completeness, generation,
publication, and Resource verification.

## Implementation impact

WORK-0019 MUST implement the explicit marker/bootstrap/discovery contract.
Layout migration, S3 behavior, and reference-Adapter designation remain
outside scope.

## Compatibility / migration impact

The previous root-record and marker text was conceptual, not a fixed
persisted schema. Unknown layouts are not silently adopted; no automatic
migration is introduced.

## Notes

DEC-STORAGE-011 and DEC-STORAGE-013 remain OPEN. The marker carries no
namespace identifier, and the filesystem Adapter is not designated an
official reference implementation. Bootstrap atomicity, interruption
classification, and retry semantics are supplemented by ADR-0042.
