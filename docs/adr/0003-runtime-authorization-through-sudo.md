# ADR 0003: Use sudo/PAM For Runtime Authorization

## Status

Accepted.

## Context

The Runtime Bridge needs to attach to the running macOS WeCom Desktop process.
That local process attachment usually requires administrator authorization.
The project also needs to remain a small open-source CLI that does not store
passwords, install privileged components by default, or blur the read-only
privacy boundary.

Alternatives considered:

- Require `sudo` and let macOS handle authorization through PAM.
- Build a wrapper that prompts for an administrator password through a GUI.
- Install a privileged helper or native app for process attachment.
- Try to bypass authorization requirements.

## Decision

Use `sudo` and the local macOS PAM stack as the first public Runtime
Authorization model.

Document optional Touch ID setup for `sudo` through `/etc/pam.d/sudo_local`
where supported by macOS. Do not store passwords, create askpass scripts, or
install privileged helpers in the CLI.

## Consequences

- The CLI stays small, auditable, and aligned with the read-only Runtime Bridge
  boundary.
- Users may need to authorize with their local macOS account password or Touch
  ID before runtime commands.
- `sudo` authorization is cached only for the local timestamp window and can
  expire between Agent runs.
- Non-interactive Agent environments need a prior `sudo -v` or another local
  authorization setup outside this project.
- A native macOS app or signed privileged helper remains a possible future
  distribution layer, not part of the first public CLI.
