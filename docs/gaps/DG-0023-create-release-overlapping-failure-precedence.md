# DG-0023 — CreateRelease overlapping failure precedence

Status: RESOLVED
Classification: BLOCKS-FEATURE
Discovered by: Core Engineer, with Spec Guardian analysis
Discovered during: WORK-0011
Date: 2026-10-09

## Relevant specifications

- `Specs/OMVCS Core Specification.md` §18, “CreateRelease”.
- `Specs/OMVCS Core Specification.md` §83, “Operation contract rule”.
- `Specs/OMVCS Core Specification.md` §56, “Repository validation”.
- `docs/decisions/ADR-0017-release-object-and-admission-contract.md`,
  Decision 7.
- `docs/decisions/ADR-0023-create-release-failure-precedence.md`.

## Problem

The Release contract separately classifies an existing `ReleaseId` resolving
to different canonical body bytes as an integrity violation and a
Project/name binding to a different `ReleaseId` as a name conflict. It did not
specify which result `CreateRelease` returns when both conditions apply to one
request.

## Why the current specifications were insufficient

The individual failure categories and exact-duplicate conditions were stated,
but ADR-0017 and the prior Core §18 text did not specify the ordering or
combination rule for overlapping failures. Core §83 requires operation
failure modes and results to be specified.

The human explicitly approved the narrow overlap decision recorded in
ADR-0023 on 2026-10-09.

## Affected work

- WORK-0011 `CreateRelease` operation result and conformance tests.
- `crates/omvcs-core/src/release.rs`.

## Can unaffected work continue?

Yes. Release representation, canonical identity, admission, exact-duplicate
behavior, isolated name-conflict behavior, and same-Project target checks are
unaffected.

## Decision recorded

When one `CreateRelease` request simultaneously encounters an existing
same-ReleaseId/different-canonical-body integrity violation and a
Project/name binding conflict, Core MUST return the integrity violation.
The name conflict MUST NOT mask it. For this overlap, failure MUST remain
atomic and MUST leave all stored objects and Project/name bindings unchanged.

This decision is limited to precedence between these two conditions. It does
not define other precondition ordering, retry behavior, or new identity/name
semantics, or a general failure-precedence framework.

## Required decision

None for this overlap: the human-approved decision is recorded in accepted
ADR-0023. It does not establish a broader rule for other Core operations.

## Resolution

Resolved by human-approved
[ADR-0023](../decisions/ADR-0023-create-release-failure-precedence.md) and
the Core §18 update. The former BLOCKS-FEATURE gap no longer blocks
WORK-0011.
