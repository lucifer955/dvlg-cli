# Release guide

This project uses a standard Rust CLI release flow. Follow the steps below for a clean, traceable release.

## 1) Prepare the release

- Ensure the working tree is clean.
- Update the version in `Cargo.toml`.
- Update `docs/README.md` if any user-facing behavior changed.
- Add or update tests for changes.

## 2) Validate locally

- Run the test suite.
- Build a release binary to ensure it compiles.

## 3) Update changelog notes

- Summarize user-facing changes in your release notes.
- Include any new or changed CLI commands and examples.

## 4) Tag the release

- Create a git tag matching the version (for example, `v1.2.3`).
- Push the tag to the remote.

## 5) Publish

- Publish to your distribution target (for example, crates.io).
- Attach release notes to the GitHub release.

## 6) Post-release checks

- Verify the published version is available.
- Smoke-test the installed CLI.
