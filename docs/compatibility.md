# Compatibility

This project targets macOS WeCom Desktop. Runtime selectors and local database
formats can change across WeCom Desktop versions, so compatibility evidence is
tracked explicitly.

## Current Verified Environment

Last updated: 2026-05-18.

| Area | Value |
| --- | --- |
| macOS | 26.4.1 |
| WeCom Desktop | 5.0.8 |
| WeCom build | 70666 |
| Rust toolchain used locally | rustc 1.95.0 |

Redacted local smoke has covered:

- `auth status --json`;
- `doctor --json`;
- `store-probe --json`;
- `conversations`;
- `history` by direct id and unique display-name Conversation Reference;
- expected empty, missing, and ambiguous Conversation Reference failures;
- `members` for group conversations;
- `search` with a synthetic no-match query;
- `stats`;
- `stats --include-members`.

Smoke notes record only status, counts, field names, and error categories.
They must not include message text, member values, group names, contact names,
real conversation ids, raw paths below the WeCom data root, keys, screenshots,
or exported files.

## Known Risks

- Runtime selectors can change across WeCom Desktop versions.
- `auth prepare` depends on interactive local `sudo`/PAM behavior. Sudo
  timestamps may be scoped to a terminal or TTY, so non-TTY or fresh-TTY Agent
  environments may still need their own interactive authorization path.
- Repeated attach operations can fail under LLDB/runtime conditions even when a
  single query succeeds.
- `members` depends on a read-only but UI-adjacent runtime selector.
- `members --full` can expose more locally visible profile fields than default
  basic output; compatibility notes should record field names only.
- `store-probe` proves local file shape only: header patterns, page-size
  aggregates, potential salt-prefix presence, and plain SQLite schema counts.
  It does not prove direct database readability.
- Local Store Reader remains experimental until key acquisition and page
  validation are proven without emitting secrets.

## Reporting Compatibility Results

When reporting compatibility, include only:

- macOS version;
- WeCom Desktop version and build;
- command name;
- pass/fail status;
- JSON field names present;
- aggregate counts when safe;
- redacted error category.

Do not include private runtime output or exported artifacts.
