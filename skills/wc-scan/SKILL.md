---
name: wc-scan
description: Scan selected locally visible WeCom chats for active groups, unfinished work, unanswered questions, and follow-up needs.
---

# wc-scan

Use this skill when the user wants to look across selected WeCom work groups
and find where attention is needed.

This is a scoped local scan. It is not monitoring, sync, surveillance, or a
full-account export.

## Authorization UX

If `sudo` authorization is needed, use an interactive system prompt in the same
terminal/TTY that will run the query. Do not ask the user to paste their macOS
password into chat. If no interactive prompt is available, stop and ask the user
to run the exact local command themselves.

## Scope First

Before reading messages, identify the scan scope from the user's request:

- conversation query terms or explicit conversation ids;
- time or message window, such as recent N messages per conversation;
- whether to include only group conversations;
- whether to write a local report or answer in chat only.

If the user says "all groups", still keep the scan bounded with a per-group
message limit. Use `conversations` to list candidates, then read each selected
conversation separately.

## Workflow

1. Check readiness when needed:

   ```bash
   wecom-local auth status --json
   wecom-local auth prepare
   wecom-local doctor --json
   ```

2. Discover conversations:

   ```bash
   sudo wecom-local conversations
   sudo wecom-local conversations --query "Example"
   ```

3. For each selected conversation, run bounded reads:

   ```bash
   sudo wecom-local history "Example Group" -n 200 --format json
   sudo wecom-local stats "Example Group" --max-scan 200 --include-members --json
   ```

4. Output a table or compact sections:

   - conversation label or redacted local label;
   - scanned message count;
   - active sender count and member participation when available;
   - what still looks unfinished;
   - questions nobody answered in the scanned window;
   - items missing owner or deadline;
   - one useful next action.

## Rules

- Do not run `members --full` unless the user explicitly asks for sensitive
  locally visible profile fields.
- Do not save raw conversation JSON by default.
- If a local artifact is requested, write only under `.local/wecom-local/`.
- Do not paste scan outputs into public issues, docs, examples, or commits.
- Treat repeated Runtime Bridge failures as compatibility evidence, not as
  proof that the conversation has no messages.
