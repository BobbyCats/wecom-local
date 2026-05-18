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

## Codex install

From the repository root:

```bash
scripts/install-codex-skills.sh
```

The script installs:

- `skills/codex` as `$CODEX_HOME/skills/wc-local`;
- `skills/wc-brief` as `$CODEX_HOME/skills/wc-brief`;
- `skills/wc-scan` as `$CODEX_HOME/skills/wc-scan`;
- `skills/wc-audit` as `$CODEX_HOME/skills/wc-audit`;
- `skills/wc-style` as `$CODEX_HOME/skills/wc-style`;
- `skills/wc-draft` as `$CODEX_HOME/skills/wc-draft`.

`CODEX_HOME` defaults to `~/.codex`. Use `--force` to replace existing local
copies. Restart Codex after installation so the new Skills appear in the list.

All analysis skills must call `wecom-local` and parse JSON. They must not
reimplement Runtime Bridge access or commit raw local WeCom data.
