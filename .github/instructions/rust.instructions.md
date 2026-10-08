---
applyTo: "**/*.rs"
description: Rust implementation rules for the OMVCS reference implementation.
---

# Rust rules

- Prefer explicit domain types over primitive-string identifiers once their normative format is resolved.
- Model invalid states so they are difficult or impossible to construct.
- Keep immutable historical objects immutable through the public API.
- Do not use `unsafe` in OMVCS Rust crates unless a future work package explicitly approves and isolates it. Workspace lint currently denies unsafe code.
- Avoid `unwrap`, `expect`, and `panic` in production paths. Return typed errors.
- Never use filenames, provider paths, timestamps, or mutable storage data as creative identity unless a specification explicitly says so.
- Canonical serialization and hashing code must be deterministic and heavily tested.
- Do not add dependencies casually. Explain new dependencies in the work package/handover.
- Keep DAW-specific semantics out of Core.
- Keep provider-specific semantics out of generic Core/storage contracts.
- Public APIs should be smaller than internal implementation APIs.
- Format with rustfmt and run Clippy with warnings denied before handover.
