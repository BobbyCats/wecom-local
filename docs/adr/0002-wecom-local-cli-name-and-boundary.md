# ADR 0002: Use WeCom Local CLI As The Project Identity

## Status

Accepted.

## Context

The project began as `wecom-history`, but its intended shape is closer to a
local read-only query interface like `wx-cli`: conversations, contacts, members,
history, search, stats, and optional export. The name also needs to avoid
confusion with the official `WecomTeam/wecom-cli`, which targets official WeCom
APIs and Bot workflows.

## Decision

Use **WeCom Local CLI** as the public product name and `wecom-local` as the
repository, binary, package, and Agent Skill name.

## Consequences

- `local` carries the privacy and Desktop-runtime boundary.
- `export` remains available but is not the project identity.
- The command stays short enough for direct human and Agent use:
  `wecom-local conversations`, `wecom-local history`, and future local query
  commands.
