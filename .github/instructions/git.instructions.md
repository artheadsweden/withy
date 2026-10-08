---
description: Git branch, commit, merge, and remote-publishing rules for OMVCS.
---

# Git rules

- Never work directly on `main`.
- One work package normally uses one branch.
- Never force-push shared/public history.
- Never discard user work with destructive reset/clean commands without explicit approval.
- Never resolve merge conflicts mechanically when semantics differ.
- Keep commits focused.
- Do not mix unrelated refactors with functional work.
- Use commit subjects such as:
  - `core(WORK-0017): add immutable revision model`
  - `test(WORK-0017): add revision identity properties`
  - `docs(DG-0012): record actor identity gap`
- Never commit credentials or signed access URLs.
- Before pushing, read `docs/project-state.md`.
- If remote publishing is `DISABLED`, do not add a remote, guess a URL, create a remote repository, or push.
- When the user says a remote is configured, verify with `git remote -v` before enabling publishing.
