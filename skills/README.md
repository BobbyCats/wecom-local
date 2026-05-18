# Agent Skills

`wecom-local` keeps platform instructions thin. The Rust binary is the source
of truth; skills only describe how an agent should call it safely.

Current skill targets:

- `codex/SKILL.md`
- `claude/SKILL.md`
- `hermes/SKILL.md`

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
