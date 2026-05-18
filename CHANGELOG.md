# Changelog

All notable changes to WeCom Local CLI will be recorded here.

The project follows a pragmatic pre-1.0 changelog. Versioned headings describe
tagged releases.

## Unreleased

- Added `auth status` and `auth prepare` to check and warm Runtime
  Authorization through system `sudo`/PAM without storing passwords.
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
