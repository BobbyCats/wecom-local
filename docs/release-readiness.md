# Release Readiness

This document tracks the minimum credible open-source state for WeCom Local CLI.
The project is a local, read-only WeCom Desktop query capability for AI agents.
It is not an official WeCom API client.

## Current Capability Matrix

| Area | State | Evidence |
| --- | --- | --- |
| Runtime authorization status | Implemented | `wecom-local auth status --json` checks sudo timestamp state without prompting |
| Runtime authorization preparation | Implemented | `wecom-local auth prepare` warms authorization through system sudo/PAM without storing passwords |
| Runtime readiness | Implemented | `wecom-local doctor` and `wecom-local doctor --json` |
| Runtime Authorization docs | Implemented | `docs/macos-permissions.md` documents `sudo`, timestamp caching, and Touch ID |
| Conversation Discovery | Implemented | `wecom-local conversations [--query <text>]` returns stable JSON |
| Conversation Reference | Implemented | `history` and `export` accept ids or unique display-name queries |
| History query | Implemented | `wecom-local history <conversation-reference> --format json` |
| Conversation Export | Implemented | `wecom-local export <conversation-reference> --format json|markdown --output <path>` |
| Agent Skills | Implemented, intentionally thin | Codex, Claude, and Hermes Skill docs use `wc-local` as the short invocation name and call the `wecom-local` binary |
| Analysis Skills | Documented Skill workflows | `wc-brief`, `wc-scan`, `wc-audit`, `wc-style`, and `wc-draft` orchestrate CLI JSON without Runtime Bridge code |
| Members query | Implemented | `wecom-local members <conversation-reference> --format json` returns basic stable JSON by default; `--full` is explicit |
| Search query | Implemented | Reuses history read path and filters decoded rows locally |
| Stats query | Implemented | Reuses history read path and summarizes decoded rows locally |
| Member Participation | Implemented | `stats --include-members` compares member count with active senders |
| Local Store Probe | Implemented | `wecom-local store-probe --json` reports redacted file-format evidence without reading row values |
| Local Store Reader | Experimental, not implemented | macOS WeCom DB headers look wxSQLite3-like, but key acquisition and page validation are not proven |
| Runtime temporary file cleanup | Implemented | Runtime Bridge removes LLDB scripts and runtime JSON export files after attach attempts |
| Redacted ambiguity errors | Implemented | Conversation Reference ambiguity failures report counts and recovery action, not candidate names or ids |
| Contacts query | Not implemented | Defer until Runtime Bridge selector shape is clear |
| CI | Implemented locally | `.github/workflows/ci.yml` runs format, clippy, tests, build, and package file list on macOS |
| Public README | Implemented | Chinese-first `README.md` plus `README.en.md` |

## Must Complete Before Open Source

- Run non-runtime validation: `cargo fmt --check`, `cargo test`, `cargo build`,
  `cargo clippy --all-targets -- -D warnings`, and `git diff --check`.
- Run one local runtime smoke with redacted notes only.
- Confirm README, schema docs, safety docs, and Agent Skills use only synthetic
  conversation ids, names, users, and message text.
- Confirm no real chat exports, screenshots, group names, contact names, or
  conversation ids are tracked by Git.
- Confirm `.gitignore` still ignores local export locations and common
  conversation output filenames.
- Confirm repository owner, remote URL, license, package metadata, issue
  templates, security policy, and install path before publishing.
- Confirm README links the macOS Runtime Authorization guidance.
- Confirm `docs/compatibility.md` states the tested macOS and WeCom Desktop
  versions without private data.
- Confirm error messages copied into public issues do not include local
  conversation names, conversation ids, member names, or runtime export paths.
- Create the GitHub repository as private first, push the clean tree, let CI
  pass, then switch visibility to public.

## Can Defer

- `contacts` command.
- Local Store Reader implementation after a safe key-acquisition proof.
- OpenCLI plugin implementation beyond external CLI registration guidance.
- Homebrew, signed binaries, or packaged release artifacts.
- Windows, Linux, and cloud-side WeCom support.
- Background monitoring, scheduled sync, or push ingestion.

## Safety Boundary

- The Runtime Bridge is read-only.
- Local Queries must only read Local Visible Data from the signed-in WeCom
  Desktop account.
- Conversation Discovery can identify candidates, but Conversation Reference
  resolution must fail closed when a display-name query is ambiguous.
- Ambiguous Conversation Reference errors should be recoverable without printing
  candidate conversation names or ids.
- Runtime Authorization is delegated to system `sudo`/PAM. The CLI may check or
  warm the sudo timestamp, but must not store macOS passwords, create askpass
  scripts, or install privileged helpers by default.
- Agent Skills are instructions for calling `wecom-local`; they must not
  implement Runtime Bridge logic.
- Analysis Skills may summarize, audit, profile, or draft from CLI JSON, but
  they must stay scoped, local, evidence-based, and separate from Runtime
  Bridge logic.
- Conversation Export is optional and should be used only when a durable local
  artifact is necessary.
- Conversation Members uses a basic Member Detail Scope by default. Full member
  profile fields require explicit opt-in and must be treated as private runtime
  output.
- Local Store Probe may inspect file headers, page-shape bytes, and plain SQLite
  schema counts, but must not read row values, emit keys, emit memory bytes, or
  write decrypted databases.

## Privacy Red Lines

- Do not commit raw chat history.
- Do not commit real conversation ids.
- Do not commit real group names, contact names, sender names, or screenshots.
- Do not commit runtime export files.
- Do not paste private runtime output into public issues, docs, examples, or
  release notes.
- Do not publish local collaboration profiles, work scans, message drafts, or
  analysis reports generated from real WeCom data.
- Public fixtures and examples must be synthetic.

## Local Smoke Checklist

Scope: local macOS WeCom Desktop runtime only. Do not run `export`, do not take
screenshots, and do not save raw command output.

Record only status, counts, field names, and error types. Do not record message
body text, group names, contact names, sender names, timestamps, message ids,
screenshots, raw ids, or exported files.

Before running smoke commands, disable command tracing and use a redacted local
note format:

```bash
set +x
set -o pipefail
```

1. Check readiness:

   ```bash
   wecom-local auth status --json
   wecom-local auth prepare
   wecom-local doctor --json
   ```

   Record: authorization status, check names, boolean status, and error
   category. Do not record process ids or private paths.

2. Probe local store shape:

   ```bash
   wecom-local store-probe --json
   ```

   Record: database classification counts, important file names, format probe
   aggregates, schema probe counts, key/page validation status, and privacy
   booleans only. Do not record account directory names, raw paths below the
   data root, keys, memory bytes, salt bytes, page plaintext, or decrypted file
   paths.

3. Discover conversations:

   ```bash
   sudo wecom-local conversations
   ```

   Record: total count, matched count, and field names present in the first row.

4. Filter conversations:

   ```bash
   sudo wecom-local conversations --query "<redacted-query>"
   ```

   Record: whether filtering returns zero, one, or multiple candidates.

5. Read by unique Conversation Reference:

   ```bash
   sudo wecom-local history "<redacted-reference>" -n 3 --format json
   ```

   Record: exported count, total message id count, and message field names. Do
   not record any field values.

6. Verify ambiguous Conversation Reference failure:

   ```bash
   sudo wecom-local history "<redacted-ambiguous-reference>" -n 3 --format json
   ```

   Record: error type only, for example `ambiguous_conversation_reference`.

7. Verify direct id path locally without publishing the id:

   ```bash
   sudo wecom-local history "<redacted-conversation-id>" -n 1 --format json
   ```

   Record: pass or fail only.

8. List members without publishing member values:

   ```bash
   sudo wecom-local members "<redacted-reference>" --format json
   ```

   Record: member count, `member_detail_scope`, `sensitive_fields_included`,
   and member field names only. Do not record member names, accounts, phone
   numbers, email addresses, user ids, external ids, or group names. Do not run
   `--full` for public smoke notes unless the raw output is discarded and only
   field names are recorded.

9. Summarize Member Participation:

   ```bash
   sudo wecom-local stats "<redacted-reference>" --max-scan 1000 --include-members --json
   ```

   Record: `member_participation` field names and numeric counts only. Do not
   record `by_sender` values.

Pass criteria:

- `doctor` reports ready.
- `store-probe` returns valid JSON, reports privacy booleans as false for row
  values, keys, memory bytes, memory dumps, and decrypted files, and does not
  emit account directories.
- `conversations` returns valid JSON with count fields and conversation field
  names.
- `history -n 1` returns valid JSON, `exported_count <= 1`, and decoded output
  does not include `content_base64` or `raw_content_base64`.
- `members` returns valid JSON with `member_count` and a `members` array.
- default `members` reports `member_detail_scope: "basic"` and
  `sensitive_fields_included: false`.
- `stats --include-members` returns `stats.member_participation` without a
  `members` array.
- No durable chat artifact is created.
- Runtime Bridge temporary LLDB scripts and runtime JSON export files are
  cleaned up after command completion or failure.
- The smoke note contains only sanitized output.

## Non-Runtime CI Checklist

- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `cargo build`
- `cargo package --list`
- `git diff --check`

## Compatibility Risks

- WeCom Desktop runtime selectors can change across versions.
- LLDB attach permissions can differ by macOS security settings.
- WeCom app process names may differ by locale or release channel.
- Conversation Discovery currently depends on active, unblocked conversation
  selectors.
- Runtime failures should be treated as compatibility evidence, not as proof
  that no local data exists.
- `sudo` authorization is cached only for a local timestamp window; unattended
  Agent runs can fail if no TTY or prior authorization is available.
- Touch ID for `sudo` depends on local PAM configuration and macOS policy.
- Local Store Reader remains experimental: wxSQLite3-like headers and page-size
  evidence are not enough to claim direct database support without key
  acquisition and SQLite page-validation proof.

## Pre-Publish Manual Checks

- Verify `Cargo.toml` metadata matches the final repository URL.
- Verify `README.md` states that this is not an official WeCom API client.
- Verify `docs/schema.md` contains only synthetic JSON examples.
- Verify `docs/safety.md` and this file describe privacy red lines clearly.
- Verify `SECURITY.md`, `CONTRIBUTING.md`, and issue templates tell users not
  to paste private WeCom data.
- Verify Agent Skills remain thin and only call the binary.
- Verify Analysis Skills do not claim MBTI, personality diagnosis, monitoring,
  full-account sync, or automatic message sending.
- Verify generated or exported files are not tracked.

## Next Query Command Priority

1. `members <conversation-reference>`: implemented after proving the read-only
   `WEWConversationMemberDataSource.fetchConversationAllUserArr` path locally.
2. `search <query> --in <conversation-reference>`: implemented through the
   existing read-only history path and bounded JSON output.
3. `stats <conversation-reference>`: implemented through the existing
   read-only history path and aggregate JSON output. Member Participation is
   implemented through `--include-members`.
4. `contacts --query <text>`: defer until contact selectors are clear enough to
   avoid broad account scans.
