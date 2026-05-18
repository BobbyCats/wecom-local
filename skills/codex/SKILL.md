---
name: wecom-local
description: Query locally visible WeCom Desktop data on macOS for personal archive and AI analysis.
---

# wecom-local

Use this skill when the user asks to inspect, query, export, summarize, or
analyze locally visible WeCom Desktop data.

## Prerequisites

- macOS.
- WeCom Desktop is installed, running, and signed in.
- `wecom-local` is built or installed.
- Runtime access usually needs `sudo` because it attaches to the local
  WeCom Desktop process.

## Commands

Check readiness:

```bash
wecom-local auth status --json
wecom-local auth prepare
wecom-local doctor --json
```

Probe local store shape without reading row values:

```bash
wecom-local store-probe --json
```

List conversations:

```bash
sudo wecom-local conversations
sudo wecom-local conversations --query "example"
```

Read a conversation by id or unique display-name query as JSON:

```bash
sudo wecom-local history "R:0000000000" -n 100 --format json
sudo wecom-local history "Example Group" -n 100 --format json
```

Search decoded messages in one conversation:

```bash
sudo wecom-local search "roadmap" --in "Example Group" -n 20 --json
```

List members in one conversation:

```bash
sudo wecom-local members "Example Group" --format json
sudo wecom-local members "Example Group" --full --format json
```

Use default `members` first. Use `--full` only when the user explicitly needs
locally visible account, email, phone, or external id fields.

Summarize one conversation:

```bash
sudo wecom-local stats "Example Group" --max-scan 1000 --json
sudo wecom-local stats "Example Group" --max-scan 1000 --include-members --json
```

Export to Markdown:

```bash
sudo wecom-local export "Example Group" \
  --format markdown \
  --output ./.local/wecom-local/conversation.md
```

If a display-name query is ambiguous, run `conversations --query` and retry
with a narrower query or the returned conversation id.

## Short Analysis Skills

When installed, `wc-local` is a short alias for this data-query Skill.

Use the shorter `wc-*` skills for higher-level work:

- `wc-brief`: one conversation brief.
- `wc-scan`: selected conversations work scan.
- `wc-audit`: unanswered questions and follow-up gaps.
- `wc-style`: local evidence-based collaboration profile.
- `wc-draft`: next-message draft without sending.

These skills still call `wecom-local`; they do not implement Runtime Bridge
access.

## Safety

Only query data the signed-in user can already view locally. Do not paste raw
chat contents, real conversation ids, group names, contact names, keys, or
exported files into public issues, README examples, or commits.
