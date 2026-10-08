---
name: spec-impact-analysis
description: Trace an OMVCS question or proposed change across the complete specification set and identify affected invariants, terms, tests, and implementation areas.
---

# Spec impact analysis

Use this skill before changing normative behaviour or planning work with cross-cutting semantics.

1. Identify the exact normative term(s) involved from `Specs/OMVCS Glossary.md`.
2. Search all eight Specs for those terms and close synonyms.
3. List directly applicable Core Invariants.
4. Separate normative rules from examples and unresolved-question lists.
5. Identify every specification section that would be affected by the proposed change.
6. Identify existing Design Gaps and ADRs on the topic.
7. Identify test/conformance impact.
8. Identify implementation modules affected.
9. State whether the change can be local or requires cross-spec revision.
10. Do not resolve missing semantics unless a human decision has been made.

Output a concise impact table plus unresolved questions.
