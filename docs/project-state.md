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

Current milestone: M2 — Local repository history

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
`origin/work/0013-repository-reachability`. DG-0027 blocks Working State
safety-reference root coverage. Contributions, configured
archival pins, and pending publication transactions remain required by Core
§62 but are excluded from this bounded package; it does not claim complete
reachability or global unreachable status. AdapterWorkingStateRef is not
itself a reachability root. Human-approved ADR-0027–ADR-0029 resolve
DEC-CORE-004, DEC-CORE-009, and DG-0019. WORK-0014's validation contract is
executable and the package is PLANNED, not started. Its first implementation
MUST report partial/unavailable coverage for unresolved Core §62 root
classes: DG-0027 still blocks Working State safety-reference roots, while
Contributions, configured archival pins, and pending publication
transactions remain unimplemented. WORK-0014 MUST NOT claim complete
reachability or global unreachable status. DEC-CORE-005/008,
DEC-INTERACTION-004, and DG-0015 remain separate. M2 is IN PROGRESS.
