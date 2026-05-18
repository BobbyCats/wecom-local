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
- Ambiguity errors must not print candidate conversation names or ids.

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

## Skill Routing

Even when the user does not name a Skill explicitly, choose the smallest
project-appropriate Skill set for the work:

- Before architecture, release, or feature implementation: use
  `$plan-eng-review` to lock module boundaries, data flow, tests, and gates.
- When a plan needs to be challenged against `CONTEXT.md`, ADRs, domain terms,
  or safety language: use `$grill-with-docs`.
- When the codebase needs deeper Modules, clearer Interfaces, better Seams, or
  more testable Adapters: use `$improve-codebase-architecture`.
- For privacy, sudo, Runtime Bridge, Local Store Probe, Local Store Reader,
  release hygiene, or open-source leakage risk: use `$cso`.
- For README, safety, schema, release readiness, changelog, or Agent Skill
  alignment: use `$document-release`.
- Before merging or publishing a diff: use `$review`.
- For final validation, versioning, commit, push, and PR/release flow: use
  `$ship`.
- For long-running or interrupted work: use `$context-save` and
  `$context-restore`.

Project-specific default: if a task touches WeCom local data, Runtime Bridge,
Local Store Reader, authorization, or public release readiness, proactively use
the relevant Skill instead of waiting for the user to name it.

When reporting work to the user, explicitly state which Skills were used in
that response and what each Skill contributed.
