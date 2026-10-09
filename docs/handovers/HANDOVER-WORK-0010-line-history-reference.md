# Handover WORK-0010

From agent: OMVCS Core Engineer
To agent: OMVCS Lead
Date: 2026-10-09
Branch: `work/0010-lines`
Implementation commit: `01d61fdabc0da58e28bff65b1fd33907da6b2d0a`

## Completed

Implemented the OMVCS 0.1 closed Line record, typed assigned `LineId`,
generation validation, and Core operation boundary for CreateLine, MoveLine,
RenameLine, SetDefaultLine, and DeleteLine. The mutex-backed reference
repository makes every mutation atomic and is explicitly not durable storage.
The Core contract requires a durable Repository Home implementation to
persist the shared Default Line there; WORK-0010 does not implement or claim
that later repository/storage adapter responsibility.

No WORK-0011–0014 work was started. No push, branch switch, merge, Platform
transport, publication flow, retention, pinning, reachability, or GC
implementation was performed.

## Specifications implemented

- Core Specification §§3.2, 4.1–4.2, 16–17, 26, 56–58, 74, and 82–83.
- Glossary: Line, Default Line, and Operational Metadata.
- Core Invariants: INV-HIST-005, INV-HIST-010, INV-HIST-011, INV-GC-001,
  and INV-UX-002.
- Accepted ADR-0016, ADR-0020, ADR-0021, and ADR-0022.
- Existing WORK-0008 admitted Revision and WORK-0009 typed graph/resolver
  contracts.

## Files changed

- `crates/omvcs-model/src/lib.rs`: added `LineId` to the assigned UUIDv7
  identifier profile.
- `crates/omvcs-core/src/line.rs`: Line and LineGeneration records,
  serialization, atomic operation contract, and mutex-backed reference
  repository.
- `crates/omvcs-core/src/lib.rs`: exports `line`.
- `crates/omvcs-core/tests/line_operations.rs`: focused Line conformance tests.
- `crates/omvcs-core/Cargo.toml` and `Cargo.lock`: direct serde dependency for
  the Line record serialization/deserialization contract; no new package
  version was introduced.
- `docs/spec-coverage.md`: updated only the WORK-0010 row with verified
  implementation evidence after independent acceptance.
- `docs/handovers/HANDOVER-WORK-0010-line-history-reference.md`: this handover.
- `docs/plans/WORK-0010-line-history-reference.md`: the IN PROGRESS branch/
  scope update was already present at task start and was not made as part of
  implementation.

## Tests added or changed

- `tests/line_operations.rs`: 23 tests covering the exact five-member closed
  record, UUIDv7 typed identity, name equality/uniqueness, admitted same-Project
  target validation without Resource-byte lookup, generation domain and JCS
  canonicalization, failed-CreateLine atomicity, successful and stale
  Move/Rename CAS, failure preservation, Default Line optionality/CAS/
  clearing/local-selection separation, deletion outcomes/history
  preservation, concurrent uniqueness, atomic SetDefaultLine/DeleteLine
  exclusion, and explicit Default Line change/clear before deletion without
  fallback.
- `src/line.rs` unit tests: 2 tests proving MoveLine and RenameLine fail
  atomically at maximum generation and DeleteLine succeeds at maximum without
  incrementing or mutating the referenced Revision.
- Focused Line-specific total: 25 tests (23 integration + 2 unit).

## Commands run

- `cargo check --offline -p omvcs-core` — passed; synchronized the lockfile
  dependency edge for the already-locked serde package.
- `cargo fmt --all` — passed.
- `cargo fmt --all -- --check` — passed.
- `cargo test --locked -p omvcs-core` — passed: 4 unit tests, 23 Line
  integration tests, 7 Revision graph tests; 34 passed total.
- `cargo test --locked --workspace` — passed: 168 tests passed across the
  workspace, zero failures.
- `cargo clippy --locked -p omvcs-core -p omvcs-model --all-targets -- -D
  warnings` — passed. An initial run identified lints that were corrected
  before this final successful run.
- `git diff --check` — passed.

## Semantic decisions made beyond the specification

`None`.

## Design Gaps discovered

No new Design Gap. DG-0016, DG-0020, DG-0021, and DG-0022 are resolved by the
accepted ADRs. DG-0015, DG-0017, DG-0018, and DG-0019 remain open but are not
Line implementation blockers per WORK-0010. DEC-CORE-008 retention/pinning and
DEC-INTERACTION-001/003 presentation remain out of scope.

## Assumptions

- The reference repository constructor receives the set of Projects that
  already exist in its in-memory repository state. Project creation is outside
  WORK-0010.
- `InMemoryLineRepository` serializes all mutable operational state behind
  one mutex to demonstrate the indivisible operation boundary; this is a
  reference/test implementation, not a durable-storage mandate.
- Target validation uses only the approved admitted Revision and admitted
  Project State resolver boundaries. Resource-byte availability is neither
  queried nor required.
- The operation boundary does not define a hosted-service role model. Any
  higher-level Core orchestration with a configured authorization mechanism
  remains subject to Core §61.

## Known limitations

- No durable Repository Home persistence or M3/M4 storage adapter is
  implemented.
- Persisting the Default Line in durable Repository Home is an integration
  obligation for the later repository/storage implementation, not a semantic
  gap and not provided by this in-memory conformance reference.
- No Platform mirror, transport, or publication state machine is implemented.
- No retention/pinning, reachability, historical-object removal, or GC policy
  is implemented or inferred.
- Independent Verifier: accepted. The initial findings were resolved by
  clarifying M2-vs-storage scope, documenting Core operation contracts,
  adding failed-CreateLine atomicity evidence, and adding explicit
  Default-Line change/clear-before-delete coverage.

## Remaining work

- No WORK-0010 implementation or verification work remains.
- Durable Repository Home persistence belongs to later repository/storage
  work; WORK-0010 supplies the operation contract and atomic reference
  implementation only.
- Leave WORK-0011–0014 unstarted and later durability work to its assigned
  package.

## Git state

Working tree: CLEAN after the WORK-0010 implementation and handover commits.
No unrelated changes were identified.

Remote push performed: NO
Remote publishing enabled: YES (`docs/project-state.md`); publishing was not
attempted, as explicitly instructed.
