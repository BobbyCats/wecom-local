---
name: wc-local
description: Short alias for the wecom-local Skill. Query locally visible WeCom Desktop data through the wecom-local CLI on macOS.
---

# wc-local

Short alias for the canonical `wecom-local` Skill.

Use this when the user wants to query, inspect, summarize, or analyze locally
visible WeCom Desktop data and wants the shorter Skill name.

This Skill still calls the `wecom-local` binary. It does not rename the project,
the repository, or the CLI.

## Commands

Check readiness:

```bash
wecom-local auth status --json
wecom-local auth prepare
wecom-local doctor --json
```

Discover conversations:

```bash
sudo wecom-local conversations
sudo wecom-local conversations --query "example"
```

Read one conversation as JSON:

```bash
sudo wecom-local history "Example Group" -n 100 --format json
```

List members:

```bash
sudo wecom-local members "Example Group" --format json
sudo wecom-local members "Example Group" --full --format json
```

Use default `members` first. Use `--full` only when the user explicitly needs
locally visible account, email, phone, or external id fields.

Search and stats:

```bash
sudo wecom-local search "roadmap" --in "Example Group" -n 20 --json
sudo wecom-local stats "Example Group" --max-scan 1000 --include-members --json
```

## Related Short Skills

- `wc-brief`: one conversation brief.
- `wc-scan`: selected conversations work scan.
- `wc-audit`: unanswered questions and follow-up gaps.
- `wc-style`: local evidence-based collaboration profile.
- `wc-draft`: next-message draft without sending.

## Safety

- Query only data the signed-in user can already view locally.
- Do not paste raw chats, real ids, real group names, contact names, member
  values, keys, screenshots, or exports into public artifacts.
- Do not reimplement Runtime Bridge access in a prompt or plugin.
