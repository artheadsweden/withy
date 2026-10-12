# DG-0037 — Per-resource replication outcome contract

Status: RESOLVED by ADR-0043
Classification: BLOCKS-FEATURE
Discovered by: Spec Guardian
Discovered during: WORK-0020 preflight
Date: 2026-10-12

## Relevant specifications

- `Specs/OMVCS Core Specification.md` §§32–35, 47–51, 55, and 83.
- `Specs/OMVCS Storage Adapter Specification.md` §§19–29, 54–59, 73,
  82, 85–86, and 181–183.
- `Specs/OMVCS Core Invariants Specification.md`, INV-RES-005,
  INV-STOR-005, and INV-INT-003–004.
- `Specs/OMVCS Glossary.md`, Resource Replica, Storage Migration, Replica
  Addition, Content Verification, and Verification Result.
- ADR-0032, ADR-0033, ADR-0034, ADR-0036, ADR-0037, and ADR-0038.

## Problem

The specifications define the component contracts used by replication:
source retrieval and destination writes have structured Adapter failures;
verification distinguishes `verified`, `failed`, and `indeterminate`;
Replica registration requires destination-applicable `resource_identity`;
Storage Map CAS reports typed conflicts and persistence failures; and
replication does not remove its source.

They do not define the Core-visible per-Resource result for the composed
replication operation when retrieval, transfer, verification, or registration
fails or is interrupted. In particular, they do not define which operation
outcome states are required or what the result may assert about partial
destination bytes and whether Storage Map registration committed.

Core §83 requires every implementation-facing Core operation contract to
specify its inputs, preconditions, effects, failure modes, retry and
idempotency behavior, and result. The existing replication definition and
stage-local contracts do not supply that composed operation contract.

## Why the current specifications are insufficient

The operation-specific contracts report local outcomes, but they do not
compose those outcomes into one per-Resource replication result. For example,
a failed or interrupted write can leave provider-side partial bytes, while a
successful write is not itself verification; verification can be
indeterminate when destination bytes are unavailable; and registration can
conflict or fail after destination assurance succeeds. The current contracts
do not state which result is required for successful replication, how a
partial or failed attempt must be represented, or which observed effects it
may claim.

WORK-0020 acceptance criteria require an explicit result distinguishing
success, partial/incomplete completion, and failure stage. Those criteria
identify the need but do not define the result's normative meaning or
partial-effect guarantees. Core §83 also requires retry and idempotency
behavior to be specified; Storage Adapter §82 distinguishes safe
transport-level retries from repository-semantic retries, and INV-INT-004
recommends idempotency for storage-transfer operations where practical.
The current contracts do not define these behaviors for the composed
replication operation. The contract must preserve the existing requirements:
no success-shaped partial result, no registration without destination
assurance and successful CAS, and no source removal. WORK-0020 excludes
M4's durable cross-step retry/recovery orchestration, but that does not
settle the operation's ordinary retry or idempotency behavior.

## Affected work

- At discovery, WORK-0020 Resource replication and its Core result API,
  storage transfer integration, and acceptance tests were blocked by this
  gap. ADR-0043 resolves the outcome contract; WORK-0020 remains blocked by
  DG-0038's separate Core §69 resumability boundary.
- WORK-0021 remains unstarted and retains its own work-package gate.

## Can unaffected work continue?

Yes. Existing byte-I/O, Replica, verification, and Storage Map CAS contracts
remain usable within their specified boundaries. They do not authorize
implementing or claiming completion of the WORK-0020 replication outcome
contract. No production change is made under this gap.

## Candidate directions

None recorded. No replication result states or partial-effect guarantees
are selected here.

## Required decision

Define the required Core-visible per-Resource replication outcomes for
success, partial completion, provider failure, verification failure, and
registration failure at the source-retrieval, destination-transfer,
destination-verification, and Storage Map registration stages. Specify what
a successful result may assert about destination assurance and committed
registration, and which observed partial destination effects and registration
state a failed or interrupted result may assert, including when the provider
outcome does not establish whether destination bytes remain. Also define the
operation's retry and idempotency behavior, distinguishing any permitted
Adapter transport retry from Core repository-semantic retry behavior, while
keeping M4's durable cross-step recovery orchestration outside WORK-0020.

## Resolution

Resolved by human-approved ADR-0043, which defines the Core per-Resource
outcome, stage, destination-effect, registration-state, source-preservation,
and ordinary retry/idempotency contract. The affected Specs and WORK-0020
test plan are updated. ADR-0044 subsequently resolved DG-0038's separate
Core §69 applicability question without altering ADR-0043.
