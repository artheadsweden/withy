---
name: conformance-test
description: Convert OMVCS normative requirements into implementation-independent conformance, property, scenario, and failure-injection tests.
---

# Conformance test procedure

1. Start from exact specification text/invariant IDs.
2. State the observable behaviour under test.
3. Avoid depending unnecessarily on private implementation structure.
4. Include success and failure cases.
5. For invariants, prefer property tests when the rule should hold for broad classes of valid inputs.
6. For storage/transactions, include interruption, retry, corruption, stale generation, and partial-failure cases where relevant.
7. If expected behaviour is not specified, stop and create a Design Gap instead of encoding a guess in the test.
8. Link the test to the requirement in `docs/spec-coverage.md`.
9. Name tests after the required behaviour, not after implementation functions.
