# ADR 0004: Default Conversation Members To Basic Detail Scope

## Status

Accepted.

## Context

`members` can return locally visible WeCom profile fields for one conversation.
Some fields are useful for analysis, but fields such as account names, email
addresses, phone numbers, and external identifiers have a higher privacy impact
when copied into logs, prompts, issues, or durable files.

Alternatives considered:

- Return every locally visible member field by default.
- Return only basic display labels by default and require explicit opt-in for
  sensitive profile fields.
- Remove sensitive profile fields entirely.

## Decision

Use a basic **Member Detail Scope** by default for `members`. The default output
returns basic display labels and omits sensitive profile fields. A caller must
request the full **Member Detail Scope** explicitly when they need locally
visible account, contact, or external identifier fields.

## Consequences

- The default Agent workflow remains useful for roster inspection while reducing
  accidental exposure in public logs and prompts.
- Users who need complete locally visible member profile fields can still opt in
  deliberately.
- `stats --include-members` may use richer member data internally for aggregate
  matching, but it must not return member rows.
