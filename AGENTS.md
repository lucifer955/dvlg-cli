# Agent rules

These rules apply to automated coding agents working in this repo.

## Core behavior
- Keep changes small and commit-friendly.
- Prefer explicit, minimal edits over sweeping refactors.
- Do not change public CLI flags without updating docs.
- Preserve existing formatting and naming conventions.

## Safety and git
- Never run destructive git commands.
- Never commit or push without explicit user approval.
- Avoid editing generated files or vendor directories.

## Rust and CLI guidelines
- Prefer stable Rust and well-known crates.
- Surface clear error messages and non-zero exit codes on failure.
- Add tests for new features and bug fixes.
- Update README/docs when behavior changes.

## Documentation
- Keep documentation concise and user-focused.
- Include examples for each new CLI command.

## File layout
- Logs live under `.dvlg/YYYY/MM/DD.yaml`.
- Config lives at `~/.config/dvlg/config.yaml`.
