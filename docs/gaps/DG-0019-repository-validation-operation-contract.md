# DG-0019 — Repository validation operation contract

Status: RESOLVED
Classification: BLOCKS-FEATURE
Discovered by: Spec Guardian
Discovered during: M2 preflight
Date: 2026-10-09

## Relevant specifications

- `Specs/OMVCS Core Specification.md` §§21–22, 29, 47–56, 62–67, 76–77,
  and 82–83.
- `Specs/OMVCS Core Invariants Specification.md`, INV-HIST-003,
  INV-HIST-009, INV-RES-007, INV-WORK-002–004, INV-INT-001–003, and
  INV-GC-001–003.
- `Specs/OMVCS Glossary.md`, OMVCS Repository, Repository Metadata,
  Historical Metadata, Operational Metadata, Resource Availability,
  Corrupt Replica, Reachability, and Repository Recovery.
- Existing `DEC-CORE-004` (local metadata-history completeness),
  `DEC-CORE-005` (unreachable Resource retention),
  `DEC-CORE-008` (Line-deletion pin), and `DEC-CORE-009`
  (shallow/incomplete imports) in `docs/decision-register.md`.

## Problem

Core §56 gives a substantial list of repository-validation checks, makes a
full Project validation optional as a repository-wide operation, and
requires the listed Revision and Project State admission checks whenever
those objects are admitted or assigned valid identities. It also states
that metadata validation need not download every historical Resource unless
deep Resource verification is requested. The Specs distinguish resolvable,
admitted metadata from locally materialised Resource bytes; missing Resource
bytes do not erase history or by themselves invalidate metadata. Available
Resource bytes must be verifiable, and corrupted replicas are distinct from
other valid replicas.

The named `ValidateRepository` operation in Core §82 has no operation
contract under §83. The Specs do not say whether validation is strictly
read-only (including whether it may update operational availability or
verification records), how callers request full versus partial checks, or
how its result reports valid, invalid, unresolved, unavailable, and
incomplete portions without conflating metadata corruption, missing
metadata, and unavailable Resource bytes.

Whether local metadata history must be complete is already DEC-CORE-004;
handling explicitly shallow/incomplete imports is DEC-CORE-009. This gap
does not choose either policy or duplicate those questions. It also does not
set a Resource-retention interval or deletion policy (DEC-CORE-005 and
DEC-CORE-008).

## Why the current specifications are insufficient

The validation checklist identifies checks but not the operation's effects
or result contract. Implementing implicit repair, changing repository
records during a diagnostic, or treating missing bytes as missing history
would each add behavior not selected by the Specs. Conversely, forcing every
validation run to fetch or require all Resource bytes would conflict with
the specified metadata-complete/resource-sparse distinction. Choosing how
to classify metadata omissions without the decisions under DEC-CORE-004 and
DEC-CORE-009 would duplicate or pre-empt those decisions.

## Affected work

- M2 `ValidateRepository` operation, diagnostic result/status model, and
  repository-wide validation tests.
- M2 distinction among invalid/corrupt metadata, unresolved or incomplete
  metadata history, and absent/unavailable/corrupt Resource data.
- Later repository recovery, import, and operational verification work
  consumes the resulting contract.

## Can unaffected work continue?

Yes. Implementers may validate each admitted historical object's canonical
identity, exact schema, required metadata references, same-Project Revision
parents, and Project State membership consistency as already required.
They may verify Resource bytes when available and report byte unavailability
without invalidating otherwise valid historical metadata. Graph traversal
from defined roots follows the specified references, including Revision
parent ancestry; it does not itself authorize deletion. Do not implement an
implicit repair/mutation policy or claim repository-wide completeness for
shallow/incomplete imports until DEC-CORE-004 and DEC-CORE-009 are resolved.

## Candidate directions

None recorded. A read-only diagnostic contract and its result states require
human approval.

## Required decision

Specify whether `ValidateRepository` is strictly read-only and identify any
permitted operational side effects; define its full/partial invocation scope
and machine-readable outcomes for integrity failure, unresolved metadata,
permitted incompleteness, and Resource availability/verification. Preserve
the distinct choices in DEC-CORE-004, DEC-CORE-005, DEC-CORE-008, and
DEC-CORE-009; resolve those through their existing decision entries rather
than this gap.

## Resolution

Resolved by human-approved
[ADR-0029](../decisions/ADR-0029-validate-repository-operation-contract.md),
with the distinct local-completeness and shallow-import decisions recorded
in [ADR-0027](../decisions/ADR-0027-local-metadata-history-completeness.md)
and
[ADR-0028](../decisions/ADR-0028-declared-shallow-history-boundaries.md).
The Core contract defines a strictly read-only operation, explicit
Repository/Project scope and Resource-verification depth, machine-readable
integrity/completeness/Resource/coverage dimensions, typed findings, and a
distinction between a completed report and inability to invoke validation.
Provider errors are reported explicitly with partial/unavailable coverage.

Cross-Spec requirements to store complete history in every local Repository
and to handle shallow parents without a matching declaration are reconciled
in Core §§14, 29–30, 55–56, and 66–67. Existing admission rules remain
unchanged: a declaration does not admit an object whose required target is
absent. Such data may be preserved outside admitted history where
supported.

WORK-0014 is PLANNED, not started. It may implement partial coverage using
the current Line/Release reachability boundary, but MUST report unsupported
Core §62 roots and MUST NOT claim complete history or global unreachable
status. DEC-CORE-005, DEC-CORE-008, DG-0015, DG-0027, and
DEC-INTERACTION-004 remain separate and unresolved.
