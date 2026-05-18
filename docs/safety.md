# Safety Boundary

`wecom-local` is a local read-only query tool for data the signed-in user can
already view in WeCom Desktop.

It should not:

- access another user's account;
- fetch cloud data outside the current account's visibility;
- run as stealth monitoring software;
- upload exported chats by default;
- publish raw chat contents in examples, tests, issues, or logs.

Public examples should use synthetic data.

## Public Artifact Rules

Do not commit or publish:

- real conversation ids;
- real group names, contact names, or sender names;
- real member lists, member ids, accounts, phone numbers, email addresses, or
  external profile identifiers;
- raw command output from local runtime smoke checks;
- exported JSON or Markdown chat files;
- screenshots of WeCom Desktop or terminal output containing private data.

Runtime smoke notes should record only command status, counts, field names, and
error categories.

Runtime error output should not include raw LLDB expressions by default because
those expressions can contain local conversation references. Use explicit debug
diagnostics only for local compatibility work, and sanitize before sharing.

Conversation Reference ambiguity errors should report the error category and
candidate count, not candidate conversation names or conversation ids. Use
`wecom-local conversations --query <reference>` locally when a human needs to
inspect candidates.

Runtime Bridge temporary files, including LLDB scripts and runtime JSON export
files, should be removed after each attach attempt even when parsing or runtime
access fails.

## Runtime Authorization

Runtime Authorization is delegated to local macOS `sudo`/PAM. `auth status`
may check whether the sudo timestamp is currently cached without prompting, and
`auth prepare` may ask system `sudo` to warm that timestamp interactively. The
CLI must not read, store, log, or load macOS passwords from `.env` files,
configuration files, Agent prompts, or environment variables.

## Conversation Members

`members` defaults to the basic **Member Detail Scope**. Basic member output is
intended for roster inspection and avoids account names, email addresses, phone
numbers, and external identifiers. `members --full` may return sensitive
locally visible profile fields and should be treated like private runtime
output.

## Local Store Probe

`store-probe` is allowed to inspect local WeCom database file headers, page
shape bytes, and plain SQLite schema counts. It may report aggregate page-size
distributions, header-pattern counts, potential salt-prefix boolean counts, and
error categories. It must not:

- read message, member, contact, or conversation row values;
- print account directory names or real database paths below the WeCom data
  root;
- print or save raw encryption keys;
- print or save raw memory bytes, key candidates, salt bytes, or page plaintext;
- write decrypted database files;
- claim that Local Store Reader support is available before key acquisition and
  page decoding are both proven.
