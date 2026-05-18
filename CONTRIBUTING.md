# Contributing

Thanks for helping improve WeCom Local CLI.

This project is a local, read-only WeCom Desktop query CLI. The main quality
bar is privacy discipline: public changes must not include real chat content,
real conversation ids, private group names, contact names, screenshots, keys,
decrypted databases, or runtime export files.

## Development Setup

```bash
cargo build
```

Runtime commands require macOS WeCom Desktop to be running and signed in.
Commands that attach to the runtime usually need local `sudo` authorization.

## Validation

After editing Rust files, run:

```bash
cargo fmt
cargo test
cargo build
```

Before opening a pull request, run:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo build --release
git diff --check
cargo package --list
```

## Pull Request Review

Maintainers and Agents should start with:

```bash
scripts/review-pr.sh <PR_NUMBER>
```

This script is review-only. It prints PR metadata, status checks, files,
commits, author hygiene, validation results, tracked-file risk scans, sensitive
text scans, and a final conclusion. It never merges or comments on the PR.

Review rules:

- Draft PRs are not merged directly.
- Passing CI does not mean the PR is safe to merge.
- Commit author email must not expose `.local`, localhost, local paths, or a
  personal machine name.
- Runtime Bridge, `sudo`, Local Store Reader, authorization, and privacy-related
  changes need a security-first review.
- If useful changes arrive with unsafe public commit metadata, clean-land the
  diff with maintainer GitHub noreply identity instead of merging the original
  commit history.
- When reporting to a non-code owner, include the change summary, risks,
  validation run, merge recommendation, and the next step if merge is not
  recommended.

## Privacy Rules

- Use synthetic data in tests, docs, fixtures, examples, and screenshots.
- Do not paste runtime output into issues unless every value is redacted.
- Do not commit exported JSON/Markdown conversations.
- Do not commit local database files, decrypted databases, memory dumps, raw
  keys, or LLDB/debug traces.
- Record runtime smoke evidence as counts, field names, and error categories
  only.

## Architecture Rules

- `Runtime Bridge` is the read-only local access layer to WeCom Desktop.
- `Local Query` is a structured read operation over locally visible data.
- `Conversation Reference` resolution must fail closed when a display-name
  query is ambiguous.
- `Agent Skill` files teach agents how to call the binary. They must not
  implement runtime access.
- `CONTEXT.md` is a domain glossary only. Put implementation details in docs,
  ADRs, or focused implementation notes.

## Pull Request Checklist

- [ ] Rust validation passed when Rust files changed.
- [ ] Public examples use synthetic data.
- [ ] No real chat content, ids, group names, contact names, keys, DB files, or
      exports are included.
- [ ] README/schema/safety docs were updated when command behavior changed.
