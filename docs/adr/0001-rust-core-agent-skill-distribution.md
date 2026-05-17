# ADR 0001: Use A Rust Core With Agent Skill Distribution

## Status

Accepted.

## Context

The project needs to query locally visible WeCom Desktop data from macOS. The
core work involves process discovery, local runtime calls, optional file output,
and stable machine-readable schemas. The project also needs to be easy for AI
agents to discover and call from Codex, Claude, Hermes, OpenCLI, and similar
environments.

## Decision

Build the local query interface as a Rust single-binary CLI. Treat Agent Skills
and OpenCLI integration as distribution layers around the binary, not as the
core implementation.

## Consequences

- Users can install and run one local binary without a Node or Python runtime.
- Runtime-specific risk stays inside a small Rust module.
- Agent integrations can stay thin and platform-specific.
- OpenCLI should initially register this binary as an external CLI instead of
  reimplementing runtime access in TypeScript.
