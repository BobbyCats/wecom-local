# macOS Permissions

The Runtime Bridge reads locally visible data through the running WeCom Desktop
process. Try the requested command as the current user first:

```bash
wecom-local conversations --query "Example Group"
wecom-local history "R:0000000000" -n 100 --format json
```

A successful query is the useful access check. Do not require `auth prepare`,
`doctor`, or `store-probe` before every query. Existing system permissions can
allow direct queries even when no sudo timestamp is cached.

## Diagnose actual access

```bash
wecom-local doctor --json
wecom-local auth status --probe --json
```

`doctor` checks LLDB, the app process, and the temporary directory. Its `ready`
state does not prove a runtime attach or a successful data query.

`auth status --probe` attaches once and checks the query service without reading
messages, names, ids, or member rows. `runtime_access: verified` proves that
service was reachable by this process at that moment. It does not prove every
query selector, grant permanent permission, or validate account visibility.
macOS may still display its own debugger authorization dialog when its policy
requires one; the probe does not suppress or automatically approve that dialog.

Without `--probe`, `auth status --json` only checks the sudo timestamp. Its legacy
`status: needs_authorization` means the sudo cache is unavailable, not that a
plain query is denied. `runtime_access: not_checked` makes that distinction
explicit. Integrations must not use the sudo-only status as a query gate.

## Recover after a permission denial

Only an explicit debugger permission denial warrants a permission retry. Reuse
existing authorization without prompting by retrying the same query once:

```bash
sudo -n wecom-local conversations --query "Example Group"
```

Do not treat selector changes, invalid references, JSON failures, a stopped app,
or a generic attach failure as evidence that sudo is needed. Those failures
need their own diagnosis. Do not widen a chat scan to recover permission.

If authorization is actually required, use an interactive system prompt:

```bash
wecom-local auth prepare
```

`auth prepare` first probes direct access. If it succeeds, preparation returns
immediately without calling interactive sudo or running a keepalive loop.
Otherwise only an explicit permission denial permits sudo preparation. It first
tries the noninteractive timestamp check; a new prompt requires an interactive
stdin and stderr. App/runtime failures return an error instead of asking for a
password. A successful sudo preparation verifies the timestamp, not root runtime
access; rerun the requested query with sudo in the same terminal.

For the sudo path only, `--keepalive-minutes 10` refreshes the timestamp while
preparation remains running. Timestamps may be scoped to a terminal, TTY, or
process context. Preparation in another terminal need not authorize an Agent
session. Expiration, a new login, changed system settings, or app updates can
require authorization again.

Passwords belong only in the macOS/sudo prompt. Never send them in chat or put
them in commands, environment variables, files, logs, or askpass scripts. The
CLI does not store passwords or install privileged helpers.

## Touch ID and operating-system policy

Where macOS provides `/etc/pam.d/sudo_local.template`, a system administrator
can configure `pam_tid.so` in the local sudo PAM override. This is optional
system configuration, not a prerequisite or an action performed by the Skill.
Touch ID availability depends on the terminal and local policy.

A Skill instruction cannot grant macOS access. Apple documents debugger
entitlements and system authorization separately from application permissions:
[Debugging tool entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.cs.debugger).
The query-first flow reuses existing access; it does not change system policy,
re-sign WeCom, install a helper, or create passwordless sudo rules.
