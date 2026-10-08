# OMVCS Development Workflow

This is a permanent process document. It is not an OMVCS specification.

## Work lifecycle

Every implementation change should follow:

```text
User objective
  -> OMVCS Lead
  -> specification impact analysis
  -> bounded WORK item
  -> implementation
  -> independent verification
  -> handover
  -> integration
```

When ambiguity appears:

```text
Implementer
  -> Spec Guardian
  -> answer already specified: continue
  -> genuine gap: create DG
      -> NON-BLOCKING: document and continue only where unaffected
      -> BLOCKS-FEATURE: stop affected feature
      -> BLOCKS-MILESTONE: stop milestone
  -> human decision
  -> ADR
  -> update Specs
  -> update tests
  -> resume implementation
```

## Design Gaps

A Design Gap means the normative design is missing, ambiguous, or contradictory.

Design Gaps are stored under `docs/gaps/`.

Agents may document candidate directions but may not select one without an approved decision.

## ADRs

An ADR records why a semantic or architectural decision was made.

ADRs do not replace Specs.

After a decision is approved:

1. record ADR;
2. update affected Specs;
3. update conformance tests/test plans;
4. update work packages;
5. implement.

## Work packages

Each work package is a Markdown file under `docs/plans/` and must reference exact normative requirements.

A work package should be narrow enough to implement and verify coherently.

## Verification

The Verifier independently reads the original normative requirements and attempts to disprove implementation correctness.

Review must cover behaviour, failure modes, concurrency/integrity where relevant, and omissions.

## Handover

Use `docs/handovers/HANDOVER-TEMPLATE.md`.

The handover is part of the work product, not optional administration.

## Git

One work package normally corresponds to one branch.

Branches should remain focused. Integration occurs only after verification.

Remote push rules are controlled by `docs/project-state.md`.
