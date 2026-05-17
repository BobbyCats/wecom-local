---
name: wecom-local
description: Use the local wecom-local CLI to query locally visible WeCom Desktop data on macOS.
---

# wecom-local for Claude

Use this skill when a user asks to inspect, query, export, summarize, or analyze
WeCom Desktop data that is locally visible to the signed-in macOS account.

## Readiness

```bash
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
```

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

## Rules

- Treat returned chat records as private user data.
- Prefer JSON for analysis and Markdown for human review.
- Do not include real chat content in commits, issues, or public examples.
- Do not include real conversation ids, group names, contact names, keys, or
  exported files in public artifacts.
- Do not describe the tool as an official WeCom API client.
