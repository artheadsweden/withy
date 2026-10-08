# Handover DG-0003 — Hashed collection ordering

From agent: Spec Guardian
To agent: OMVCS Lead / future Core Engineer
Date: 2026-10-08
Branch: `spec/0003-canonical-collection-order`
HEAD: `fa45945530c3a22ab9433d4cc0146913eff44b9d`

## Completed

- Recorded the human-approved array collection-order decision in ADR-0001.
- Resolved DG-0003 for array-valued collections and updated its register entry.
- Updated affected normative specifications and M1 coverage/work packages.
- During cross-spec impact review, identified the separate unresolved interaction between set-like element-byte sorting and hashed JSON object maps. Recorded it as DG-0005 without choosing a rule.
- No production code or tests were added.

## Specifications implemented

- `Specs/OMVCS Core Specification.md` §5.1 and hashed collection descriptions in §§8.2, 10, 12–14.
- `Specs/OMVCS Core Invariants Specification.md`, INV-HIST-006.
- `Specs/OMVCS Glossary.md`, Chunk Manifest.
- `Specs/OMVCS DAW Adapter Specification.md` §§20–22 and §§145–146.
- `Specs/OMVCS Storage Adapter Specification.md` Chunk Manifest metadata.
- `Specs/Ardour Reference Adapter Design.md` §§23 and 31.

## Files changed

- Specs: Core, Core Invariants, Glossary, DAW Adapter, Storage Adapter, Ardour Reference Adapter Design.
- Decision/gap records: `docs/decisions/ADR-0001-hashed-collection-ordering.md`, DG-0003 resolved, DG-0005 open, `docs/decision-register.md`.
- Planning/coverage: `docs/spec-coverage.md`, WORK-0001 through WORK-0003, WORK-0006 through WORK-0008.
- This handover.
- `docs/project-state.md` has a pre-existing unrelated working-tree modification and was not intentionally changed by this task.

## Tests added or changed

None. This work is specification and planning documentation only; acceptance tests are specified in ADR-0001 and the relevant work packages.

## Commands run

- `git diff --check` — passed; Git emitted line-ending conversion warnings for touched Markdown files.
- No build, unit test, conformance test, formatting, or Clippy command was run because no code was changed.

## Semantic decisions made beyond the specification

None. The only approved semantic decision is the human-provided array collection policy in ADR-0001. No map-entry semantics were selected; DG-0005 remains open.

## Design Gaps discovered

- DG-0003 — RESOLVED for array-valued collections by ADR-0001.
- DG-0005 — OPEN, BLOCKS-MILESTONE: determine how set-like byte sorting applies to hashed JSON object maps alongside RFC 8785 member ordering.
- DG-0001, DG-0002, DG-0004 remain open and unchanged.

## Assumptions

- None. Work-package readiness is not inferred past the open DG-0005 and other listed gaps.

## Known limitations

- Array-valued collection rules are documented, but M1 canonical identity conformance for hashed JSON object maps cannot be finalized until DG-0005 is resolved.
- Documentation updates are not committed or pushed.

## Remaining work

1. Obtain a human decision for DG-0005.
2. Update ADR/specifications, decision register, coverage, and affected work packages according to that decision.
3. Independently verify the final M1 conformance plan before implementation.

## Git state

Working tree: DIRTY (spec/documentation changes from this task plus the pre-existing `docs/project-state.md` modification; no commit made).
Remote push performed: NO. Remote publishing is enabled in the current project state; no push was requested for this task.
