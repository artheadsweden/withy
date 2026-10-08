---
applyTo: "Specs/**/*.md"
description: Protection rules for normative OMVCS specifications.
---

# Specification editing rules

Files under `Specs/` are normative and read-only by default.

Before modifying any specification:

1. identify the Design Gap or approved design question;
2. obtain explicit human approval for the decision;
3. create/update the ADR;
4. identify every affected specification;
5. identify test/conformance impact;
6. make the smallest consistent normative change;
7. run a cross-spec terminology and contradiction review.

Never change a specification merely to match existing code.

Never silently rename normative terms.

Never add implementation-specific Rust, Ardour, storage-provider, or Platform behaviour to a generic spec unless the decision is intentionally normative.
