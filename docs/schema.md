# Output Schema

Successful JSON commands write formatted JSON to stdout and exit with status 0.
Failures exit non-zero and currently emit human-readable errors. Do not parse
failure text as a stable machine schema.

`wecom-local auth status --json` returns the current Runtime Authorization
state without prompting for a password.

```json
{
  "platform": "macos",
  "status": "needs_authorization",
  "authorization_method": "sudo_prompt_required",
  "sudo_timestamp_cached": false,
  "running_as_root": false,
  "password_stored": false,
  "can_prepare": true,
  "prepare_command": "wecom-local auth prepare",
  "detail": "run auth prepare interactively before Agent-driven runtime queries"
}
```

`wecom-local auth prepare --json` prompts through system `sudo`/PAM when needed
and returns the status before and after preparation. The CLI never receives or
stores the macOS password.

```json
{
  "prepared": true,
  "keepalive_minutes": 0,
  "keepalive_refresh_count": 0,
  "password_stored": false,
  "status_before": {
    "platform": "macos",
    "status": "needs_authorization",
    "authorization_method": "sudo_prompt_required",
    "sudo_timestamp_cached": false,
    "running_as_root": false,
    "password_stored": false,
    "can_prepare": true,
    "prepare_command": "wecom-local auth prepare",
    "detail": "run auth prepare interactively before Agent-driven runtime queries"
  },
  "status_after": {
    "platform": "macos",
    "status": "ready",
    "authorization_method": "sudo_timestamp",
    "sudo_timestamp_cached": true,
    "running_as_root": false,
    "password_stored": false,
    "can_prepare": true,
    "prepare_command": "wecom-local auth prepare",
    "detail": "sudo timestamp is cached for this user session"
  }
}
```

`wecom-local doctor --json` returns local readiness checks.

```json
{
  "platform": "macos",
  "lldb": {
    "ok": true,
    "detail": "/usr/bin/lldb"
  },
  "wecom_process": {
    "ok": true,
    "detail": "WeWork pid=12345"
  },
  "container_tmp": {
    "ok": true,
    "detail": "/Users/example/Library/Containers/com.tencent.WeWorkMac/Data/tmp"
  },
  "status": "ready"
}
```

`wecom-local store-probe --json` returns a redacted local store capability
report. It reads database headers and plain SQLite schema counts only. It does
not read row values, emit real account directories, emit keys, or write
decrypted files.

```json
{
  "platform": "macos",
  "data_root": {
    "found": true,
    "redacted_path": "~/Library/Containers/com.tencent.WeWorkMac/Data/Library/Application Support/WXWork/Data",
    "account_dir_count": 1
  },
  "db_files": {
    "total": 10,
    "plain_sqlite": 2,
    "wxsqlite3_like_header": 8,
    "opaque_or_other": 0,
    "unreadable": 0
  },
  "important_files": [
    {
      "name": "message.db",
      "total": 1,
      "plain_sqlite": 0,
      "wxsqlite3_like_header": 1,
      "opaque_or_other": 0,
      "unreadable": 0
    }
  ],
  "format_probe": {
    "attempted": true,
    "files_checked": 10,
    "header_bytes_per_file": 24,
    "salt_prefix_bytes": 16,
    "sqlite_header_pattern_count": 2,
    "wxsqlite3_header_pattern_count": 8,
    "salt_prefix_checked_count": 8,
    "salt_prefix_nonzero_count": 8,
    "salt_prefix_all_zero_count": 0,
    "page_size_candidates": [
      {
        "page_size": 4096,
        "total": 10
      }
    ],
    "error_count": 0
  },
  "schema_probe": {
    "attempted": true,
    "sqlite3_available": true,
    "plain_sqlite_files_checked": 2,
    "plain_sqlite_files_with_schema": 2,
    "total_table_count": 6,
    "error_count": 0
  },
  "key_probe": {
    "attempted": false,
    "result": "not_attempted_by_store_probe",
    "candidate_count": 0,
    "matched_count": 0,
    "validated": false,
    "error_kind": "not_attempted_by_store_probe"
  },
  "page_validation_probe": {
    "attempted": false,
    "algorithm_label": "not_attempted",
    "page_size": null,
    "validated": false,
    "error_kind": "no_validated_key"
  },
  "privacy": {
    "row_values_read": false,
    "message_content_read": false,
    "member_values_read": false,
    "real_paths_emitted": false,
    "keys_emitted": false,
    "memory_bytes_emitted": false,
    "decrypted_files_written": false,
    "memory_dump_written": false
  }
}
```

`wecom-local conversations` returns a conversation discovery payload.

```json
{
  "query": null,
  "total_count": 2,
  "matched_count": 2,
  "conversations": [
    {
      "conversation_id": "R:0000000000",
      "conversation_name": "Example Group",
      "conversation_type": 2,
      "last_message_id": 123,
      "modify_time": 1779037200.0,
      "modify_time_text": "2026-05-18 01:00:00",
      "create_time": 1779033600.0,
      "create_time_text": "2026-05-18 00:00:00",
      "is_sticked": false,
      "is_marked": false,
      "is_blocked": false
    }
  ]
}
```

`--query <text>` filters by `conversation_id` or `conversation_name` and keeps
the same payload shape.

`history` and `export` accept the same conversation ids returned by
`conversations`. They also accept a display-name query when it resolves to one
conversation. Ambiguous display-name queries fail without reading messages.

`wecom-local history --format json` returns one conversation payload.

```json
{
  "conversation_id": "R:0000000000",
  "conversation_name": "Example Group",
  "total_message_ids": 383,
  "offset": 0,
  "exported_count": 100,
  "conversation_last_message_id": 123,
  "messages": [
    {
      "message_id": 1,
      "server_id": 1,
      "seq": 1,
      "sender_id": 1,
      "sender_display": "Alice",
      "sender_name": "Alice",
      "conversation_id": "R:0000000000",
      "content_type": 2,
      "send_time": 1779037200.0,
      "send_time_text": "2026-05-18 01:00:00",
      "text": "hello",
      "display_text": "hello",
      "is_read": true,
      "is_revoke": false,
      "has_quote_message": false
    }
  ]
}
```

Raw runtime content fields are decoded and removed from normal output.

`wecom-local members <conversation-reference> --format json` returns visible
members for one conversation.

```json
{
  "conversation_id": "R:0000000000",
  "conversation_name": "Example Group",
  "conversation_last_message_id": 123,
  "member_count": 2,
  "members": [
    {
      "user_id": 1,
      "gid": 1001,
      "corp_id": 2001,
      "display_name": "Alice",
      "name": "Alice",
      "real_name": "Alice Example",
      "rtx_name": "alice",
      "account": "alice.example",
      "colleague_remark": "",
      "name_pinyin": "alice",
      "rtx_name_pinyin": "alice",
      "email": "alice@example.invalid",
      "biz_mail": "alice@example.invalid",
      "mobile": "",
      "phone": "",
      "office_phone": "",
      "position": "Product",
      "external_company_name": "",
      "union_id": "",
      "wx_open_id": "",
      "gender": 0,
      "name_status": 0,
      "display_order": 1,
      "is_wechat_friend": false,
      "is_corp_customer": false,
      "is_user_use_nick_name": false,
      "is_associate_admin": false,
      "is_biz_mail_available": true,
      "is_wx_work_mail": true,
      "is_hide_dept": false,
      "is_email_hidden": false,
      "is_biz_mail_hidden": false,
      "is_mobile_hidden": true,
      "is_phone_hidden": true,
      "is_office_phone_hidden": true,
      "is_position_hidden": false,
      "is_external_personal_corp_hidden": false
    }
  ]
}
```

Member fields reflect what the signed-in desktop runtime exposes locally.
Fields hidden by the runtime privacy flags are returned as empty strings where
the CLI can identify the hide flag.

`wecom-local search <query> --in <conversation-reference> --json` returns a
search payload. Search scans decoded message rows from one conversation and
filters locally.

```json
{
  "conversation_id": "R:0000000000",
  "conversation_name": "Example Group",
  "query": "roadmap",
  "total_message_ids": 383,
  "scan_limit": 1000,
  "scanned_count": 1000,
  "matched_count": 2,
  "returned_count": 2,
  "conversation_last_message_id": 123,
  "messages": [
    {
      "message_id": 1,
      "sender_display": "Alice",
      "send_time_text": "2026-05-18 01:00:00",
      "text": "roadmap update",
      "display_text": "roadmap update"
    }
  ]
}
```

`wecom-local stats <conversation-reference> --json` returns a Conversation
Stats payload. Stats scans decoded message rows from one conversation and
returns aggregate counts.

```json
{
  "conversation_id": "R:0000000000",
  "conversation_name": "Example Group",
  "total_message_ids": 383,
  "scan_limit": 1000,
  "scanned_count": 1000,
  "conversation_last_message_id": 123,
  "stats": {
    "message_count": 1000,
    "text_message_count": 820,
    "non_text_message_count": 180,
    "read_count": 990,
    "unread_count": 10,
    "revoked_count": 3,
    "quoted_count": 42,
    "sender_count": 2,
    "first_send_time_text": "2026-05-18 01:00:00",
    "last_send_time_text": "2026-05-19 12:00:00",
    "by_sender": [
      {
        "sender_display": "Alice",
        "message_count": 600
      }
    ],
    "by_content_type": [
      {
        "content_type": 1,
        "message_count": 820
      }
    ],
    "by_day": [
      {
        "day": "2026-05-18",
        "message_count": 700
      }
    ]
  }
}
```

`--include-members` adds aggregate Member Participation counts without
returning the full member list.

```json
{
  "stats": {
    "sender_count": 2,
    "member_participation": {
      "member_count": 30,
      "active_sender_count": 2,
      "matched_active_member_count": 2,
      "unmatched_sender_count": 0,
      "silent_member_count": 28,
      "active_member_ratio": 0.06666666666666667
    }
  }
}
```

## Field Stability

- `conversation_id`, `conversation_name`, and `total_message_ids` are intended
  to be stable for agent JSON parsing.
- History and search payloads include `messages`; members payloads include
  `members`; stats payloads intentionally omit message rows and return
  aggregate counts instead.
- Store probe payloads include database file counts, important database file
  classifications, redacted format evidence, key/page validation status, and a
  privacy block. They intentionally omit file paths below the WeCom data root,
  key bytes, memory bytes, decrypted files, row values, and schema table names.
- History payloads include `offset` and `exported_count`; search payloads
  include `query`, `scan_limit`, `scanned_count`, `matched_count`, and
  `returned_count`.
- Members payloads include `member_count` and `members`.
- Stats payloads include `scan_limit`, `scanned_count`, and `stats`.
- Stats payloads include `stats.member_participation` only when
  `--include-members` is requested.
- `text` and `display_text` are best-effort decoded fields. Non-text messages
  may use labels such as `[media]`, `[emoji]`, or `[content_type=...]`.
- `*_time_text` fields use the local runtime timezone and are intended for
  human review. Prefer numeric timestamp fields for sorting.
- Public examples must use synthetic ids, names, users, timestamps, and message
  text.
