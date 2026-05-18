# WeCom Local CLI

[![CI](https://github.com/BobbyCats/wecom-local/actions/workflows/ci.yml/badge.svg)](https://github.com/BobbyCats/wecom-local/actions/workflows/ci.yml)
![Platform](https://img.shields.io/badge/platform-macOS-lightgrey)
![Runtime](https://img.shields.io/badge/runtime-read--only-green)
![Status](https://img.shields.io/badge/status-experimental-orange)
![License](https://img.shields.io/badge/license-Apache--2.0-blue)

Help agents read the WeCom conversations already visible on your Mac.

Work chats are often messy. A long reply may start with background, excuses, or
side issues before getting to the one sentence that matters. A group may discuss
for a long time and still leave the owner, next step, or deadline unclear.

Humans can scroll through that pain. Agents need stable, structured data.

`wecom-local` keeps the scope narrow: read locally visible data from the
signed-in macOS WeCom Desktop account and return stable JSON. It does not upload
data, send messages, call the official WeCom API, or expand account visibility.

[中文 README](README.md)

## Why This Exists

Tools like `wx-cli` have made local WeChat history useful for search, review,
and private analysis. WeCom is where much of the work conversation happens, but
the same local agent-friendly path has been missing.

Screenshots, copy/paste, and manual exports are fine once. They are not a good
foundation for repeatable agent workflows. WeCom Local CLI turns local visible
desktop data into explicit Local Queries with stable output and privacy-aware
failure modes.

Useful questions include:

- What was actually decided in this project chat?
- Which tasks have an owner, and which only got discussed?
- Who participated in the recent discussion window?
- Which issues need a follow-up today?
- Which long replies hide the actual point under background and excuses?
- Where is the conversation missing a clear owner, deadline, or next step?

The CLI only reads and structures local data. Higher-level Agent Skills should
do the actual analysis.

## How It Works With Official WeCom CLI

[WecomTeam/wecom-cli](https://github.com/WecomTeam/wecom-cli) is the official
WeCom Open Platform CLI. It covers official API workflows such as docs, smart
sheets, messages, contacts, todos, meetings, and schedules.

`wecom-local` is different. It reads locally visible WeCom Desktop conversations
from the signed-in Mac. It is for understanding what happened before deciding
what to do next.

A useful workflow is:

```text
wecom-local reads the recent N messages in a project chat
-> the agent extracts the point, owner, unanswered questions, and risks
-> official wecom-cli creates a todo, writes a sheet row, drafts a document,
   schedules a meeting, or sends a confirmation message
```

In short: `wecom-local` helps the agent understand the local chat context;
official `wecom-cli` helps it act through WeCom's supported APIs.

## Current Capabilities

| Area | Command | State |
| --- | --- | --- |
| Runtime authorization status | `wecom-local auth status --json` | Implemented |
| Runtime authorization preparation | `wecom-local auth prepare` | Implemented |
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

For Apple Silicon Macs, download the prebuilt binary from GitHub Releases:

```bash
curl -L -o wecom-local.tar.gz \
  https://github.com/BobbyCats/wecom-local/releases/download/v0.1.1/wecom-local-v0.1.1-aarch64-apple-darwin.tar.gz
tar -xzf wecom-local.tar.gz
sudo install -m 755 wecom-local-v0.1.1-aarch64-apple-darwin/wecom-local /usr/local/bin/wecom-local
```

Or build from source:

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

## Why Rust

Rust keeps the CLI small and easy to ship:

- one binary, no Node or Python runtime needed;
- low process startup overhead for repeated agent calls;
- typed error handling and stable JSON output;
- direct access to local files, processes, and macOS permissions.

Query speed still depends on WeCom Desktop runtime attach and macOS
authorization. The CLI itself is lightweight; runtime access is usually the
slow part.

## Quick Start

```bash
wecom-local auth status --json
wecom-local auth prepare
wecom-local doctor --json
wecom-local store-probe --json
sudo wecom-local conversations --query "Example"
sudo wecom-local history "Example Group" -n 100 --format json
sudo wecom-local members "Example Group" --format json
sudo wecom-local members "Example Group" --full --format json
sudo wecom-local stats "Example Group" --max-scan 1000 --include-members --json
```

`members` defaults to the basic Member Detail Scope. Use `--full` only when the
agent needs sensitive locally visible profile fields such as accounts, email,
phone, or external ids.

`conversation-reference` can be a conversation id returned by `conversations`
or a unique display-name query. Ambiguous display-name queries fail closed
instead of selecting the closest match.

## Agent Workflow

Recommended sequence:

```text
auth status -> auth prepare -> doctor -> conversations --query -> history -> members -> stats/search -> analysis
```

Agent integrations should call the binary and parse JSON. They should not
reimplement Runtime Bridge access in a Skill, plugin, or prompt.

## Safety Boundary

- Reads only Local Visible Data for the signed-in macOS WeCom Desktop account.
- Runtime Bridge stays read-only.
- Runtime commands usually require `sudo` because process attach permission is
  controlled by local macOS PAM.
- `auth status` checks the current authorization cache without prompting.
  `auth prepare` warms authorization through system `sudo`/PAM interaction.
- The CLI does not store passwords, create askpass scripts, or install a
  privileged helper.
- Public docs and tests must use synthetic data only.
- `members` returns basic fields by default. `--full` returns more locally
  visible profile fields and should not be pasted into public locations.
- `store-probe` reads database headers, page-shape bytes, and plain SQLite
  schema counts only. It does not read row values, print keys or memory bytes,
  or write decrypted databases.

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
