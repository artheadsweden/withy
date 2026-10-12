# Handover WORK-0020 — Resource replication preflight

From agent: OMVCS Lead
To agent: Core Engineer, Storage Engineer, independent Verifier
Date: 2026-10-12
Branch: `work/0020-resource-replication`
HEAD at preflight start: `2d831b929dd81f6ceed8f8ae5fd4f0653f570d5b`

## Completed

Verified the initial integration branch was clean at the expected HEAD,
created the WORK-0020 branch, read the work package and applicable Specs/ADRs,
and searched the full Spec set for replication, Replica, locator, key,
verification, and outcome terms. The Spec Guardian confirmed that an
Adapter-level locator interpretation path is authorized, but the Specs do
not define the composed per-Resource replication outcome contract.

Recorded DG-0037 as `BLOCKS-FEATURE`, changed WORK-0020 to BLOCKED, corrected
the stale WORK-0017 dependency/status wording, and updated the M3 status and
coverage records. Core Engineer confirmed no production Core work can safely
proceed before resolution; Storage Engineer confirmed the locator path is
Adapter-owned and no premature Storage change is needed. Independent
Verifier ACCEPTED the gap and status documentation with no remaining findings.
No production implementation was started.

## Specifications implemented

None. This is a blocked preflight, not an implementation handover.

Normative requirements reviewed:

- Core Specification §§32–36, 47–55, and 83.
- Storage Adapter Specification §§14–29, 30–36, 54–59, 73–76, 82,
  85–86, 109–112, 181–183, 204–205.
- Core Invariants INV-RES-001–007, INV-STOR-003–005, INV-INT-001–004,
  and INV-GC-001–003.
- ADR-0032/0033/0034/0036/0037/0038.

## Files changed

- `docs/gaps/DG-0037-replication-outcome-contract.md`
- `docs/decision-register.md`
- `docs/plans/WORK-0020-resource-replication.md`
- `docs/plans/WORK-0021-storage-migration.md` (stale dependency status only)
- `docs/plans/M3-STORAGE-PREFLIGHT.md`
- `docs/spec-coverage.md`
- `docs/milestones.md`
- `docs/project-state.md`
- `docs/handovers/HANDOVER-WORK-0020-preflight.md`

## Tests added or changed

None. Production code and test files are unchanged.

## Commands run

- `git status`, `git branch --show-current`, `git rev-parse HEAD`,
  `git remote -v` — initial state matched the expected clean integration
  branch at `2d831b929dd81f6ceed8f8ae5fd4f0653f570d5b`; `origin` is
  `https://github.com/artheadsweden/withy.git`.
- `git switch -c work/0020-resource-replication` — passed.
- Repeated repository-state commands after branch creation — clean branch
  at the expected HEAD; no test/build/format commands run because no code
  changed.

## Semantic decisions made beyond the specification

`None`.

## Design Gaps discovered

- DG-0037 — OPEN, BLOCKS-FEATURE: per-Resource replication outcomes,
  partial-effect reporting, and ordinary retry/idempotency contract.
- DG-0034 remains OPEN for generic ProviderLocator schema-identifier
  lexical validation only; it does not block the opaque locator envelope.
- DG-0036 remains OPEN for non-filesystem Repository Home discovery only;
  it is outside WORK-0020.
- DEC-STORAGE-011 remains OPEN; shared namespace claims remain excluded.
- DEC-STORAGE-013 remains OPEN; no official reference designation is made.

## Assumptions

None.

## Known limitations

WORK-0020 implementation, transfer orchestration, and acceptance tests are
blocked until the DG-0037 human decision is recorded in an ADR and the
affected Specs/work package are updated. The decision must cover ordinary
retry/idempotency behavior while keeping M4 durable cross-step recovery out
of scope. The Storage Adapter may interpret
ProviderLocator in the Endpoint context; Core must not parse the locator or
treat it as a logical key. Existing ByteStorage, verification, and CAS APIs
remain usable only within their currently specified contracts.

## Remaining work

1. Resolve DG-0037 through a human decision, ADR, and affected Spec update.
2. Revisit WORK-0020 and define the implementation/result contract strictly
   from the resolved Specs.
3. Implement and test WORK-0020, then obtain Storage Engineer review and
   independent Verifier acceptance.
4. Keep WORK-0021 unstarted until its own prerequisites and gate are met.

## Git state

Working tree: CLEAN after documentation-only commit
Remote publishing enabled: YES
Remote push performed: NO
Integration occurred: NO
