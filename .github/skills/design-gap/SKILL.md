---
name: design-gap
description: Record an OMVCS semantic ambiguity, omission, or contradiction without allowing an agent to invent the missing rule.
---

# Design Gap procedure

Use this skill whenever implementation or planning requires behaviour not clearly defined by the Specs.

1. Search the complete specification set first.
2. Determine whether the answer exists elsewhere or follows unambiguously.
3. If genuinely missing/contradictory, allocate the next DG number.
4. Copy `docs/gaps/DG-TEMPLATE.md`.
5. Cite exact relevant specification sections.
6. Explain why choosing a behaviour would invent semantics.
7. Classify:
   - NON-BLOCKING
   - BLOCKS-FEATURE
   - BLOCKS-MILESTONE
8. List affected work and what may safely continue.
9. Candidate directions may be documented, but mark them explicitly non-normative.
10. State the exact human decision required.
11. Add/update the entry in `docs/decision-register.md`.
12. Do not modify production semantics or Specs until the gap is resolved through an ADR and approved Spec update.
