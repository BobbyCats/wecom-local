---
name: wc-local
description: Short Skill name for querying locally visible WeCom Desktop data through the wecom-local CLI on macOS.
---

# wc-local for Claude

Use this skill when a user asks to inspect, query, export, summarize, or analyze
WeCom Desktop data that is locally visible to the signed-in macOS account.

The Skill name is short. The binary is still `wecom-local`.

## Readiness

```bash
wecom-local auth status --json
wecom-local auth prepare
wecom-local doctor --json
```

## Local Store Probe

```bash
wecom-local store-probe --json
```

## Discover Conversations

```bash
sudo wecom-local conversations
sudo wecom-local conversations --query "example"
```

## Read JSON

```bash
sudo wecom-local history "R:0000000000" -n 100 --format json
sudo wecom-local history "Example Group" -n 100 --format json
```

## Search JSON

```bash
sudo wecom-local search "roadmap" --in "Example Group" -n 20 --json
```

## Members JSON

```bash
sudo wecom-local members "Example Group" --format json
sudo wecom-local members "Example Group" --full --format json
```

Use default `members` first. Use `--full` only when the user explicitly needs
locally visible account, email, phone, or external id fields.

## Stats JSON

```bash
sudo wecom-local stats "Example Group" --max-scan 1000 --json
sudo wecom-local stats "Example Group" --max-scan 1000 --include-members --json
```

## Export Markdown

```bash
sudo wecom-local export "Example Group" \
  --format markdown \
  --output ./.local/wecom-local/conversation.md
```

If a display-name query is ambiguous, run `conversations --query` and retry
with a narrower query or the returned conversation id.

## Short Analysis Skills

Use the other `wc-*` skills for higher-level work:

- `wc-brief`: one conversation brief.
- `wc-scan`: selected conversations work scan.
- `wc-audit`: unanswered questions and follow-up gaps.
- `wc-style`: local evidence-based collaboration profile.
- `wc-draft`: next-message draft without sending.

These skills still call `wecom-local`. They do not implement Runtime Bridge
access or read local databases directly.

## Rules

- Treat returned chat records as private user data.
- Prefer JSON for analysis and Markdown for human review.
- Do not include real chat content in commits, issues, or public examples.
- Do not include real conversation ids, group names, contact names, keys, or
  exported files in public artifacts.
- Do not describe the tool as an official WeCom API client.
