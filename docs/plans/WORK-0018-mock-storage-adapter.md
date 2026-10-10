# WORK-0018 — Mock Storage Adapter

Status: PLANNED
Owner agent: Storage Engineer
Milestone: M3
Branch: `work/0018-mock-storage-adapter`
Required review: Core Engineer + Verifier

## Objective

Implement a deterministic in-memory Storage Adapter conforming to the
approved generic contract and verification/Replica interfaces, to provide
shared conformance fixtures and controlled provider failures.

## Normative requirements

- Storage Adapter Specification §§1–21, 22–38, 85–86, 176–192, and
  applicable conformance sections §§199–205.
- Core Specification §§8, 29–36, and 47–55.
- Core Invariants: INV-RES-001–007, INV-STOR-001–005, INV-INT-001–003.

## Dependencies

- WORK-0015, WORK-0016, and WORK-0017.
- ADR-0032/0033 Replica identity and locator contracts reflected in
  approved interfaces; ADR-0034 Storage Map generation/CAS contract
  reflected in guarded mutation fixtures.
- DEC-STORAGE-013 is not required to develop the test Adapter and must not
  be interpreted as official reference-implementation designation.

## Allowed scope

- `crates/omvcs-storage-mock/`
- Shared, provider-neutral conformance fixtures where approved by Verifier.

## Deliverables

- In-memory Adapter implementing only the approved logical operations.
- Deterministic controls for absent, unavailable, failed, truncated,
  wrong-identity, and corrupt data conditions.
- Reusable conformance fixture support without encoding provider-specific
  behavior in Core.

## Acceptance tests

- Run the approved contract/verification conformance suite unchanged.
- Same configured input and injected-failure sequence yields deterministic
  results.
- All injected failures are explicit; no failure becomes empty success.
- Partial/corrupt data cannot be registered as a valid Replica.
- Mutation operations preserve their specified failure atomicity.
- No unapproved delete, retention, namespace-sharing, grant, or encryption
  behavior is added to the mock.

## Explicit non-goals

- Filesystem, network, cloud, or persistence behavior.
- Choosing verification/chunking semantics; the mock implements the
  contracts approved by ADR-0036/0037/0038.
- Official reference-conformance claims.
- GC, cleanup, retention, M4 transactions, or Platform policy.

## Known Design Gaps

- DG-0032 is resolved by ADR-0034; guarded Storage Map mutation fixtures
  must assert the Project-wide atomic generation contract.
- Temporary grants remain optional under Storage §§101–102/190; public
  playback decisions do not block this package.

## Implementation plan

1. Implement the approved generic interfaces using isolated in-memory
   state.
2. Add deterministic failure injection.
3. Run all applicable conformance tests.
4. Obtain Core Engineer review and independent Verifier acceptance.

## Verification requirements

Verifier must demonstrate each injected failure is observable, deterministic,
and unable to produce false successful verification or registration.

## Completion criteria

Relevant workspace and package tests, rustfmt, warnings-denied Clippy,
coverage update, handover, and clean diff checks pass.
