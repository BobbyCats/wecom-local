# Security Policy

## Supported Versions

`wecom-local` is currently an experimental macOS prototype. Security and
privacy fixes target the latest public release and the latest commit on `main`.
Users should prefer the newest release unless they are testing unreleased
changes from source.

## Reporting A Vulnerability

Open a private GitHub security advisory when available, or contact the
maintainer through GitHub. Do not include private WeCom data in the report.

Safe report contents:

- command name;
- platform and WeCom Desktop version;
- redacted error category;
- expected behavior;
- whether the issue affects read-only behavior, privacy redaction, or local
  authorization.

Do not include:

- real chat history;
- real conversation ids;
- private group names;
- contact names or profile fields;
- screenshots of WeCom content;
- raw encryption keys;
- decrypted databases;
- memory dumps;
- full LLDB output.

## Security Boundary

`wecom-local` reads data that the signed-in macOS WeCom Desktop account can
already view locally. It does not grant new WeCom permissions, send messages,
or write back to WeCom Desktop.

Runtime commands may need `sudo` because macOS controls process attachment
through local authorization. `auth status` and `auth prepare` may check or warm
the local sudo timestamp, but the CLI must not store passwords, create askpass
scripts, or install privileged helpers by default.

`members` uses a basic member detail scope by default. `members --full` can
return sensitive locally visible profile fields, including account, contact, and
external identifier fields, and should not be pasted into public reports.

## Local Store Research

Local Store Reader work is experimental. Research must start with no-output
proofs and must not print raw keys, key candidates, memory bytes, salt bytes,
page plaintext, write decrypted databases, or include real row values in logs,
tests, docs, or issues.
