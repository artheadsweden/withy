# OMVCS Copilot Instructions

Always read `/AGENTS.md` before beginning project work.

The normative OMVCS design lives under `/Specs/`. Treat those files as authoritative and read-only unless an explicit approved specification-change workflow is in progress.

For implementation tasks:

1. Work from a `docs/plans/WORK-*.md` work package.
2. Read every referenced specification section and invariant.
3. Search the rest of `Specs/` for relevant terminology.
4. Check open `docs/gaps/` entries.
5. Never invent missing semantics.
6. Use the Design Gap workflow when necessary.
7. Respect module/agent boundaries in `docs/code-ownership.md`.
8. Add tests from the specification, not from implementation convenience.
9. Update `docs/spec-coverage.md`.
10. Finish with a structured handover.

Reference implementation language:

- OMVCS Core: Rust.
- Public interoperability/API contracts: language-neutral.
- Native reference FFI: stable C ABI, designed later under a dedicated work package.
- Ardour Adapter: expected primarily in C++, outside the Rust Core boundary.

The Rust workspace currently contains bootstrap crates only. Do not treat placeholder APIs as normative.

Remote publishing is controlled by `docs/project-state.md`. Do not add or push to a remote while it says `DISABLED`.

Useful project skills live under `.github/skills/`.
