---
name: wc-draft
description: Draft the next WeCom message from local chat context without sending it automatically.
---

# wc-draft

Use this skill when the user wants the next WeCom message to be clearer:
shorter, more concrete, and harder to dodge.

This skill drafts only. It does not send messages. If the user wants to send
through official `WecomTeam/wecom-cli`, require explicit confirmation before
calling any send action outside `wecom-local`.

## Authorization UX

If `sudo` authorization is needed, use an interactive system prompt in the same
terminal/TTY that will run the query. Do not ask the user to paste their macOS
password into chat. If no interactive prompt is available, stop and ask the user
to run the exact local command themselves.

## Workflow

1. Read the relevant context:

   ```bash
   sudo wecom-local history "Example Group" -n 100 --format json
   ```

2. Optionally use `wc-style` output if the user has an existing local profile
   or asks for style-aware wording.

3. Decide the message type:

   - `confirm`: confirm owner, scope, deadline, or acceptance.
   - `clarify`: ask a direct question when the prior reply is vague.
   - `follow_up`: ask for status after an unanswered or dropped item.
   - `deescalate`: reduce noise and bring the thread back to facts.

4. Draft 1-3 versions:

   - short direct version;
   - slightly softer version;
   - structured bullet version when the topic has multiple variables.

5. Include why the draft is shaped that way only if it helps the user choose.

## Rules

- Do not send automatically.
- Do not mention private profile labels in the outgoing message.
- Prefer concrete asks: owner, deadline, deliverable, blocker, acceptance.
- Avoid blame language unless the user explicitly asks for a harder tone.
- Do not invent facts not present in local context.
