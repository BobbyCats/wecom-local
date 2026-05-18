<h1 align="center">WeCom Local CLI</h1>

<p align="center">
  <a href="https://github.com/BobbyCats/wecom-local/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/BobbyCats/wecom-local/actions/workflows/ci.yml/badge.svg"></a>
  <img alt="Platform" src="https://img.shields.io/badge/platform-macOS-lightgrey">
  <img alt="Language" src="https://img.shields.io/badge/language-Rust-orange">
  <img alt="Runtime" src="https://img.shields.io/badge/runtime-read--only-green">
  <img alt="Status" src="https://img.shields.io/badge/status-experimental-orange">
  <img alt="License" src="https://img.shields.io/badge/license-Apache--2.0-blue">
</p>

<p align="center">
  <a href="README.md">中文 README</a>
</p>

Let agents understand locally visible WeCom work conversations.

Work chats usually do not fail because there are too few messages. They fail
because the point is scattered: progress, explanations, responsibility,
deadlines, and blockers are mixed together. After reading N messages, you may
still not know who owns what, when it is due, or where it is stuck.

`wecom-local` is a local read-only query layer for the signed-in macOS WeCom
Desktop account. It reads visible conversations, messages, members, and stats
so an agent can analyze them. It does not upload data, send messages, call the
official WeCom API, or expand account visibility.

JSON is only the machine interface. The point is not exporting chat logs; it is
giving agents a stable local query path with recoverable failures and clear
privacy boundaries.

## 30-Second Example

Ask an agent like this:

```text
Use wc-brief on "Example Project" recent N messages. Keep it short: where is
this stuck, who should reply, and what should I ask next?
```

Under the hood, it uses local read-only queries:

```bash
wecom-local history "Example Project" -n 100 --format json
wecom-local stats "Example Project" --max-scan 100 --include-members --json
```

The agent can turn the result into something like:

```text
- Blocker: launch timing is still not confirmed; the chat keeps circling
  between assets and review.
- Owner: Alice is waiting for assets. Bob said he would add them, but did not
  give a time.
- Missing answer: who signs off the final version, and whether Friday is still
  possible.
- Next question: @Bob can you send the assets by 18:00 today? If not, is the
  blocker shooting, review, or scheduling?
```

All names and content above are synthetic examples.

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

When Codex or another agent installs this project from the repository URL,
there are two layers to install:

1. the `wecom-local` binary, which performs local read-only queries;
2. the `wc-local` and `wc-*` Skills, which tell the agent how to call the binary
   safely and run higher-level analysis.

For Apple Silicon Macs, download the prebuilt binary from GitHub Releases:

```bash
curl -L -o wecom-local.tar.gz \
  https://github.com/BobbyCats/wecom-local/releases/download/v0.1.1/wecom-local-v0.1.1-aarch64-apple-darwin.tar.gz
tar -xzf wecom-local.tar.gz
sudo install -m 755 wecom-local-v0.1.1-aarch64-apple-darwin/wecom-local /usr/local/bin/wecom-local
```

To use the short Skills in Codex, run this from a checkout of this repository:

```bash
scripts/install-codex-skills.sh
```

This installs `skills/codex` as `wc-local`, then installs `wc-brief`,
`wc-scan`, `wc-audit`, `wc-style`, and `wc-draft`. Restart Codex after
installation so the new Skills appear in the list.

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

For Agent use, `sudo` authorization must happen in the same interactive
terminal/TTY that will run the runtime query. Running `auth prepare` in a
separate Terminal window may not authorize another Agent command session. An
Agent should not ask the user to paste a macOS password into chat; if no
interactive system `sudo` or Touch ID prompt is available, stop and ask the
user to run the exact local command themselves.

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

## Short Analysis Skills

The CLI binary is still `wecom-local`. Skill names use the shorter `wc-*`
prefix: `wc-local` reads local data, and the other `wc-*` skills do the actual
analysis. Shorter daily calls, same binary underneath.

| Skill | Use |
| --- | --- |
| `wc-local` | Read conversations, messages, members, search, and stats; no judgment layer |
| `wc-brief` | Turn recent N messages into what happened, who owns it, what is missing, and what to ask next |
| `wc-scan` | Scan selected chats for active groups, unfinished items, and unanswered questions |
| `wc-audit` | Find questions nobody answered, vague commitments, missing owners, and missing deadlines |
| `wc-style` | Describe observable communication habits without turning them into personality labels |
| `wc-draft` | Draft the next message from context; never send automatically |

These skills orchestrate `wecom-local` JSON commands. They do not reimplement
Runtime Bridge access and do not write exports by default.

Example prompts:

```text
Use wc-scan on "Example Project", "Example Ops", and "Example Support" recent N
messages. List active chats, unanswered questions, and unfinished items. Do
not paste message text.
```

```text
Use wc-audit on "Example Team" recent N messages. Find questions nobody
answered, vague commitments, missing owners, and missing deadlines. Give me one
follow-up line for each item.
```

## Safety Boundary

- Reads only Local Visible Data for the signed-in macOS WeCom Desktop account.
- Runtime Bridge stays read-only.
- Runtime commands usually require `sudo` because process attach permission is
  controlled by local macOS PAM.
- `auth status` checks the current authorization cache without prompting.
  `auth prepare` warms authorization through system `sudo`/PAM interaction.
- The `sudo` authorization cache may be scoped to a terminal or TTY. Agents
  should run queries in the same interactive authorization scope and must not
  ask users to paste macOS passwords into chat.
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

## Disclaimer

`wecom-local` is an unofficial open-source project. It is not affiliated with,
authorized by, endorsed by, or sponsored by Tencent, WeCom, or WeCom/WeChat Work
official teams. Names such as `Tencent`, `WeCom`, and `企业微信` are used only
to identify the compatibility target. Related trademarks, product names, and
logos belong to their respective owners.

This tool is intended only for local read-only queries against data already
visible to the signed-in WeCom Desktop account in the user's own local
environment, for personal or internal analysis. Users are responsible for
making sure their use complies with applicable laws, company policies, data
protection requirements, labor/privacy rules, and Tencent or WeCom terms.

Do not use this tool for unauthorized data access, permission bypass,
monitoring other people, bulk collection, redistribution of private content, or
any illegal, infringing, or terms-violating purpose.

The project is provided under the Apache-2.0 license on an "as is" basis. It
does not promise compatibility with every WeCom Desktop version, and the
maintainers are not responsible for data leakage, account risk, compliance
liability, or other consequences caused by use of the tool. If you believe this
project raises an infringement, security, or compliance issue, contact the
maintainers through [SECURITY.md](SECURITY.md).

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

## Contributing

Issues and pull requests are welcome, especially for:

- compatibility evidence from more WeCom Desktop and macOS versions;
- reproducible Runtime selector, authorization, or repeated attach failures;
- better Agent Skill workflows, README examples, and install experience;
- Local Store Reader safety proofs, without keys, memory dumps, decrypted
  databases, or real data.

Do not paste real chat content, real conversation ids, private group names,
contact names, screenshots, or exports into public issues or PRs. If command
output is needed, redact it down to status, counts, field names, and error
types. See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Apache-2.0. See [LICENSE](LICENSE).
