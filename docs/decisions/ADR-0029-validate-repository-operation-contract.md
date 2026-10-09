# ADR-0029 — ValidateRepository operation contract

Status: ACCEPTED
Date: 2026-10-09
Decision owner: Human
Related Design Gap: DG-0019

## Context

Core §82 names `ValidateRepository`, but §83 requires operation-facing
contracts to define inputs, effects, failure modes, retry, idempotency, and
results. Validation needs to distinguish metadata integrity, local history
completeness, Resource availability, and incomplete provider coverage.

## Decision

### Invocation

1. `ValidateRepository` is strictly read-only. It MUST NOT repair objects,
   mutate metadata or availability/verification records, fetch or
   materialise Resources, change Lines/Releases/Working State, pin/archive
   objects, or delete data.
2. Each invocation explicitly supplies:
   - a scope: the whole local Repository or one identified Project; another
     narrower scope is permitted only if already defined by a normative
     specification; and
   - a Resource verification depth: `metadata_only`,
     `verify_available_resources`, or `deep_resources`.
3. Validation MUST echo the requested scope and depth in its result. It
   validates only within that scope and MUST NOT imply checks outside it.
4. The repository context MUST be open/readable and scope/depth inputs MUST
   be valid. Ordinary repository read authorization continues to apply; no
   mutation authorization is introduced. The operation has no historical,
   operational, or Working State effects.
5. An invalid request or inability to establish the requested repository
   context returns an invocation error. Provider/enumeration failures after
   report construction begins are findings in an explicit incomplete
   report.
6. The caller may invoke validation again because it is read-only. Each
   invocation re-reads provider/repository state; identical reports are not
   guaranteed if that state changes. Provider failures MUST NOT be hidden
   by an automatic retry or empty success-shaped result.
   Repeated invocations with unchanged inputs and underlying state are
   observationally equivalent and need no operation identifier.

### Machine-readable report

7. A completed invocation returns a report, not a pass/fail Boolean. The
   report contains:
   - metadata integrity: `valid`, `invalid`, or `indeterminate`;
   - history completeness: `complete`, `declared_incomplete`, `unresolved`,
     or `not_assessed`;
   - Resource state for each assessed Resource: `available`,
     `unavailable_or_not_locally_materialised`, `corrupt`, or `not_checked`,
     with verification method/strength when checked;
   - coverage by requested scope, object enumeration, and required root
     provider, including `complete`, `partial`, or `unavailable` status and
     any unavailable capability; and
   - typed findings for metadata integrity failure, schema/admission
     failure, identity/Project/reference mismatch, cycles, unresolved
     metadata, declared boundaries, Resource unavailability/corruption,
     incomplete coverage, and provider/enumeration failure.
8. `valid` means all required metadata checks within sufficient
   requested-scope coverage passed and required targets resolved. `invalid`
   means an actually checked metadata integrity, schema, identity, or graph
   invariant failed. Missing targets, declared omissions, unavailable
   schemas/providers, unrequested Resource checks, or incomplete coverage
   alone MUST NOT be reported as `invalid`. When absence prevents an
   admission check, the referring object remains unadmitted and metadata
   integrity is `indeterminate`, not `invalid` solely due to that absence.
   `indeterminate` means required metadata or provider coverage prevents a
   conclusive integrity determination. A declaration does not validate an
   absent target.
9. Completeness is `complete` only when coverage is sufficient for the
   requested scope, all required references within that scope resolve, and
   no applicable declared boundary exists. A known undeclared missing
   reference yields `unresolved`; otherwise a known declared omission yields
   `declared_incomplete`; when neither is known but providers cannot
   meaningfully assess the requested scope, the result is `not_assessed`.
   A partial scope/provider result MUST NOT imply complete history or global
   unreachable classification.
10. Resource verification is separate from metadata integrity and history
   completeness:
   - `metadata_only` does not inspect Resource bytes;
   - `verify_available_resources` verifies bytes already available through
     the supplied verification boundary and reports the method/strength;
   - `deep_resources` requires full-content Resource hash verification for
     bytes obtainable without automatic fetch or materialisation.
   Unavailable bytes are not corruption. No mode initiates an automatic
   fetch.

### Provider and operation failures

11. Core uses provider-neutral boundaries to enumerate objects/roots,
   query declared history boundaries by the exact
   `(referring object Identifier, edge kind, target Identifier)` tuple, and
   request Resource verification. A boundary lookup returns a matching
   declaration, no matching declaration, or an explicit provider failure.
   A provider failure MUST NOT be converted to an empty result or to “no
   declaration”.
12. A provider/enumeration failure after report construction begins is a
   typed finding with partial/unavailable coverage; the report remains an
   explicit incomplete result. An invalid invocation or failure before a
   report can be constructed returns an invocation error and MUST NOT be
   presented as a completed validation report.
13. A report with negative findings means the validation invocation ran; it
    does not mean the repository passed. An invocation error means the
    requested validation could not be performed.

### Existing boundaries

14. The report MUST enumerate required root/provider coverage from Core §62.
    Where a provider is not yet implemented or its semantics remain open,
    the result reports partial or unavailable coverage; it MUST NOT invent
    roots or claim full Core §62 reachability. The WORK-0013 Line/Release
    traversal may be consumed only with its explicitly partial scope.
15. Validation does not select Resource-retention or Line-deletion pin
    policy, Contribution semantics, Working State safety-reference roots,
    or temporary-checkpoint behavior. DEC-CORE-005, DEC-CORE-008,
    DG-0015, DG-0027, and DEC-INTERACTION-004 remain separate.
16. Existing historical-object admission rules remain unchanged. A declared
    boundary classifies an absent edge for completeness reporting but does
    not admit a referring object whose required target is unresolved.

## Rationale

An explicit report makes partial assessment honest and distinguishes
negative findings from inability to run validation. Read-only operation
preserves the diagnostic boundary. Reusing provider-neutral contracts keeps
Core independent of persistence and Storage Adapter mechanics while making
M2 behavior testable.

## Alternatives considered

- Allow validation to repair or update operational state. Rejected: a
  diagnostic operation must not hide or change the condition it reports.
- Treat every incomplete or unavailable portion as invalid. Rejected:
  incompleteness and Resource unavailability are not metadata corruption.
- Return success-shaped empty results when a provider fails. Rejected:
  provider failure and incomplete coverage must be explicit.
- Claim full validation from the currently implemented Line/Release roots.
  Rejected: Core §62 has additional root classes.

## Specification impact

- `Specs/OMVCS Core Specification.md` §§4, 10, 13–14, 18, 21–22, 29–30,
  47–56, 62–67, 76–77, 82–83, and 89.
- `Specs/OMVCS Core Invariants Specification.md` INV-INT-003, INV-INT-005,
  INV-HIST-003, INV-HIST-008–009, INV-RES-007, INV-WORK-002,
  INV-REC-002–006, and INV-GC-001–003.
- `Specs/OMVCS Glossary.md`, Content-derived Identifier, Repository
  Metadata, Availability State,
  history-completeness, validation-report, coverage, and verification-depth
  terms.

## Test impact

WORK-0014 must test read-only behavior, both scopes and all depths,
result-state separation, typed findings, exact boundary lookup, provider
failures, unavailable root classes, no false completeness/global
unreachability, no implicit fetch/materialisation, and invocation-error
versus completed-report behavior. The later Working State safety-reference
coverage is defined by ADR-0030; the treatment of a missing safety-reference
target against a declared history boundary remains open under DG-0028.

## Implementation impact

WORK-0014 (M2) may implement an in-memory/reference provider boundary,
separate body-derived Identifier verification from admission in the model
boundary, and produce a partial report using WORK-0013. Durable persistence
and full root-provider coverage remain outside this package.

## Compatibility / migration impact

This adds a logical Core operation contract and operational reporting
metadata only. It does not change historical schemas, hashes, identifiers,
Resource identity, storage-provider mechanics, or durable metadata format.

## Notes

No general error-precedence framework is established.

ADR-0030 extends the applicable §62 root coverage without changing this
validation operation's report contract. Repository-wide reachability remains
partial while later root classes are unsupported.

ADR-0031 specifies that missing Working State safety-reference root targets
are `unresolved` and do not use declared-boundary lookup.
