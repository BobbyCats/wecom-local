# WeCom Local CLI

[![CI](https://github.com/BobbyCats/wecom-local/actions/workflows/ci.yml/badge.svg)](https://github.com/BobbyCats/wecom-local/actions/workflows/ci.yml)
![Platform](https://img.shields.io/badge/platform-macOS-lightgrey)
![Runtime](https://img.shields.io/badge/runtime-read--only-green)
![Status](https://img.shields.io/badge/status-experimental-orange)
![License](https://img.shields.io/badge/license-Apache--2.0-blue)

Local read-only WeCom Desktop query CLI for humans and AI agents.

`wecom-local` lets Codex, Claude, Hermes, and similar agents query locally
visible WeCom Desktop data through stable JSON: conversations, messages,
members, search, stats, and optional exports. It does not upload data, send
messages, call the official WeCom API, or expand the signed-in account's
visibility.

[中文 README](README.md)

## Why This Exists

Official WeCom APIs are useful for bots and enterprise integrations, but they
do not cover every conversation a user can already see in the desktop app.
Agent workflows also need structured, repeatable, fail-closed access instead of
screenshots, manual copy/paste, or ad hoc export files.

WeCom Local CLI turns local visible desktop data into explicit Local Queries
with stable output and privacy-aware failure modes.

## Current Capabilities

| Area | Command | State |
| --- | --- | --- |
| Readiness | `wecom-local doctor --json` | Implemented |
| Conversation discovery | `wecom-local conversations [--query <text>]` | Implemented |
| Message history | `wecom-local history <conversation-reference>` | Implemented |
| Members | `wecom-local members <conversation-reference>` | Implemented |
| Search | `wecom-local search <query> --in <conversation-reference>` | Implemented |
| Stats | `wecom-local stats <conversation-reference>` | Implemented |
| Member participation | `wecom-local stats <conversation-reference> --include-members` | Implemented |
| Optional export | `wecom-local export <conversation-reference>` | Implemented |
| Local store probe | `wecom-local store-probe --json` | Implemented |
| Local store reader | Experimental | Not implemented |
| Contacts | `contacts` | Not implemented |

## Install

```bash
cargo build --release
```

The binary is written to:

```bash
target/release/wecom-local
```

Optionally link it into PATH:

```bash
ln -sf "$PWD/target/release/wecom-local" "$HOME/.local/bin/wecom-local"
```

## Quick Start

```bash
wecom-local doctor --json
wecom-local store-probe --json
sudo wecom-local conversations --query "Example"
sudo wecom-local history "Example Group" -n 100 --format json
sudo wecom-local members "Example Group" --format json
sudo wecom-local stats "Example Group" --max-scan 1000 --include-members --json
```

`conversation-reference` can be a conversation id returned by `conversations`
or a unique display-name query. Ambiguous display-name queries fail closed
instead of selecting the closest match.

## Agent Workflow

Recommended sequence:

```text
doctor -> conversations --query -> history -> members -> stats/search -> analysis
```

Agent integrations should call the binary and parse JSON. They should not
reimplement Runtime Bridge access in a Skill, plugin, or prompt.

## Safety Boundary

- Reads only Local Visible Data for the signed-in macOS WeCom Desktop account.
- Runtime Bridge stays read-only.
- Runtime commands usually require `sudo` because process attach permission is
  controlled by local macOS PAM.
- The CLI does not store passwords, create askpass scripts, or install a
  privileged helper.
- Public docs and tests must use synthetic data only.
- `store-probe` reads database headers and plain SQLite schema counts only. It
  does not read row values, print keys, or write decrypted databases.

See [docs/safety.md](docs/safety.md) and
[docs/macos-permissions.md](docs/macos-permissions.md).

## Documentation

- [docs/schema.md](docs/schema.md): JSON output schema.
- [docs/safety.md](docs/safety.md): safety and privacy boundary.
- [docs/macos-permissions.md](docs/macos-permissions.md): macOS authorization.
- [docs/compatibility.md](docs/compatibility.md): tested environment and risks.
- [docs/release-readiness.md](docs/release-readiness.md): release checklist.
- [CONTEXT.md](CONTEXT.md): domain glossary.

## Development

After editing Rust files:

```bash
cargo fmt
cargo test
cargo build
```

Before release:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo build --release
git diff --check
cargo package --list
```

## License

Apache-2.0. See [LICENSE](LICENSE).
