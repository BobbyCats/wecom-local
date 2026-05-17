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
