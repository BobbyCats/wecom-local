---
name: wc-audit
description: Audit locally visible WeCom messages for questions nobody answered, vague commitments, missing owners, and missing deadlines.
---

# wc-audit

Use this skill when the user wants to know what was asked, promised, or left
hanging in a WeCom conversation.

## Authorization UX

Follow `wc-local`: run the requested read-only query without sudo first. The
user's task authorizes the needed reads; do not ask again for each command.
Diagnose only after failure. `auth status` without `--probe` checks sudo cache,
not actual query access. Only explicit permission denial allows one `sudo -n`
retry of the same command. Use system authorization only if it remains required;
never request a password in chat. Keep failed-run notes private.

## Workflow

1. Resolve one conversation or a small selected set with `conversations`.
2. Read a bounded window with `history`.
3. Use `search` only when the user gives keywords or when checking a known
   topic:

   ```bash
   wecom-local search "roadmap" --in "Example Group" -n 50 --json
   ```

4. For each possible follow-up item, classify it:

   - `unanswered_question`: a question was asked and no answer appears in the
     scanned window.
   - `vague_commitment`: someone agreed but did not specify delivery, owner, or
     acceptance.
   - `missing_owner`: work exists but no responsible person is clear.
   - `missing_deadline`: owner exists but timing is unclear.
   - `conflicting_status`: later messages conflict with earlier status.

5. Output:

   - short item title;
   - evidence time range;
   - current status;
   - one follow-up line the user could send.

## Rules

- Evidence must come from local JSON output.
- Do not claim a task is closed unless the scanned messages show closure.
- Do not treat silence outside the scan window as proof.
- Do not write raw exports unless the user explicitly asks.
- Keep public artifacts synthetic.
