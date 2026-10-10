# DG-0034 — ProviderLocator schema identifier grammar

Status: OPEN
Classification: BLOCKS-FEATURE
Discovered by: Storage Engineer and Spec Guardian
Discovered during: WORK-0016 Storage Engineer review
Date: 2026-10-10

## Relevant specifications

- `Specs/OMVCS Storage Adapter Specification.md` §73, ProviderLocator.
- `Specs/OMVCS Core Specification.md` §76, Schema versioning.
- ADR-0033, Decisions 1–2 and its test impact.
- `docs/plans/WORK-0016-replica-storage-map.md`.

## Problem

ProviderLocator requires a schema identifier using the "existing
versioned schema-identifier convention" and identifying the exact provider
locator schema and version. The text provides examples such as
`example.storage.chunk-locator/1`, but does not define a lexical grammar.
It is therefore unclear whether a generic Core parser must reject values
such as a non-numeric version suffix, multiple slash-delimited components,
or other syntactically unusual but non-empty strings.

## Why the current specifications are insufficient

Core §76 and Storage Adapter §73 require exact schema/version association
but do not define allowed characters, delimiters, version-token syntax, or
whether version components are opaque to generic Core. Examples do not
establish a complete grammar. Rejecting a string such as
`provider.locator/not-a-version`, or requiring exactly one slash or a
numeric suffix, would select syntax not established by the Specs.

## Affected work

- WORK-0016: ProviderLocator schema-identifier validation and its negative
  conformance tests.
- Future Storage Adapter implementations and conformance tests that create
  or consume ProviderLocator envelopes.

## Can unaffected work continue?

Yes. Replica identity/representation, locator-value canonical JSON,
Storage Map reconstruction under ADR-0035, and Storage Map generation/CAS
are unaffected. The basic opaque envelope can be implemented without
inventing provider-specific value semantics. WORK-0016 cannot claim complete
lexical validation of the versioned schema identifier until this gap is
resolved.

## Candidate directions

The following are discussion material only and are NOT approved:

- define a constrained dotted numeric version suffix;
- define a general opaque non-empty version token separated from a schema
  name by a specified delimiter;
- reuse a separately standardized schema identifier grammar, if one is
  selected and precisely referenced.

## Required decision

Specify the lexical grammar for ProviderLocator schema identifiers,
including how the schema name and exact version are separated, which
characters/components are valid, and whether generic Core or the
Storage Adapter/provider validates each part.

## Resolution

UNRESOLVED
