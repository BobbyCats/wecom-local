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

## Local Store Probe

`store-probe` is allowed to inspect local WeCom database file headers and plain
SQLite schema counts. It must not:

- read message, member, contact, or conversation row values;
- print account directory names or real database paths below the WeCom data
  root;
- print or save raw encryption keys;
- write decrypted database files;
- claim that Local Store Reader support is available before key acquisition and
  page decoding are both proven.
