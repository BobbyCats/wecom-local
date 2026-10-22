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

Follow `wc-local`: run the requested read-only query without sudo first. The
user's task authorizes the needed reads; do not ask again for each command.
Diagnose only after failure. `auth status` without `--probe` checks sudo cache,
not actual query access. Only explicit permission denial allows one `sudo -n`
retry of the same command. Use system authorization only if it remains required;
never request a password in chat. Keep failed-run notes private.

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

1. Discover conversations:

   ```bash
   wecom-local conversations
   wecom-local conversations --query "Example"
   ```

2. For each selected conversation, run bounded reads:

   ```bash
   wecom-local history "Example Group" -n 200 --format json
   wecom-local stats "Example Group" --max-scan 200 --include-members --json
   ```

3. Output a table or compact sections:

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
