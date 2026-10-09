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
on `spec/0003-canonical-collection-order`. DG-0016, DG-0020, and DG-0022
are resolved by ADR-0016, ADR-0020, and ADR-0022; DG-0021 is resolved by
ADR-0021 and WORK-0010 is PLANNED. DEC-CORE-008 remains separate for
automatic Line-deletion retention/pinning; DEC-INTERACTION-001/003 remain
presentation decisions and do not block the Core Line contract. WORK-0011,
WORK-0012, and WORK-0014 remain blocked by their recorded M2 decisions/gaps.
M2 is IN PROGRESS.
