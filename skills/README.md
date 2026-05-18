# Agent Skills

`wc-local` is the short Skill name for the `wecom-local` binary. The Rust
binary is the source of truth; skills only describe how an agent should call it
safely.

Data-query skill targets:

- `codex/SKILL.md`: Codex-facing `wc-local`.
- `claude/SKILL.md`: Claude-facing `wc-local`.
- `hermes/SKILL.md`: Hermes-facing `wc-local`.

Short analysis skills:

- `wc-brief/SKILL.md`: summarize one conversation window into facts, owners,
  open questions, risks, and next questions.
- `wc-scan/SKILL.md`: scan selected conversations with a bounded message window
  for local work overview.
- `wc-audit/SKILL.md`: find unanswered questions, vague commitments, missing
  owners, and missing deadlines.
- `wc-style/SKILL.md`: build a local evidence-based collaboration profile
  without treating MBTI or personality labels as facts.
- `wc-draft/SKILL.md`: draft the next WeCom message from local context without
  sending it automatically.

OpenCLI is documented separately under `opencli/` because the first integration
path is external CLI registration.

All analysis skills must call `wecom-local` and parse JSON. They must not
reimplement Runtime Bridge access or commit raw local WeCom data.
