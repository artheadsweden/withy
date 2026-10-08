---
applyTo: "**/tests/**/*.rs"
description: Conformance and verification rules for OMVCS tests.
---

# Test rules

Tests should trace back to normative specifications, invariants, approved ADRs, or explicit failure behaviour.

Prefer names that describe the invariant or observable behaviour.

For critical rules, include negative tests and failure injection.

Good examples:

- moving_resource_location_does_not_change_revision_identity
- stale_line_generation_is_rejected
- corrupt_replica_is_never_registered_as_verified
- platform_outage_does_not_invalidate_durable_revision

Do not write tests that merely mirror implementation internals.

Do not weaken a valid test to accommodate implementation behaviour.

When testing a Design Gap, mark the test blocked/pending in project planning rather than encoding an unapproved answer.
