# Changelog

All notable changes to WeCom Local CLI will be recorded here.

The project follows a pragmatic pre-1.0 changelog. Versioned headings describe
tagged releases.

## Unreleased

- Added short analysis Skill docs: `wc-brief`, `wc-scan`, `wc-audit`,
  `wc-style`, and `wc-draft`.
- Renamed the Agent-facing data-query Skill invocation to `wc-local` while
  keeping the CLI binary named `wecom-local`.
- Documented that analysis Skills stay local, scoped, evidence-based, and
  separate from Runtime Bridge implementation.

## v0.1.1 - 2026-05-18

- Added `auth status` and `auth prepare` to check and warm Runtime
  Authorization through system `sudo`/PAM without storing passwords.
- Hardened Runtime Bridge temporary file cleanup so LLDB scripts and runtime
  JSON exports are removed after attach attempts.
- Redacted Conversation Reference ambiguity errors so they report counts and
  recovery guidance without listing candidate names or ids.
- Changed `members` to default to basic member fields, with `--full` required
  for sensitive locally visible profile fields.
- Extended `store-probe --json` proof output with redacted format, key, page
  validation, and privacy fields while keeping Local Store Reader disabled.

## v0.1.0 - 2026-05-18

- Added read-only Runtime Bridge queries for conversation discovery, history,
  members, search, stats, and optional export.
- Added Conversation Reference resolution by id or unique display-name query.
- Added `stats --include-members` for Member Participation aggregates.
- Added `store-probe --json` as a redacted Local Store Probe.
- Added macOS Runtime Authorization documentation.
- Added release-readiness, safety, schema, compatibility, and Agent Skill docs.
- Reduced Runtime Bridge attach churn for `stats --include-members` by reading
  history and members in one LLDB attach.
