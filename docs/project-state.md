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

Implementation status: M1 — Normative data model COMPLETE at integration commit `abd0807ed2152aa3e331c6ca058796eb70cf4d4a`. M2 preflight is complete; WORK-0009 is in progress on `work/0009-revision-graph`.
