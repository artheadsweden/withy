# WORK-0015 — Storage primitives and provider-neutral contract

Status: VERIFIED — integrated
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
- Do not choose Chunking policy; at WORK-0015 implementation time
  DEC-CORE-001 and DEC-STORAGE-002 were coupled and open. They are
  subsequently resolved by ADR-0036; WORK-0015 does not implement the
  chunker.
- No persistent Replica or Storage Map model; DG-0029 was deferred to
  WORK-0016 and has since been resolved by ADR-0032. Storage Map generation
  and guarded mutation mechanics are defined separately by ADR-0034 and
  remain outside WORK-0015.
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

- At WORK-0015 closeout, DG-0029 blocked persistent Replica/Storage Map
  semantics, not this byte-I/O contract. ADR-0032 later resolved DG-0029;
  DG-0032 now gates guarded map mutations.
- At WORK-0015 closeout, DEC-CORE-001 and DEC-STORAGE-002 were open and no
  chunking policy was chosen in that package. ADR-0036 subsequently resolves
  both decisions.

## Implementation plan

1. Map current normative operation inputs, outcomes, and capability rules to
   typed Rust interfaces without adding semantic fields.
2. Implement the restricted byte-I/O contract and structured failures.
3. Add contract tests using a minimal fake provider.
4. Obtain Core Engineer review, then independent Verifier review.

Implementation and both reviews are complete. Implementation commit:
`1be170e4cc3afd8e825580d41129847870f359f7`; documentation commit:
`74474db9d3007ffa0ee54623ba9e1d4d4d719f5d`. Integrated on
`spec/0003-canonical-collection-order` by merge commit
`aade8af1031a42fd9c7d093f35f6c4ae3f2818a0`.

## Verification requirements

Verifier must compare each exposed operation and failure distinction with
the cited clauses, attempt identity substitution and provider-failure
cases, and confirm that no physical location crosses into historical
identity.

## Completion criteria

Relevant workspace and crate tests pass; rustfmt and warnings-denied
Clippy pass; `docs/spec-coverage.md` is updated; handover is completed;
`git diff --check` is clean; independent Verifier accepts; no unresolved
requirement has been silently implemented. Integration completed at
`aade8af1031a42fd9c7d093f35f6c4ae3f2818a0`.
