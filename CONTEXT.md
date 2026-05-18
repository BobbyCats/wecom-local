# Context

Domain language for WeCom Local CLI, a local read-only query interface over
WeCom Desktop data visible to the signed-in user.

## Language

**Local Visible Data**:
Data that the current signed-in WeCom Desktop account can already view or has
cached on this machine.
_Avoid_: cloud archive, remote history

**Runtime Bridge**:
A read-only access layer that asks the running WeCom Desktop client for local
visible data.
_Avoid_: scraper, bypass, sync agent

**Runtime Authorization**:
Local macOS permission needed before the Runtime Bridge can attach to the
running WeCom Desktop process.
_Avoid_: WeCom login, account token

**Local Query**:
A structured read operation against local visible data.
_Avoid_: export, scrape

**Conversation Discovery**:
A local query that returns candidate conversations before a specific
conversation is read.
_Avoid_: full sync, account scan

**Conversation Reference**:
A user-supplied conversation id or display-name query that must resolve to one
conversation before messages are read.
_Avoid_: chat name, search term

**Conversation Export**:
The act of writing one WeCom conversation to a durable local file such as JSON
or Markdown.
_Avoid_: backup, archive sync

**Conversation Stats**:
A Local Query that summarizes decoded messages from one conversation into
counts, buckets, and scan metadata without writing a durable export.
_Avoid_: analytics sync, monitoring

**Conversation Members**:
A Local Query that lists locally visible members for one conversation.
_Avoid_: directory sync, contact scrape

**Member Detail Scope**:
The privacy scope used when returning **Conversation Members**.
_Avoid_: full dump, profile export

**Member Participation**:
A Conversation Stats view that compares the visible member count with the
senders found in a scanned message window.
_Avoid_: group consensus, attendance

**Local Store Reader**:
An experimental read path that would query WeCom Desktop's local encrypted data
files directly after proving a safe local key and page-decoding path.
_Avoid_: cloud backup, database dump

**Local Store Probe**:
A Local Query that reports local database file format evidence without reading
row values or trying to acquire encryption keys.
_Avoid_: decryptor, dump tool

**Compatibility Evidence**:
Redacted proof that a Local Query works or fails on a specific local WeCom
Desktop environment, recorded as status, counts, field names, and error types.
_Avoid_: runtime log, chat transcript

**Agent Skill**:
A small instruction file that teaches an AI agent how to call `wecom-local`.
Agent Skills do not implement runtime access.
_Avoid_: plugin, exporter

**Analysis Skill**:
An Agent Skill that combines one or more Local Queries to produce a private
work summary, follow-up audit, communication draft, or collaboration profile.
_Avoid_: monitor, classifier, surveillance

**Work Scan**:
A bounded Analysis Skill over selected conversations or message windows.
_Avoid_: full account sync, background monitor

**Collaboration Profile**:
A local, evidence-based summary of observable communication patterns within a
specific message window.
_Avoid_: MBTI result, personality diagnosis, permanent label

## Relationships

- A **Runtime Bridge** serves one or more **Local Queries**.
- **Runtime Authorization** belongs to the local machine and does not grant new
  WeCom account visibility.
- **Conversation Discovery** identifies conversations that can be used by later
  **Local Queries**.
- A **Conversation Reference** is resolved by **Conversation Discovery** unless
  it is already a conversation id.
- A **Conversation Export** is optional output from a **Local Query**.
- **Conversation Members** resolves one **Conversation Reference** before
  reading the member list.
- **Member Detail Scope** decides whether **Conversation Members** returns only
  basic display labels or also sensitive profile fields.
- **Conversation Stats** reuses a conversation-scoped **Local Query** and does
  not require a new **Runtime Bridge** selector.
- **Member Participation** uses **Conversation Members** and **Conversation
  Stats** to avoid treating total group size as active speaker count.
- A **Local Store Reader** is an alternative local read path only after its
  safety and compatibility are proven.
- A **Local Store Probe** can inform whether a **Local Store Reader** is worth
  prototyping, but it is not a decryption path.
- **Compatibility Evidence** supports release readiness without exposing
  private values.
- An **Agent Skill** calls WeCom Local CLI instead of reimplementing the
  **Runtime Bridge**.
- An **Analysis Skill** uses **Local Queries** as evidence and keeps analysis
  outside the **Runtime Bridge**.
- A **Work Scan** must stay bounded by conversation scope and message window.
- A **Collaboration Profile** describes observed communication patterns, not a
  fixed personality type.

## Example Dialogue

> **Dev:** "Should the agent export the group first?"
> **Domain expert:** "No. Run a **Local Query** first. Use **Conversation
> Export** only if we need a durable file."

## Flagged Ambiguities

- "exporter" previously described the whole project. Resolved: export is only
  one optional output mode; the project is a local read-only query CLI.
