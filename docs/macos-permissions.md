# macOS Permissions

WeCom Local CLI reads locally visible WeCom Desktop data by asking the running
desktop process through LLDB. On macOS, attaching to another running process
usually requires local administrator authorization.

This authorization is **Runtime Authorization**. It is not a WeCom login, does
not grant new account visibility, and should not be stored by `wecom-local`.

## Expected CLI Flow

Most runtime commands should be run with `sudo`:

```bash
sudo wecom-local conversations
sudo wecom-local history "Example Group" -n 100 --format json
sudo wecom-local members "Example Group" --format json
```

In an interactive terminal, `sudo` prompts for the local macOS account password
when authorization is needed. The password is handled by macOS and `sudo`;
`wecom-local` never receives it.

After a successful authorization, `sudo` normally caches the authorization for
a short timestamp window for the same user session. It is not a one-time
permanent setup. A new terminal, a later session, or an expired timestamp can
prompt again.

To warm the authorization before an Agent workflow:

```bash
wecom-local auth status --json
wecom-local auth prepare
```

`auth status` uses non-interactive `sudo -n -v` semantics and will not prompt.
It only reports whether the local authorization timestamp is already usable.

`auth prepare` delegates prompting to system `sudo`/PAM. The password or Touch
ID interaction is handled by macOS and `sudo`; `wecom-local` does not receive,
store, or log the password. For a bounded interactive session, this command can
keep the timestamp warm while it remains running:

```bash
wecom-local auth prepare --keepalive-minutes 10
```

The keepalive mode does not make authorization permanent and does not write a
password or token to disk.

## Touch ID For sudo

On macOS versions that provide `/etc/pam.d/sudo_local.template`, Touch ID for
`sudo` can be enabled through the local PAM override file. The template is
included by `/etc/pam.d/sudo` and is designed to survive system updates.

Create or edit `/etc/pam.d/sudo_local` as root and enable:

```text
auth       sufficient     pam_tid.so
```

After that, `sudo wecom-local ...` can use Touch ID where the terminal and
system policy allow it.

Do not commit machine-specific PAM files to this repository.

## Non-Interactive Agents

If an Agent runs commands without a TTY, `sudo` may fail before `wecom-local`
starts. In that case, run `wecom-local auth prepare` interactively first, or
configure a local authorization flow outside this project.

The open-source CLI should not embed passwords, write askpass scripts, or
install a privileged helper by default. A native macOS app or signed helper can
be considered later, but it is outside the first public CLI scope.
