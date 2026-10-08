# Open Music / OMVCS

This repository contains the reference implementation work for the **Open Music Version Control System (OMVCS)**.

OMVCS is defined by the normative documents under `Specs/`. The reference Core implementation is planned in Rust, while DAW and storage integrations communicate through language-neutral contracts.

## Current status

The project is in bootstrap and specification-consolidation stage.

Do not begin broad production implementation until:

- repository bootstrap is complete;
- the specification decision register has been populated;
- milestone M0 is complete;
- blocking Design Gaps for the next milestone are resolved.

## Important directories

- `Specs/` — normative OMVCS specification set.
- `docs/decisions/` — Architecture Decision Records (decisions already made).
- `docs/gaps/` — unresolved Design Gaps.
- `docs/plans/` — implementation work packages.
- `docs/handovers/` — structured agent handovers.
- `crates/` — Rust reference implementation crates.
- `ffi/` — language-neutral/native FFI layer.
- `tools/` — developer and reference tools.
- `tests/` — cross-crate conformance/scenario/failure tests.
- `schemas/` — normative/reference serialized schemas once defined.
- `.github/agents/` — durable Copilot agent roles.
- `.github/skills/` — reusable Copilot procedures.

## Start here

During initial setup, follow `SETUP-AND-INITIALIZATION.md`.

That file is intentionally temporary and may be deleted after M0 setup has been completed and the repository is operating normally.

All agents must read `AGENTS.md`.
