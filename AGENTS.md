# OMVCS Agent Constitution

This repository implements the Open Music Version Control System (OMVCS).

## Authority

The files under `Specs/` are the normative design authority for OMVCS.

Code implements the specifications. Code does not complete, reinterpret, or silently extend them.

When a specification is unclear, incomplete, or contradictory:

1. Search the full specification set before concluding that the answer is missing.
2. Do not invent semantics.
3. Create or update a Design Gap under `docs/gaps/`.
4. Classify the gap as `NON-BLOCKING`, `BLOCKS-FEATURE`, or `BLOCKS-MILESTONE`.
5. Continue only if the unresolved point cannot affect correctness of the current work.
6. If blocked, stop the affected work and report the gap.

A TODO comment is not a substitute for a Design Gap when the missing item concerns OMVCS semantics.

## Specification changes

Files under `Specs/` are read-only by default.

An agent may modify a specification only when all of the following are true:

- the user explicitly approved the semantic decision;
- the decision has been recorded in an ADR under `docs/decisions/`;
- the affected specifications have been identified;
- corresponding conformance tests or test plans are updated.

Implementation convenience is never sufficient reason to change a specification.

## Architecture boundaries

Respect the boundaries already defined by the specifications:

- OMVCS Core owns version-control history and repository semantics.
- DAW Adapters own interpretation of DAW-native state.
- Storage Adapters own provider mechanics, not creative history.
- Platforms are replaceable collaboration and discovery layers.
- Resource identity is independent of physical storage.
- Platform availability is not part of Revision durability.
- Ardour is a reference DAW, not the definition of OMVCS.
- Rust is the reference Core implementation language, not part of OMVCS semantics.
- Public interop boundaries must remain language-neutral.

Do not bypass an adapter or public API because changing another module seems easier.

## Work packages

Production work must be tied to a work package under `docs/plans/`.

A work package must identify:

- specification sections and invariants implemented;
- dependencies;
- allowed scope;
- deliverables;
- acceptance tests;
- explicit non-goals;
- known Design Gaps;
- owner agent;
- expected handover.

Do not begin broad implementation from an informal request such as "implement OMVCS Core".

## Preflight

Before editing production code:

1. Read this file.
2. Read the assigned work package.
3. Read every referenced specification section and invariant.
4. Search the rest of `Specs/` for the important normative terms involved.
5. Check `docs/gaps/` for open gaps affecting the work.
6. Inspect the current branch and working tree.
7. Confirm the task boundary.
8. State a concise implementation and test plan.
9. Only then edit code.

## Testing

The specification is the source of test intent.

Implementation work must include appropriate tests. Critical semantics should prefer direct conformance tests, property tests, failure-injection tests, or fuzz tests where useful.

The implementing agent is not the final judge of its own implementation. The Verifier must independently compare behaviour with the specification.

Never weaken or remove a valid test merely to make an implementation pass.

## Git safety

- Never work directly on `main`.
- One work package should normally use one branch.
- Use branch names such as `work/0017-revision-object`, `fix/0042-replica-verification`, or `spec/0012-actor-identity`.
- Never force-push a shared branch.
- Never rewrite public history.
- Never use destructive Git commands to discard user work without explicit approval.
- Never resolve a merge conflict with blanket "ours" or "theirs" unless equivalence is mechanically proven.
- Never commit secrets, credentials, private keys, access tokens, or temporary signed URLs.
- Keep commits focused on the work package.
- Do not perform unrelated refactors.
- Run formatting and relevant tests before integration.
- Do not push until `docs/project-state.md` says remote publishing is `ENABLED`.

## Completion and handover

Before declaring a work package complete:

1. Run formatting.
2. Run relevant unit and integration tests.
3. Run applicable conformance tests.
4. Run Clippy with warnings denied for touched Rust crates.
5. Review `git diff` for unrelated changes.
6. Update `docs/spec-coverage.md`.
7. Record every discovered Design Gap.
8. Create a handover from `docs/handovers/HANDOVER-TEMPLATE.md`.
9. Leave the working tree clean unless the handover explicitly explains otherwise.

A handover must explicitly state whether any semantic decisions were made beyond the specification. Normally the correct answer is `None`.
