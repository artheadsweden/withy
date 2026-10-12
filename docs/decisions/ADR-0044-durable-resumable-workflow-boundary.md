# ADR-0044 — Durable resumable workflow boundary

Status: ACCEPTED
Date: 2026-10-12
Decision owner: Human
Related gap: DG-0038

## Context

Core §69 said that long-running operations MUST be resumable, which could
make elapsed time or transfer size appear to determine protocol semantics.
DG-0038 identified the resulting ambiguity for WORK-0020: one Resource
replication may take hours or transfer a very large object, while ADR-0043
defines its result/retry semantics and expressly left §69 applicability
unresolved.

The decision distinguishes a Core durable resumable workflow from an
operation that merely runs for a long time or uses an Adapter transport
resume capability.

## Decision

1. Core §69 durable resumability applies only to a workflow-scale operation
   whose normative contract explicitly designates it as a durable resumable
   workflow. Such a workflow spans independently recoverable steps or work
   units and defines durable continuation across Core process restart.
2. Runtime observations do not designate a durable resumable workflow.
   Resource size, Chunk count, bytes transferred, elapsed time, transfer
   rate, background execution, progress reporting, or operation identifiers
   MUST NOT switch durable resumability on or off. OMVCS 0.1 defines no
   byte-count, Chunk-count, or elapsed-time threshold for this purpose.
3. Bulk/multi-Resource replication is a workflow-scale operation subject to
   §69 when implemented. Its durable state may identify completed Resource
   work units, but this decision defines no state schema, retry policy, or
   orchestration contract for that future workflow.
4. The bounded WORK-0020 operation that replicates one Resource from a
   source Endpoint to a destination Endpoint is invocation-scoped. It has no
   required Core durable continuation record, persisted stage checkpoint,
   implicit transaction, operation log for continuation, automatic crash
   recovery, or cross-process continuation.
5. If Core remains alive and can return a result after a transport or other
   recoverable interruption, it returns the strongest result supported by
   observed facts under ADR-0043. No known destination effect may permit
   `failed`; possible or known destination residue requires `incomplete`;
   a verified but unregistered destination is
   `incomplete`/`verified_unregistered`; and confirmed verification plus
   committed registration is `success`.
6. If the Core process terminates before returning, no result for that
   invocation is guaranteed to survive restart and no result may be
   retroactively fabricated. A later caller action is a fresh ADR-0043
   invocation that re-observes the authoritative Storage Map and current
   provider/verification evidence.
7. A fresh invocation may reuse existing destination bytes or evidence only
   where the existing Storage and verification contracts establish their
   identity, applicability, and safety. It MUST NOT trust them merely
   because they probably came from an interrupted invocation.
8. Adapter transport/provider resume mechanisms remain permitted only where
   existing Adapter contracts allow them. They are implementation/storage
   capabilities, not Core durable-workflow guarantees. They MUST NOT weaken
   verification, imply replication success before ADR-0043 conditions hold,
   create hidden Replica registrations, or turn uncertain state into assumed
   success. This decision adds no provider-resume semantics.
9. ADR-0043 remains unchanged: Core does not automatically retry the
   composed operation after returned non-success; caller reinvocation is
   new; no global replay idempotency is promised; source preservation is
   unconditional; destination effects may remain; no silent cleanup occurs;
   and success requires confirmed destination verification and committed
   Replica registration.
10. This decision does not add durable replication transactions, persisted
    stage checkpoints, operation logs, process-restart continuation,
    background retry, automatic crash recovery, or durable bulk orchestration
    to WORK-0020. Those remain later transaction/recovery or bulk-operation
    work, including M4 where applicable.
11. This decision does not decide whether a separate migration operation is
    invocation-scoped or a designated durable resumable workflow. Existing
    operation-specific migration requirements remain unchanged.

## Rationale

Durability is a semantic property of an explicitly specified workflow, not
an emergent property of runtime duration. The distinction keeps one
Resource replication implementable without inventing M4 recovery state,
while preserving §69's direction for future bulk replication.

## Alternatives considered

- Enable resumability above a Resource-size threshold: rejected because the
  same operation would have different protocol semantics based on data size.
- Enable resumability after an elapsed-time threshold: rejected because
  runtime and implementation conditions would change semantics.
- Treat every streaming operation as durable: rejected because transport
  streaming/resume is distinct from Core cross-process continuation.
- Exempt all replication from §69: rejected because bulk/multi-Resource
  replication remains a designated workflow-scale case.
- Define the future bulk workflow state schema now: rejected as outside
  WORK-0020 and later milestone planning.

## Specification impact

- `Specs/OMVCS Core Specification.md` §§34, 69, and 83.
- `Specs/OMVCS Storage Adapter Specification.md` §§27–28 and 57.
- `Specs/OMVCS Interaction Specification.md` §§160 and 241–243.
- `Specs/OMVCS Glossary.md`, Invocation-Scoped Operation, Durable Resumable
  Workflow, and Transport Resume.
- DG-0038, decision register, WORK-0020/0021, M3 preflight, spec coverage,
  milestone, project state, and handover.

No Core Invariant change is required. Platform Protocol, DAW Adapter, and
Ardour reference semantics are unaffected.

## Test impact

WORK-0020 tests MUST prove:

- success includes returned `ReplicaId`, destination verification, and known
  committed registration;
- source preservation on success and every failure path;
- known CAS non-commit is distinct from any permitted unknown status;
- Adapter stage-local retry/resume occurs only where its existing contract
  permits it and never implies Core success;
- no automatic composed Core retry after returned non-success;
- interruption with possible destination residue reports `incomplete`;
- a later invocation re-observes authoritative state and does not trust
  probable residue;
- size, Chunk count, elapsed time, background execution, and process restart
  do not create a durable continuation promise; and
- no result is fabricated after an invocation disappears with its process.

Future bulk replication conformance belongs to its own work package and
must test the durable workflow contract defined there.

## Implementation impact

WORK-0020 may implement provider-neutral single-Resource replication after
independent specification verification. It MUST NOT implement durable bulk
or M4 recovery machinery. Future bulk/multi-Resource replication requires a
separate bounded work package.

## Compatibility / migration impact

No persisted representation or existing operation result changes. This ADR
clarifies the applicability of Core §69 and leaves ADR-0043 intact.

## Notes

DG-0038 is resolved by this decision. No production code is changed.
