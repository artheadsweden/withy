# OMVCS Development Ownership

This describes agent responsibility, not legal ownership.

| Path / Concern | Primary agent | Required review |
|---|---|---|
| `crates/omvcs-model/` | Core Engineer | Verifier |
| `crates/omvcs-core/` | Core Engineer | Verifier |
| `crates/omvcs-storage/` | Storage Engineer | Core Engineer + Verifier |
| `crates/omvcs-storage-local/` | Storage Engineer | Verifier |
| `crates/omvcs-storage-mock/` | Storage Engineer | Verifier |
| `crates/omvcs-daw-contract/` | DAW Adapter Engineer | Core Engineer + Verifier |
| `ffi/omvcs-c/` | Core Engineer | DAW Adapter Engineer + Verifier |
| `tests/conformance/` | Verifier | relevant implementer |
| `Specs/` | Spec Guardian after explicit human approval | human approval required |
| `docs/gaps/` | Spec Guardian | Lead |
| `docs/decisions/` | Spec Guardian | human decision required |

Cross-boundary changes must be identified in the relevant work package before implementation when possible.
