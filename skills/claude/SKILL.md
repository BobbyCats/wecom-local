---
name: wc-local
description: "【企微本地查询】通过 wecom-local 直接查询本地可见企微聊天、成员和统计；按实际失败诊断权限，避免重复授权。"
---

# wc-local

Use this skill for read-only queries of locally visible WeCom Desktop data.
The binary is `wecom-local`; the Skill must not implement Runtime Bridge access.

## Authorization and fast path

A user's request to query or analyze local WeCom data authorizes the necessary
read-only queries and private local output within that task. Do not ask again
for each command, conversation lookup, or bounded retry. This does not authorize
sending messages, changing account visibility, or publishing private data.

1. Run the actual query as the current user, **without sudo**. If the reference
   is a known conversation id, go straight to `history`, `search`, `members`,
   or `stats`. Otherwise resolve it with `conversations --query`.
2. On success, continue immediately. Do not run `auth prepare`, `doctor`, or
   `store-probe` as mandatory preflight steps. Reuse results within this task.
3. On failure, inspect the error and run only the diagnostic that answers it.
   `doctor --json` checks environment prerequisites. `auth status --probe --json`
   tests actual access without reading chat rows. `store-probe` is only for
   questions about local store shape, never a permission recovery step.
4. Only after an explicit runtime permission denial, retry the **same command**
   once with `sudo -n` if existing authorization may apply. `-n` must not be
   removed in an unattended session. Do not run another broad scan.
5. If that fails, record the command, timestamp, failure category, and recovery
   hint privately under `.local/wecom-local/<task-slug>/` or a mode-0700 temporary
   directory. Report the actual blocker once. Do not treat it as an empty chat.

`auth status --json` without `--probe` checks **sudo cache state only**.
`needs_authorization` with `runtime_access: not_checked` is not proof that the
query needs sudo. Older binaries may omit `runtime_access`; use the direct query
as the test. `doctor: ready` also does not prove runtime access.

`auth prepare` is a recovery tool, not a routine step. Current versions first
probe direct access and return without sudo when it works. Only an explicit
permission denial may lead to interactive system sudo/PAM authorization.
Never request a password in chat or put it in commands, files, logs, environment
variables, or askpass scripts. Existing macOS grants can be reused; a Skill
cannot manufacture grants or promise that an OS prompt will never recur.

## Private output

```bash
umask 077
mkdir -p ./.local/wecom-local/<task-slug>
wecom-local conversations --query "Example Group" > ./.local/wecom-local/<task-slug>/conversations.json
```

Keep raw results out of Git and public output. For a one-off task, use a private
`mktemp -d` directory. Read back and parse saved JSON before drawing conclusions.

## Conversation resolution

1. Use a known id directly. Otherwise run `conversations --query "<keyword>"`.
2. If display-name lookup fails, use explicit task context to narrow candidates.
   List all conversations only when targeted lookup could not resolve the name.
3. Use the returned conversation id for subsequent reads. Ambiguity must fail
   closed; ask once only if the available task context cannot select a target.
4. `conversation reference did not match` is a resolution error, not an auth
   error. Resolve again instead of escalating privileges.

## Commands

```bash
wecom-local conversations --query "Example Group"
wecom-local history "R:0000000000" -n 100 --format json
wecom-local search "roadmap" --in "R:0000000000" -n 20 --max-scan 1000 --json
wecom-local stats "R:0000000000" --max-scan 1000 --json
wecom-local members "R:0000000000" --format json
wecom-local export "R:0000000000" -n 100 --format markdown --output ./.local/wecom-local/<task-slug>/conversation.md
```

Use basic members by default. Use `members --full` only when the task explicitly
needs locally visible account, email, phone, or external id fields. Start with a
bounded message window; widen it only to satisfy the requested scope. Use
`stats --include-members` when both aggregates are needed, avoiding duplicate
queries for the same window.

## Analysis and failure handling

- Paraphrase themes, decisions, owners, constraints, and open questions. Do not
  paste raw messages unless explicitly requested and appropriate for the output.
- `stats` shows senders in visible messages, not the full roster. Only successful
  `members` output supports a roster claim.
- If `stats` fails but `history` succeeds, derive the requested counts from that
  bounded history and state the window. Failure is not zero activity.
- A permission, runtime interface, or parse failure is not an empty result.
  Keep failed-run notes private and do not update public summaries from failure.
- Convert private observations to safe summaries before writing project docs.

## Related skills

`wc-brief`, `wc-scan`, `wc-audit`, `wc-style`, and `wc-draft` use this same
query-first authorization flow. They call the CLI and parse JSON; they do not
read databases directly or implement their own authorization policy.

Only query data the signed-in user can already view locally. Do not expose raw
chat, real ids, group/contact names, keys, or raw exports in public issues,
examples, repository docs, commits, or unrelated final answers.
