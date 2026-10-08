# DG-0002 — Component State parentage requirement

Status: OPEN
Classification: BLOCKS-MILESTONE
Discovered by: OMVCS Lead
Discovered during: M0
Date: 2026-10-08

## Relevant specifications

- `Specs/OMVCS Glossary.md`, Component State.
- `Specs/OMVCS Core Specification.md`, sections 10–11 and 89, question 10.
- `Specs/OMVCS Core Invariants Specification.md`, INV-HIST-003 and INV-PROV-003.

## Problem

The Glossary says a Component State MAY have one or more parent Component States. Core section 10 shows a `parents` field in its conceptual structure, section 11 describes Component lineage, and section 89 explicitly leaves unresolved whether parentage is mandatory or merely recommended.

The specifications therefore do not settle whether a conforming Component State must record lineage, when parentage is required, or whether an initial state is the only permitted parentless case.

## Why the current specifications are insufficient

The permissive Glossary wording and Core's explicit unresolved question do not determine the required presence and validation rules for parent references. Those rules affect valid Component State objects and their content-derived identities.

## Affected work

- M1 Component State model, validation, and identity tests.
- Later Component lineage and provenance behaviour.

## Can unaffected work continue?

Yes. Resource identity and other M1 types not dependent on Component lineage may continue. Component State requirements and conformance cases that assert mandatory or optional parentage must wait for the decision.

## Candidate directions

Non-normative options include making parentage optional for all Component States, requiring it for non-initial states, or treating it as recommended metadata. None is selected here.

## Required decision

Is Component State parentage optional, recommended, or mandatory for defined classes of Component State? If mandatory in any case, how are initial states distinguished and validated?

## Resolution

UNRESOLVED
