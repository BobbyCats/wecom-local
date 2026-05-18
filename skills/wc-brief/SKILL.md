---
name: wc-brief
description: Brief one locally visible WeCom chat window: what happened, who owns it, what is missing, and what to ask next.
---

# wc-brief

Use this skill when the user wants a straight answer from one WeCom group or
chat, without reading the whole thread by hand.

This skill is an analysis wrapper around `wecom-local`. It must not implement
Runtime Bridge access.

## Authorization UX

If `sudo` authorization is needed, use an interactive system prompt in the same
terminal/TTY that will run the query. Do not ask the user to paste their macOS
password into chat. If no interactive prompt is available, stop and ask the user
to run the exact local command themselves.

## Workflow

1. Check readiness when needed:

   ```bash
   wecom-local auth status --json
   wecom-local auth prepare
   wecom-local doctor --json
   ```

2. Resolve the target conversation:

   ```bash
   sudo wecom-local conversations --query "Example Group"
   ```

   If multiple conversations match, stop and ask for a narrower reference or a
   conversation id from the local result. Do not guess.

3. Read a bounded window:

   ```bash
   sudo wecom-local history "Example Group" -n 200 --format json
   sudo wecom-local stats "Example Group" --max-scan 200 --include-members --json
   ```

   Increase `-n` / `--max-scan` only when the user asks for a larger window.

4. Answer in this shape. Keep it short unless the user asks for a report:

   - `这事卡在哪`: the concrete bottleneck.
   - `已经说清楚的`: decisions, owners, dates, deliverables.
   - `还差什么`: missing owner, deadline, scope, evidence, or acceptance.
   - `下一句怎么问`: direct questions the user can send next.
   - `风险`: only risks supported by the messages.

## Rules

- Use JSON output as the evidence source.
- Keep references private. Do not commit raw messages, names, ids, exports, or
  screenshots.
- Prefer paraphrase plus message time over long raw quotes.
- Do not moralize, diagnose, or infer motive beyond the chat evidence.
- Do not write files unless the user explicitly asks for a local report. If a
  file is needed, use `.local/wecom-local/` and keep it out of Git.
