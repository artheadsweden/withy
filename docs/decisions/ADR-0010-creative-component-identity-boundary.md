# ADR-0010 — Creative Component identity and object boundary

Status: ACCEPTED
Date: 2026-10-08
Decision owner: Human
Related Design Gap: DG-0010

## Context

Core Specification §9 showed a conceptual Component record containing `component_id`, `project_id`, `kind`, `name`, and `created_at`, but the Specs did not define the generic Creative Component object or the historical semantics of those fields. Treating that example as a schema would couple Component identity to project ownership or descriptive metadata without an approved decision.

## Decision

1. In OMVCS 0.1, a Creative Component is a stable logical identity anchor. The normative generic Core Creative Component object contains only the REQUIRED assigned, globally stable Creative Component Identifier, `component_id`. No other field belongs to that generic object.
2. The Component Identifier is independent of Resource content, Component State parentage, DAW-native identifiers, Project membership, names, descriptive metadata, timestamps, storage, Platform accounts, and locations.
3. `project_id` MUST NOT be a field of the generic Creative Component object. Project association is represented through Project State membership/reference: a Project State determines which Creative Components and corresponding Component States participate in that historical Project State.
4. The absence of `project_id` MUST NOT be interpreted as approval of any cross-Project reuse, copy, import, move, clone, fork, ownership, or identity-preservation behavior. This ADR defines none of those semantics.
5. `kind`, `name`, `created_at`, and comparable descriptive values are not fields of the generic OMVCS 0.1 Creative Component object and MUST NOT affect Component identity. Generic Core MUST NOT infer historical significance for such values.
6. Presentation/friendly names and UI labels belong to mutable descriptive, local, or Platform metadata unless another approved historical schema explicitly gives them historical meaning. DAW-native names, classifications, identifiers, and reconstruction-relevant values belong in Adapter State when required by the applicable Adapter. Values that an approved historical Component State schema explicitly defines as creative state belong in that Component State and affect it according to that schema.
7. A rename or descriptive metadata change MUST NOT create a new Creative Component Identifier. Generic Core MUST NOT require a new Component State merely because presentation metadata changes. A Component State changes only when its applicable historical schema says a changed value belongs to that state.
8. `created_at` is not a generic Creative Component historical field. Implementations MAY maintain operational/audit creation timestamps outside the generic object; such timestamps MUST NOT affect Component identity or historical object identity unless another approved schema explicitly includes them elsewhere.

The normative 0.1 Core representation is:

```json
{
  "component_id": "019cc..."
}
```

Project State membership and Component State history are separate relationships and are not fields of this object.

## Rationale

Separating stable logical identity from Project State membership avoids duplicating the association and leaves future cross-Project semantics undecided. Keeping names, classifications, and timestamps out of the generic object prevents mutable presentation or adapter-specific meaning from silently becoming Core identity or history.

## Alternatives considered

- Put `project_id` in the Creative Component object: rejected because it duplicates Project State membership and would imply an unapproved ownership or reuse model.
- Include `kind`, `name`, or `created_at` in the generic object: rejected because no generic Core meaning is selected for them; names are descriptive, `kind` is schema-specific, and `created_at` is operational/audit metadata.
- Derive Component identity from Resource content, Component State parentage, DAW-native identity, or Project membership: rejected; identity is an assigned stable identifier and remains independent of those values.
- Define clone, copy, import, fork, move, or cross-Project reuse behavior: not decided by this ADR.

## Specification impact

- `Specs/OMVCS Core Specification.md` §§9–10 and 13: define the one-field generic object, Component State's identity reference, and Project State membership association.
- `Specs/OMVCS Glossary.md`: distinguish the identity anchor, its Component States, and Project State membership.
- `Specs/OMVCS Core Invariants Specification.md`, INV-PROJ-002: state identity independence and the descriptive-field boundary.
- `Specs/OMVCS DAW Adapter Specification.md` §§25–27: clarify that rename/presentation changes preserve Component identity and only schema-defined historical changes affect Component State.
- `Specs/OMVCS Interaction Specification.md` §§51 and 56 and `Specs/OMVCS Platform Protocol.md` §§7 and 32: clarify that UI labels and indexed names are descriptive presentation metadata, not fields of the generic Core object.
- `Specs/Ardour Reference Adapter Design.md` §§21 and 24: clarify that example names are presentation labels and route rename handling does not alter generic Component identity.

No additional field schema, naming vocabulary, clone/fork/copy/import/move behavior, or cross-Project ownership semantics is defined.

## Test impact

WORK-0005 MUST cover:

- required, typed assigned `component_id` and no other generic Component object field;
- Component identity stability across Resource replacement/re-recording and independence from Project membership, DAW-native identifiers, names, kind/classification, timestamps, filenames, storage, Platform accounts, and locations;
- Project association by Project State membership/reference, not a Component `project_id`;
- presentation rename does not create a new Component Identifier or require a new Component State absent an applicable schema rule;
- no clone/fork/copy/import/move or cross-Project reuse behavior is inferred or implemented.

WORK-0006 and WORK-0007 MUST preserve the distinction between Component identity, Component State, and Project State membership.

## Implementation impact

- WORK-0005 models only the generic Creative Component identity object.
- WORK-0006 refers to that identity from Component State; it does not expand the generic Component object.
- WORK-0007 expresses Project membership through the Project State `components` map from Component Identifier to Component State Identifier.
- Platform, Interaction, and DAW Adapter presentation/mapping may use descriptive labels, but they do not add fields to the generic Core object.

## Compatibility / migration impact

No production Creative Component objects or implementations are known in this workspace. The former conceptual §9 example is superseded and was not a normative schema. No cross-Project migration, reuse, ownership, or fork behavior is introduced.

## Notes

No semantic decisions were made beyond the human-approved decision recorded here.
