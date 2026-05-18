## Summary

-

## Change Type

- [ ] docs-only
- [ ] Rust code
- [ ] script/tooling
- [ ] Runtime Bridge / authorization
- [ ] release/docs

## Privacy Checklist

- [ ] No real chat content.
- [ ] No real conversation ids.
- [ ] No private group, contact, or member names.
- [ ] No screenshots, exports, databases, keys, or dumps.
- [ ] Examples and fixtures are synthetic.

## Validation Checklist

- [ ] `cargo fmt --check`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo test`
- [ ] `cargo build`
- [ ] `cargo build --release` if release-facing
- [ ] `git diff --check`
- [ ] `cargo package --list`
- [ ] Script-specific checks when scripts changed

## Runtime Smoke Notes

If runtime behavior changed, include only redacted counts, field names, and
error kinds. Do not paste raw messages, names, conversation ids, member values,
screenshots, database paths, keys, memory bytes, or exports.
