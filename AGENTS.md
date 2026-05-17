# Agent Rules

This repository is a local, read-only WeCom Desktop query CLI. Agents working
on it must preserve the public safety boundary.

## Validation

After editing Rust files, run:

```bash
cargo fmt
cargo test
cargo build
```

Before release-facing changes, also run:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo build --release
git diff --check
```

## Safety

- Keep Runtime Bridge access read-only.
- Do not commit real chat history, real conversation ids, private group names,
  contact names, screenshots, raw keys, decrypted databases, memory dumps, or
  runtime export files.
- Public examples must use synthetic data.
- Conversation Reference ambiguity must fail closed.

## Architecture Language

- `Runtime Bridge` is the local access layer to the running WeCom Desktop app.
- `Local Query` is a structured read operation against locally visible data.
- `Conversation Reference` resolves one target before message/member reads.
- `Conversation Export` is optional durable output.
- `Agent Skill` is an integration document, not runtime implementation.

## Documentation

- Keep `CONTEXT.md` limited to domain vocabulary.
- Record durable public decisions in `docs/adr/`.
- Keep Agent Skills thin; they should call the binary and parse JSON.
