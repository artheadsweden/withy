# OMVCS Project Setup and Initialization
## TEMPORARY BOOTSTRAP DOCUMENT

This document exists only to initialize the OMVCS repository safely. Delete it after milestone M0 is complete, the local development environment works, and remote publishing has been explicitly enabled and verified.

Do not treat this document as an OMVCS specification.

---

## 1. Expected starting point

The repository root should contain the eight existing specification files under:

```text
Specs/
```

Expected specification set:

```text
Specs/
├── Ardour Reference Adapter Design.md
├── OMVCS Core Invariants Specification.md
├── OMVCS Core Specification.md
├── OMVCS DAW Adapter Specification.md
├── OMVCS Glossary.md
├── OMVCS Interaction Specification.md
├── OMVCS Platform Protocol.md
└── OMVCS Storage Adapter Specification.md
```

Do not rename, rewrite, normalize, or reformat these files during bootstrap.

---

## 2. Install Rust

Install Rust using rustup from the official Rust project.

After installation, close and reopen the VS Code terminal if necessary.

Verify:

```powershell
rustc --version
cargo --version
rustup --version
```

The repository contains `rust-toolchain.toml`, so rustup should install/select the declared toolchain components automatically when Cargo is first run.

Verify the active toolchain:

```powershell
rustup show
```

---

## 3. Recommended VS Code extensions

Open the repository root in VS Code.

The repository recommends extensions through `.vscode/extensions.json`.

At minimum, ensure these are available:

- GitHub Copilot
- GitHub Copilot Chat
- rust-analyzer

CodeLLDB is useful later for native debugging but is not required for basic bootstrap.

---

## 4. Verify repository layout

Confirm that these top-level files/directories exist:

```text
AGENTS.md
README.md
SETUP-AND-INITIALIZATION.md
Cargo.toml
rust-toolchain.toml
.gitignore
.gitattributes
.editorconfig

Specs/
docs/
crates/
ffi/
tools/
tests/
schemas/
.github/
.vscode/
```

The bootstrap package creates all non-Spec files. The existing `Specs/` directory should remain the one already in the project.

---

## 5. Initialize Git locally

If the project is not already a Git repository:

```powershell
git init
```

Then verify:

```powershell
git status
```

Do not add a remote yet unless you, the user, have chosen and created the destination repository yourself.

The repository starts with:

```text
docs/project-state.md
Remote publishing: DISABLED
```

Agents must respect this state.

---

## 6. Validate Rust workspace

From the repository root:

```powershell
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

The bootstrap code is intentionally minimal. All commands should pass before real implementation begins.

`cargo check` or `cargo test` will create `Cargo.lock`. Commit `Cargo.lock`; this repository is a reference application/workspace, so it is intentionally tracked.

---

## 7. Initial Git commit

Review the files before committing:

```powershell
git status
git diff -- . ':!Specs'
```

Do not allow bootstrap tooling to rewrite the specifications.

Then stage the repository:

```powershell
git add .
```

Inspect staged changes:

```powershell
git diff --cached
```

Create the initial local commit:

```powershell
git commit -m "chore: bootstrap OMVCS development workspace"
```

If the Specs were already committed before this bootstrap, keep their history intact rather than recreating them unnecessarily.

---

## 8. Configure the remote manually

The agents are explicitly forbidden from inventing or creating the remote.

When you are ready:

1. Create or choose the remote Git repository yourself.
2. Add it manually, for example:

```powershell
git remote add origin <YOUR-REMOTE-URL>
```

3. Verify:

```powershell
git remote -v
```

4. Tell the Copilot/OMVCS Lead agent that the remote has now been configured.

The agent must independently run `git remote -v` before considering remote publishing enabled.

Do not let an agent guess the URL.

---

## 9. Enable remote publishing

After you have configured the correct remote and explicitly approved pushes, edit:

```text
docs/project-state.md
```

Change:

```text
Remote publishing: DISABLED
```

to:

```text
Remote publishing: ENABLED
Approved remote: origin
```

Record the remote host/repository description without including credentials or tokens.

Commit the change.

Only after this is committed may agents push normally.

---

## 10. First push

After remote publishing is enabled:

```powershell
git branch -M main
git push -u origin main
```

If the remote already contains history, stop and inspect it before pulling, merging, rebasing, or overwriting anything.

Never force-push merely to make bootstrap convenient.

---

## 11. Verify Copilot customizations

In VS Code, inspect the Agent Customizations/agents picker.

The workspace should expose these agents:

- OMVCS Lead
- Spec Guardian
- Core Engineer
- Storage Engineer
- DAW Adapter Engineer
- Verifier

If they do not appear, check that:

```text
.github/agents/*.agent.md
```

is present and valid.

The repository also includes skills under:

```text
.github/skills/
```

and targeted instructions under:

```text
.github/instructions/
```

---

## 12. Run milestone M0

Once the local workspace is healthy, ask:

```text
OMVCS Lead: initialise milestone M0.
Do not implement OMVCS semantics yet.
Audit the repository setup, have the Spec Guardian build the initial decision register from all unresolved 0.1 items in Specs, classify which decisions block which milestones, validate the spec-coverage mechanism, and produce bounded work packages for M1.
```

M0 should focus on:

- repository/tooling health;
- extracting unresolved decisions from all eight Specs;
- identifying contradictions before implementation;
- setting up the first spec-coverage entries;
- deciding which unresolved items block M1;
- preparing M1 work packages.

M0 is not permission to implement the Core broadly.

---

## 13. Choose a code licence before public release

The bootstrap intentionally does **not** choose an open-source code licence on your behalf.

Before the repository is made public or presented as reusable software, decide the code licence and add the appropriate `LICENSE` file and Cargo package metadata.

This does not block local M0/M1 development, but it should be resolved before public distribution.

---

## 14. When this file can be deleted

Delete `SETUP-AND-INITIALIZATION.md` only when all of the following are true:

- Rust toolchain works;
- the workspace passes formatting/check/clippy/tests;
- Git is initialized;
- the bootstrap commit exists;
- Copilot agents/skills/instructions load correctly;
- M0 has created the initial decision register and M1 plan;
- the remote is either intentionally still disabled or has been explicitly configured and verified;
- `docs/project-state.md` accurately records the remote state.

After deletion, permanent process rules remain in:

```text
AGENTS.md
docs/development-workflow.md
.github/copilot-instructions.md
```

Commit the deletion as a normal M0 cleanup change.
