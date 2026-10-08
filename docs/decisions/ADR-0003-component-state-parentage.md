# ADR-0003 — Optional Component State parentage

Status: ACCEPTED
Date: 2026-10-08
Decision owner: Human
Related Design Gap: DG-0002

## Context

Component State parentage represents semantic creative derivation. The Glossary described parentage as optional, while Core listed a `parents` field and left unresolved whether parentage was mandatory or recommended. DAW Adapter guidance also prohibited silently inventing lineage. Implementations therefore lacked a complete rule for initial states, derived states, and historical states whose origins are unknown.

## Decision

1. Component State parentage is optional in the OMVCS 0.1 schema.
2. An initial Component State has zero parents.
3. A derived Component State SHOULD record one or more parent Component States when its lineage is known.
4. Parentage is mandatory only where a specific OMVCS operation or provenance rule explicitly requires preserving the derivation.
5. Unknown historical lineage MUST NOT be fabricated.
6. Schema field-presence interpretation: an explicitly empty `parents` array means the state is known to have zero parents; an omitted `parents` field means parentage is unknown or was not asserted. Omission MUST NOT be treated as proof that the state is initial.
7. When present, `parents` is a set-like collection under the collection rules established by ADR-0001.

## Rationale

This permits states with incomplete historical knowledge without requiring invented ancestry, while encouraging known derivations to remain visible. It also preserves a distinction between a known initial state and an older or imported state whose lineage is unavailable.

## Alternatives considered

- Require parentage for every non-initial state: rejected because historical lineage may be unknown and must not be fabricated.
- Treat all parentless states as initial: rejected because it would conflate missing historical information with known origin.
- Allow known derived lineage to be omitted without qualification: rejected as it would weaken discoverability of derivation without a reason; the approved default is SHOULD record known parents.

## Specification impact

- `Specs/OMVCS Core Specification.md` §§5.1, 10–11, 23, 89: classify the optional parent collection; define initial, derived, omitted, and explicitly empty cases; remove the resolved Core question.
- `Specs/OMVCS Glossary.md`, Component State: define optional parentage and known-lineage expectations.
- `Specs/OMVCS Core Invariants Specification.md`, INV-PROV-003: preserve discoverability where derivation is known without fabricating unknown lineage.
- `Specs/OMVCS DAW Adapter Specification.md` §28: align split/mapping behavior with optional, non-fabricated parentage.
- `Specs/Ardour Reference Adapter Design.md` §27: align the reference adapter's lineage proposal rule.

## Test impact

M1 conformance tests must cover:

- valid initial Component State with `parents: []`;
- valid state with omitted parentage, without inferring it is initial;
- valid derived state with one or more known parents;
- a derived state with known lineage records parentage by default (SHOULD behavior);
- operations/provenance rules that explicitly require derivation preservation enforce it;
- unknown historical lineage remains omitted and is never fabricated;
- parent set ordering and duplicate handling follow Core Specification §5.1.

These cases are tracked in WORK-0006 and `docs/spec-coverage.md`. No code or tests are part of this documentation task.

## Implementation impact

- `crates/omvcs-model/`: optional Component State parent collection and validation (WORK-0006).
- Component-lineage capture and user-confirmed mapping in DAW Adapter workflows.
- Later Contribution/provenance flows must preserve derivation when their explicit operation rules require it.

## Compatibility / migration impact

Existing Draft 0.1 conceptual examples that omit parentage cannot on that basis alone be treated as known initial states; omission means unknown or unasserted parentage under this decision. No production objects or implementations are known in this bootstrap workspace.

## Notes

This ADR resolves parentage requirements only. It does not add operation-specific mandatory-parent rules; such a rule must be explicit in the normative operation or provenance contract that needs it.
