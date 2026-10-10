# WORK-0015 — Storage primitives and provider-neutral contract

Status: ACCEPTED — integration pending
Owner agent: Storage Engineer
Milestone: M3
Branch: `work/0015-storage-primitives-contract`
Required review: Core Engineer ACCEPT; independent Verifier ACCEPT

## Objective

Define and implement the smallest provider-neutral Storage Adapter seam for
byte-preserving Resource/Chunk put, get, existence/stat, Endpoint identity,
capability reporting, and explicit provider failures.

This first package intentionally does not establish Replica registration,
Storage Map persistence, verification-strength classifications, or
Repository Home conformance.

## Normative requirements

- Core Specification §§6–8, 29–31.
- Storage Adapter Specification §§1–21, 22–29, 35–36, 85–86, and
  176–182, 185, 190.
- Glossary: Resource Object, Resource Identifier, Chunk, Chunk Identifier,
  Storage Adapter, Storage Endpoint, and Operational Metadata.
- Core Invariants: INV-RES-001–007, INV-STOR-001–003, INV-INT-001–003.

## Dependencies

- Existing M1 Resource/Chunk identifiers and current Storage Adapter
  operation contract.
- No human decision is required for the restricted byte-I/O seam.
- Do not choose Chunking policy; DEC-CORE-001 and DEC-STORAGE-002 remain
  coupled and open.
- No persistent Replica or Storage Map model; DG-0029 is deferred to
  WORK-0016.
- DEC-STORAGE-003's Repository Home minimum is not claimed or implemented.

## Allowed scope

- `crates/omvcs-storage/`
- Focused contract tests and test doubles for that crate.
- Any cross-boundary changes must be agreed with Core Engineer before
  implementation.

## Deliverables

- Typed, provider-neutral Endpoint identity and capability description.
- Resource/Chunk byte-stream put/get and existence/stat operation contracts,
  with structured outcomes and provider failures.
- Explicit byte-preservation and caller-supplied identity boundaries.
- Contract tests against minimal fake providers.

## Acceptance tests

- Put/get round trips preserve exact input bytes for Resource and Chunk
  identities; a fake provider returning different bytes is not success.
- Existence/stat distinguish absent from provider failure and do not claim
  integrity based on existence or length alone.
- Capability reporting is truthful; an unsupported operation is explicit,
  not represented as a success-shaped empty result.
- Provider errors remain typed and distinguishable from ordinary missing
  content according to Storage §§85–86 and the operation contracts.
- No Resource identity is derived from Endpoint IDs, paths, keys, timestamps,
  or provider-generated identifiers.
- Byte-I/O calls do not create or mutate historical metadata or assert
  global reachability.

## Explicit non-goals

- Chunking algorithm or automatic rechunking.
- Replica identity, Replica registration/removal, or Storage Map persistence.
- Verification-strength taxonomy, upload promotion, or publication policy.
- DeleteReplica, GC, retention, orphan cleanup, or source removal.
- Repository Home conformance/bootstrap, layout keys, or marker/discovery.
- Encryption, Platform authorization, temporary grants, FFI, or DAW behavior.
- Concrete Mock or filesystem Adapter implementation.
- M4 operation log, publication transaction, or recovery orchestration.

## Known Design Gaps

- DG-0029 blocks persistent Replica/Storage Map semantics in WORK-0016, not
  this byte-I/O contract.
- DEC-CORE-001 and DEC-STORAGE-002 remain open; no chunking policy is chosen.

## Implementation plan

1. Map current normative operation inputs, outcomes, and capability rules to
   typed Rust interfaces without adding semantic fields.
2. Implement the restricted byte-I/O contract and structured failures.
3. Add contract tests using a minimal fake provider.
4. Obtain Core Engineer review, then independent Verifier review.

Implementation and both reviews are complete on the feature branch. The
implementation commit is `1be170e`; integration is pending.

## Verification requirements

Verifier must compare each exposed operation and failure distinction with
the cited clauses, attempt identity substitution and provider-failure
cases, and confirm that no physical location crosses into historical
identity.

## Completion criteria

Relevant workspace and crate tests pass; rustfmt and warnings-denied
Clippy pass; `docs/spec-coverage.md` is updated; handover is completed;
`git diff --check` is clean; independent Verifier accepts; no unresolved
requirement has been silently implemented. Integration remains a separate
step.
