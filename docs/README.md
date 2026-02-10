# dvlg 🦀

**A Git-backed CLI developer diary**

`dvlg` is a lightweight, open-source CLI tool that helps developers **track daily work, decisions, and progress** directly inside a Git repository.

No cloud. No database. No UI.
Just structured files + Git history.

---

## Why dvlg?

Most developers struggle to answer questions like:

- _What did I work on last week?_
- _Why did we make this architectural decision?_
- _What changed during this sprint?_

`dvlg` solves this by turning your daily work into **structured, versioned, searchable logs** that live alongside your code.

---

## Key Ideas

- 📁 **Files, not a database** — logs are plain YAML
- 🌱 **Git is the storage engine** — history, diffs, blame come for free
- ⚡ **CLI-first** — log work in seconds
- 🔍 **Searchable & exportable**
- 📴 **Works offline**
- 🦀 **Written in Rust**

---

## Installation

### From source (recommended during early development)

```bash
git clone https://github.com/yourname/dvlg
cd dvlg
cargo install --path .
```

### From prebuilt binaries (planned)

Download from GitHub Releases and place the binary in your `PATH`.

---

## Quick Start (2 minutes)

### 1. Initialize in a Git repo

```bash
cd my-project
dvlg init
```

Creates:

```text
.dvlg/
└── 2026/
```

If dvlg is not initialized, commands like `today`, `list`, `search`, `export`, and `decisions` will exit with an error and prompt you to run `dvlg init`.

---

### 2. Log your work

```bash
dvlg add "Investigated Service Bus retry behavior"
```

This will:

- Append an entry to today’s log
- Capture Git context (repo, branch, commit)
- Optionally auto-commit the log

---

### 3. View today’s progress

```bash
dvlg today
```

Example output:

```text
2026-02-09
- Investigated Service Bus retry behavior
- Added exponential backoff to worker
- Updated architecture.yaml
```

---

## Daily Usage

### Add an entry

```bash
dvlg add "Fixed retry logic in worker"
```

### List recent activity

```bash
dvlg list --week
dvlg list --month
```

### Search logs

```bash
dvlg search "service bus"
```

### Push log repo

If you configured `log_repo_path`, push its commits:

```bash
dvlg push
```

---

## Git Integration

`dvlg` is designed to **embrace Git**, not replace it.

By default:

- Logs are written to `.dvlg/YYYY/MM/DD.yaml`
- Each entry can auto-commit to Git
- Logs are branch-aware

You get:

- Full history
- Diffs over time
- Blame and attribution
- Easy backups via any Git remote

---

## Exporting Logs

Generate summaries for standups, reports, or reviews.

```bash
dvlg export --week --format markdown
dvlg export --month --format text
```

Example use cases:

- Sprint summaries
- Status updates
- Performance reviews

---

## Decisions (ADR-Lite)

Log decisions without ceremony:

```bash
dvlg add --decision \
  "Chose Service Bus over Event Grid due to ordering guarantees"
```

Later:

```bash
dvlg decisions
```

This gives you **decision history** without heavyweight ADR processes.

---

## Storage schema

Logs are stored as YAML arrays at:

```text
.dvlg/YYYY/MM/DD.yaml
```

Each entry has the following required fields:

- `timestamp` (RFC 3339)
- `message`
- `tags` (array, can be empty)
- `decision` (bool)
- `git` object: `repo`, `branch`, `commit`

Example entry:

```yaml
- timestamp: "2026-02-10T09:14:22Z"
  message: "Investigated Service Bus retry behavior"
  tags: ["service-bus", "retries"]
  decision: false
  git:
    repo: "/home/xavier511/projects/order-platform"
    branch: "main"
    commit: "a1b2c3d"
```

---

## Configuration (Optional)

Global config file:

```yaml
# ~/.config/dvlg/config.yaml
auto_commit: true
default_project: order-platform
editor: vim
log_repo_path: /path/to/dvlg-logs
log_repo_remote: https://github.com/yourname/dvlg-logs.git
```

Use `log_repo_path` to store logs (and auto-commit) in a separate Git repo. If omitted, dvlg writes to the current repo. Use `log_repo_remote` to configure the remote URL used by `dvlg push`.

### Configure via CLI

Show current config:

```bash
dvlg config show
```

Set values:

```bash
dvlg config set --log-repo-path /path/to/dvlg-logs
dvlg config set --log-repo-remote https://github.com/yourname/dvlg-logs.git
dvlg config set --auto-commit true
```

---

## What dvlg Is Not

- ❌ Not a task manager
- ❌ Not a note-taking app
- ❌ Not a cloud service
- ❌ Not AI-dependent

It’s a **developer logbook**, optimized for real engineering work.

---

## Roadmap (Planned)

- Architecture file linking
- Git hook integration
- Weekly summaries
- Optional LLM-based summarization (opt-in)
- Plugin system

---

## Philosophy

> If Git disappears tomorrow, dvlg still makes sense.
> If AI disappears tomorrow, dvlg still works.

The tool is designed to be:

- deterministic
- inspectable
- boring in the best way

---

## License

MIT

---

## Contributing

Contributions, ideas, and feedback are welcome.
Open an issue or submit a PR.

---
