# dvlg implementation steps (full details)

This plan is designed to be **commit-friendly**, with each step producing a small, reviewable change.

## 1) Initialize Rust CLI project (commit 1)

- Create a new Cargo binary named `dvlg`.
- Add `src/main.rs` with a basic CLI entrypoint that prints help.
- Ensure the binary name is `dvlg` in `Cargo.toml`.
- Commit message: `chore: initialize dvlg CLI`

## 2) Define file layout and YAML schema (commit 2)

- Document the storage root as `.dvlg/` in the repo.
- Define file layout: `.dvlg/YYYY/MM/DD.yaml`.
- Define entry schema (required fields):
  - `timestamp` (RFC 3339)
  - `message`
  - `tags` (array, can be empty)
  - `decision` (bool)
  - `git` object: `repo`, `branch`, `commit`
- Add an example YAML entry to docs.
- Commit message: `docs: add dvlg storage schema`

## 3) CLI framework + config loading (commit 3)

- Add CLI parsing (e.g., clap).
- Add config loader for `~/.config/dvlg/config.yaml` (optional fields: `auto_commit`, `default_project`, `editor`).
- Add `--help` for all commands.
- Commit message: `feat: add CLI parsing and config loading`

## 4) Implement `dvlg init` (commit 4)

- Create `.dvlg/YYYY/` for current year.
- If already exists, exit cleanly with a message.
- Add tests for init in temp directories.
- Commit message: `feat: add dvlg init`

## 5) Implement `dvlg add` (commit 5)

- Resolve today’s log path using local date.
- Create parent dirs if missing.
- Read existing YAML list (or create new).
- Append a new entry with:
  - `timestamp` (now)
  - `message` (positional argument)
  - `tags` (parse `--tag` repeated)
  - `decision` (flag `--decision`)
  - `git` context (repo root, branch, commit if available)
- Optional auto-commit: if config `auto_commit: true`, run `git add .dvlg/...` and `git commit -m`.
- Commit message: `feat: add dvlg add`

## 6) Implement `dvlg today` (commit 6)

- Load today’s YAML file.
- Render:
  - Date header.
  - Bulleted list of entries, decisions marked with `*` or `[decision]`.
- If no file exists, print “No entries for today.”
- Commit message: `feat: add dvlg today`

## 7) Implement `dvlg list --week/--month` (commit 7)

- Compute date range (last 7 days or current month).
- Scan `.dvlg/YYYY/MM/DD.yaml` files in range.
- Print condensed list: `YYYY-MM-DD - message`.
- Commit message: `feat: add dvlg list`

## 8) Implement `dvlg search <term>` (commit 8)

- Scan all YAML entries.
- Match term against `message` and `tags` (case-insensitive).
- Print matches with date and message.
- Commit message: `feat: add dvlg search`

## 9) Implement `dvlg export --week/--month --format` (commit 9)

- Support `markdown` and `text` formats.
- Aggregate entries by day.
- Include decisions section if any.
- Commit message: `feat: add dvlg export`

## 10) Implement `dvlg decisions` (commit 10)

- Scan all entries and filter `decision: true`.
- Print chronological list with date + message.
- Commit message: `feat: add dvlg decisions`

## 11) Testing + fixtures (commit 11)

- Add YAML fixtures under `tests/fixtures/`.
- Unit tests for parsing/writing entries.
- Integration tests for CLI commands (init/add/today/list/search/export/decisions).
- Commit message: `test: add dvlg fixtures and CLI tests`

## 12) Polish + release readiness (commit 12)

- Add consistent error messages and exit codes.
- Validate behavior outside git repos.
- Ensure README uses `dvlg` name everywhere.
- Commit message: `chore: polish CLI UX`
