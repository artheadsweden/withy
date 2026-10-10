# OMVCS Project State

This file records operational development-project state that agents must not infer from chat history.

## Remote publishing

Remote publishing: ENABLED

Approved remote: origin

Rules:

- Agents may initialize Git, create branches, and commit locally.
- Agents must not add, guess, create, or push to a remote while publishing is disabled.
- The user must configure the remote explicitly.
- After the user says the remote is configured, an agent must verify with `git remote -v`.
- Only then may this file be changed to `Remote publishing: ENABLED`.
- Never record credentials, tokens, or signed URLs here.

## Current milestone

Current milestone: M3 — Storage abstraction (IN PROGRESS; WORK-0015 VERIFIED and integrated at aade8af1031a42fd9c7d093f35f6c4ae3f2818a0; ADR-0032 resolves DG-0029 and ADR-0033 resolves DEC-STORAGE-007; WORK-0016 Replica/locator modeling may proceed, but guarded map updates await DG-0032 and verified promotion awaits WORK-0017; WORK-0017–WORK-0021 remain unstarted)

## Specification state

Top-level specification set: Draft 0.1

Implementation status: M1 — Normative data model COMPLETE at integration
commit `abd0807ed2152aa3e331c6ca058796eb70cf4d4a`. M2 preflight is complete;
WORK-0009 is verified and integrated at `3139bc5eeb4e29ce931e217b6e815a3a88b75772`
on `spec/0003-canonical-collection-order`. DG-0016, DG-0020, DG-0021, and
DG-0022 are resolved by ADR-0016, ADR-0020, ADR-0021, and ADR-0022.
WORK-0010 is independently verified and integrated at
`ef1937dcef4079cd6bd863c584464691e1ac72e6` on
`spec/0003-canonical-collection-order`. DEC-CORE-008 remains separate for
automatic Line-deletion retention/pinning; DEC-INTERACTION-001/003 remain
presentation decisions and do not block the Core Line contract. ADR-0017
resolves the Release object and admission contract; WORK-0011 is independently
verified and integrated at `e179d27147ea6efcac3aa46fedcecd34ddd5725f` on
`spec/0003-canonical-collection-order`. Its feature branch is pushed to
`origin/work/0011-releases`.
Accepted, human-approved ADR-0023 resolves DG-0023 only for the exact
`CreateRelease` integrity/name-conflict overlap, with atomic failure and
unchanged stored objects/bindings; it defines no general failure precedence.
DEC-CORE-002 remains separate
and does not block WORK-0011. Human-approved ADR-0018 resolves DG-0018's
Working State semantics. Human-approved ADR-0024–ADR-0026 resolve
DG-0024–DG-0026 for destructive replacement authorization,
AdapterWorkingStateRef persistence/recovery, and operation
failure/retry/idempotency contracts. DEC-INTERACTION-004 remains OPEN and
separate for temporary checkpoints; it does not block WORK-0012 or WORK-0013.
WORK-0012 is independently verified and integrated at
`fd68d969da671546ec1e00f5362a75c0a52ed61f` on
`spec/0003-canonical-collection-order`; its feature branch is pushed to
`origin/work/0012-working-state-lifecycle`. WORK-0013 is independently verified
and integrated at `a67e1811ce0e7e15001f8517d379d7da607d3bdf` on
`spec/0003-canonical-collection-order` for the explicitly partial Line/Release
root subset; its feature branch is pushed to
`origin/work/0013-repository-reachability`. At WORK-0013's original
integration, DG-0027 remained open and Working State safety-reference roots
were excluded. Contributions, configured
archival pins, and pending publication transactions remain required by Core
§62 but are excluded from this bounded package; it does not claim complete
reachability or global unreachable status. AdapterWorkingStateRef is not
itself a reachability root. Human-approved ADR-0027–ADR-0029 resolve
DEC-CORE-004, DEC-CORE-009, and DG-0019. WORK-0014 is independently accepted
and integrated at `3f37fa1a981fa06408616b080edb6c56355bc1b0`. Repository
validation originally reused partial Line/Release reachability, reported
unsupported Core §62 roots, and did not claim complete reachability or global
unreachable status. At WORK-0014's original integration, DG-0027 and the
other unsupported root classes were outside its accepted scope.
ADR-0030 resolves DG-0027: current Working State Base Revision and present
component-source Component States are safety-reference roots. DG-0028 is
resolved by ADR-0031: absent Working-State-root targets are unresolved, with
no declared-boundary lookup. The required WORK-0013/0014 implementation extension on
`work/0013-working-state-roots` passed acceptance validation and received
independent Verifier ACCEPT with no findings; it is integrated at
`3eb1c3e01fa01904f8925f9bcdf3171059594c50`. The M2 closeout audit found
WORK-0009 through WORK-0014 verified/integrated at their bounded scopes and
no open M2-blocking Design Gap. DG-0015 remains open for M6 Contribution
semantics; DEC-CORE-005/008 and DEC-INTERACTION-004 remain separate.
Contributions, configured archival pins, and pending publication
transactions remain unsupported Core §62 root classes; reachability is
explicitly partial and does not classify global unreachability. M2 is
COMPLETE at the stated integration commit. M3 bounded preflight is complete:
WORK-0015–WORK-0021 are defined. Human-approved ADR-0032 resolves
DG-0029's Replica identity/cardinality and representation-binding gap;
ADR-0033 resolves DEC-STORAGE-007's provider-locator representation.
WORK-0015's restricted byte-I/O implementation has passed focused and
workspace tests, formatting, warnings-denied Clippy, Core Engineer review,
and independent Verifier review; it is integrated at
`aade8af1031a42fd9c7d093f35f6c4ae3f2818a0`. DG-0032 remains OPEN and blocks
guarded/versioned Storage Map mutations and completion of WORK-0016; its
Replica/locator model may proceed. DEC-STORAGE-004/005 continue to gate
verification evidence and upload assurance, so verified promotion cannot
be implemented until the WORK-0017 result contract is approved. WORK-0017
may consume WORK-0016's independently reviewed model/locator subdeliverable
without waiting for WORK-0016's full completion; WORK-0016 promotion
integration follows the approved WORK-0017 result. Incomplete/unverified
candidates are not registered Resource Replicas. No implementation beyond
WORK-0015 has started.
DEC-STORAGE-011 remains OPEN; stable shared-namespace, cross-Project
physical-object/deduplication guarantees, and shared-namespace GC are
excluded from WORK-0016. DEC-CORE-001 and DEC-STORAGE-002 remain OPEN; no
chunking algorithm was selected. WORK-0016–WORK-0021 have not started
implementation.
