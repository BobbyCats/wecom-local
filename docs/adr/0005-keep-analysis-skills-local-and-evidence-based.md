# ADR 0005: Keep Analysis Skills Local And Evidence-Based

## Status

Accepted.

## Context

WeCom Local CLI is useful on its own as a local read-only query tool, but the
most valuable workflows come from Agent analysis on top of the JSON output:
conversation briefs, work scans, follow-up audits, collaboration profiles, and
message drafts.

Those workflows can involve sensitive workplace behavior. A whole-group scan or
person-level profile may be read-only technically, but it still has higher
privacy impact than a single `history` query.

Alternatives considered:

- Add opinionated analysis commands directly into the Rust CLI.
- Put all analysis prompts into one large generic Agent Skill.
- Create short, focused Agent Skills that call the existing CLI and keep
  analysis evidence-based.
- Make MBTI or fixed personality labels a headline feature.

## Decision

Keep higher-level analysis in short Agent Skills first. The Rust CLI remains
the local fact layer: it reads Local Visible Data and emits structured JSON.

Use short Skill names for the first analysis workflows:

- `wc-brief`: one conversation brief.
- `wc-scan`: selected conversations work scan.
- `wc-audit`: follow-up and missing-clarity audit.
- `wc-style`: local collaboration profile.
- `wc-draft`: draft the next WeCom message.

Analysis Skills must stay bounded by explicit conversation scope and message
window. They should use observed message evidence, confidence, and time window
instead of fixed personality labels. MBTI-like framing can be offered only as an
explicit local lens, not as the default public promise.

## Consequences

- The CLI stays small, auditable, and reusable by Codex, Claude, Hermes,
  OpenCLI, and other agents.
- Analysis workflows can evolve without changing Runtime Bridge selectors.
- Public examples can show practical work-chat analysis without committing real
  chats, group names, member names, or ids.
- Collaboration profiles are treated as local evidence summaries, not
  personality diagnosis.
- If the same analysis flow becomes stable and repeated enough, a future Rust
  command may expose lower-level batch JSON support, but not subjective
  judgment.
