---
name: wc-style
description: Describe observable WeCom communication habits for a person or group without turning them into MBTI or personality labels.
---

# wc-style

Use this skill when the user wants practical guidance for talking to a person
or group, based only on locally visible WeCom messages.

Default output is a collaboration profile, not MBTI, diagnosis, or a permanent
label. If the user explicitly asks for an MBTI-like lens, present it as a loose
communication lens with low confidence unless there is strong evidence.

## Authorization UX

If `sudo` authorization is needed, use an interactive system prompt in the same
terminal/TTY that will run the query. Do not ask the user to paste their macOS
password into chat. If no interactive prompt is available, stop and ask the user
to run the exact local command themselves.

## Workflow

1. Resolve the relevant conversation with `conversations`.
2. Read a bounded message window:

   ```bash
   sudo wecom-local history "Example Group" -n 300 --format json
   sudo wecom-local stats "Example Group" --max-scan 300 --include-members --json
   ```

3. If identifying a member is needed, use basic members first:

   ```bash
   sudo wecom-local members "Example Group" --format json
   ```

4. Build the profile from observable patterns:

   - message frequency in the scanned window;
   - whether replies answer the direct question;
   - whether messages contain owner, deadline, acceptance, or concrete next
     step;
   - whether the person tends to explain background before the point;
   - whether the person asks clarifying questions or waits for direction;
   - whether commitments are explicit or vague.

5. Output:

   - `观察窗口`: conversation and message/time range.
   - `能看出来的习惯`: 3-6 patterns, each with confidence.
   - `下次怎么问`: how to ask this person next time.
   - `不要过度解读`: what the evidence does not prove.

## Rules

- Do not infer protected attributes, mental health, loyalty, or intent.
- Do not call the profile a fact about the person. It is a local evidence
  summary for the scanned window.
- Do not use `members --full` unless the user explicitly asks and accepts the
  privacy impact.
- If saving a profile, write under `.local/wecom-local/profiles/` and do not
  commit it.
