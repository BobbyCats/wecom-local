---
name: wc-brief
description: Summarize one locally visible WeCom conversation window into concrete work facts, owners, open questions, risks, and the next useful question.
---

# wc-brief

Use this skill when the user wants to understand one WeCom group or chat:
what happened, who is responsible, what is unclear, and what to ask next.

This skill is an analysis wrapper around `wecom-local`. It must not implement
Runtime Bridge access.

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

4. Answer in this shape:

   - `发生了什么`: 3-7 concrete facts.
   - `已经说清楚的`: decisions, owners, dates, deliverables.
   - `没说清楚的`: missing owner, deadline, scope, evidence, or acceptance.
   - `需要追问的`: direct questions the user can send next.
   - `风险`: only risks supported by the messages.

## Rules

- Use JSON output as the evidence source.
- Keep references private. Do not commit raw messages, names, ids, exports, or
  screenshots.
- Prefer paraphrase plus message time over long raw quotes.
- Do not moralize, diagnose, or infer motive beyond the chat evidence.
- Do not write files unless the user explicitly asks for a local report. If a
  file is needed, use `.local/wecom-local/` and keep it out of Git.
