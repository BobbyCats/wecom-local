use std::collections::{BTreeMap, BTreeSet};

use anyhow::{anyhow, Result};
use serde_json::{json, Value};

pub fn summarize_payload(mut payload: Value, scan_limit: usize) -> Result<Value> {
    summarize_payload_inner(&mut payload, scan_limit, None)?;
    Ok(payload)
}

pub fn summarize_payload_with_members(
    mut payload: Value,
    scan_limit: usize,
    members: &Value,
) -> Result<Value> {
    summarize_payload_inner(&mut payload, scan_limit, Some(members))?;
    Ok(payload)
}

fn summarize_payload_inner(
    payload: &mut Value,
    scan_limit: usize,
    members: Option<&Value>,
) -> Result<()> {
    let rows = payload
        .get_mut("messages")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| anyhow!("runtime export does not contain a messages array"))?;

    let summary = summarize_rows(rows);
    let mut stats = summary.to_json();
    if let Some(members) = members {
        if let Some(obj) = stats.as_object_mut() {
            obj.insert(
                "member_participation".to_string(),
                member_participation(&summary, members),
            );
        }
    }

    if let Some(obj) = payload.as_object_mut() {
        obj.insert("scan_limit".to_string(), limit_or_all(scan_limit));
        obj.insert("scanned_count".to_string(), json!(summary.message_count));
        obj.insert("stats".to_string(), stats);
        obj.remove("offset");
        obj.remove("exported_count");
        obj.remove("messages");
    }

    Ok(())
}

#[derive(Debug, Default)]
struct StatsSummary {
    message_count: usize,
    text_message_count: usize,
    non_text_message_count: usize,
    read_count: usize,
    unread_count: usize,
    revoked_count: usize,
    quoted_count: usize,
    senders: BTreeMap<String, usize>,
    content_types: BTreeMap<i64, usize>,
    days: BTreeMap<String, usize>,
    first_send_time: Option<(f64, String)>,
    last_send_time: Option<(f64, String)>,
}

impl StatsSummary {
    fn to_json(&self) -> Value {
        json!({
            "message_count": self.message_count,
            "text_message_count": self.text_message_count,
            "non_text_message_count": self.non_text_message_count,
            "read_count": self.read_count,
            "unread_count": self.unread_count,
            "revoked_count": self.revoked_count,
            "quoted_count": self.quoted_count,
            "sender_count": self.senders.len(),
            "first_send_time_text": self.first_send_time.as_ref().map(|(_, text)| text),
            "last_send_time_text": self.last_send_time.as_ref().map(|(_, text)| text),
            "by_sender": sender_counts(&self.senders),
            "by_content_type": content_type_counts(&self.content_types),
            "by_day": day_counts(&self.days),
        })
    }
}

fn summarize_rows(rows: &[Value]) -> StatsSummary {
    let mut summary = StatsSummary::default();
    for row in rows {
        summary.message_count += 1;

        if text_value(row, "text").is_some() {
            summary.text_message_count += 1;
        } else {
            summary.non_text_message_count += 1;
        }

        match row.get("is_read").and_then(Value::as_bool) {
            Some(true) => summary.read_count += 1,
            Some(false) => summary.unread_count += 1,
            None => {}
        }
        if row
            .get("is_revoke")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            summary.revoked_count += 1;
        }
        if row
            .get("has_quote_message")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            summary.quoted_count += 1;
        }

        let sender = text_value(row, "sender_display")
            .unwrap_or("unknown")
            .to_string();
        *summary.senders.entry(sender).or_insert(0) += 1;

        let content_type = row
            .get("content_type")
            .and_then(Value::as_i64)
            .unwrap_or(-1);
        *summary.content_types.entry(content_type).or_insert(0) += 1;

        if let Some(day) = message_day(row) {
            *summary.days.entry(day).or_insert(0) += 1;
        }

        if let Some((send_time, send_time_text)) = message_time(row) {
            update_time_range(
                &mut summary.first_send_time,
                send_time,
                &send_time_text,
                true,
            );
            update_time_range(
                &mut summary.last_send_time,
                send_time,
                &send_time_text,
                false,
            );
        }
    }
    summary
}

fn text_value<'a>(row: &'a Value, key: &str) -> Option<&'a str> {
    row.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn message_day(row: &Value) -> Option<String> {
    text_value(row, "send_time_text").and_then(|value| value.get(0..10).map(str::to_string))
}

fn message_time(row: &Value) -> Option<(f64, String)> {
    let send_time = row.get("send_time").and_then(Value::as_f64)?;
    let send_time_text = text_value(row, "send_time_text")?.to_string();
    Some((send_time, send_time_text))
}

fn update_time_range(
    current: &mut Option<(f64, String)>,
    send_time: f64,
    send_time_text: &str,
    keep_min: bool,
) {
    let should_update = current
        .as_ref()
        .map(|(current_time, _)| {
            if keep_min {
                send_time < *current_time
            } else {
                send_time > *current_time
            }
        })
        .unwrap_or(true);
    if should_update {
        *current = Some((send_time, send_time_text.to_string()));
    }
}

fn sender_counts(values: &BTreeMap<String, usize>) -> Vec<Value> {
    let mut rows = values
        .iter()
        .map(|(sender_display, message_count)| {
            json!({
                "sender_display": sender_display,
                "message_count": message_count,
            })
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        let a_count = a
            .get("message_count")
            .and_then(Value::as_u64)
            .unwrap_or_default();
        let b_count = b
            .get("message_count")
            .and_then(Value::as_u64)
            .unwrap_or_default();
        let a_sender = a
            .get("sender_display")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let b_sender = b
            .get("sender_display")
            .and_then(Value::as_str)
            .unwrap_or_default();
        b_count.cmp(&a_count).then(a_sender.cmp(b_sender))
    });
    rows
}

fn content_type_counts(values: &BTreeMap<i64, usize>) -> Vec<Value> {
    values
        .iter()
        .map(|(content_type, message_count)| {
            json!({
                "content_type": content_type,
                "message_count": message_count,
            })
        })
        .collect()
}

fn day_counts(values: &BTreeMap<String, usize>) -> Vec<Value> {
    values
        .iter()
        .map(|(day, message_count)| {
            json!({
                "day": day,
                "message_count": message_count,
            })
        })
        .collect()
}

fn member_participation(summary: &StatsSummary, members: &Value) -> Value {
    let member_rows = members
        .get("members")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let member_count = members
        .get("member_count")
        .and_then(Value::as_u64)
        .map(|value| value as usize)
        .unwrap_or(member_rows.len());

    let member_keys = member_rows
        .iter()
        .flat_map(member_identity_keys)
        .collect::<BTreeSet<_>>();
    let sender_keys = summary
        .senders
        .keys()
        .filter_map(|sender| normalized_identity_key(sender))
        .collect::<BTreeSet<_>>();
    let matched_active_member_count = sender_keys
        .iter()
        .filter(|sender| member_keys.contains(*sender))
        .count();
    let unmatched_sender_count = sender_keys
        .len()
        .saturating_sub(matched_active_member_count);
    let silent_member_count = member_count.saturating_sub(matched_active_member_count);
    let active_member_ratio = if member_count == 0 {
        Value::Null
    } else {
        json!(matched_active_member_count as f64 / member_count as f64)
    };

    json!({
        "member_count": member_count,
        "active_sender_count": summary.senders.len(),
        "matched_active_member_count": matched_active_member_count,
        "unmatched_sender_count": unmatched_sender_count,
        "silent_member_count": silent_member_count,
        "active_member_ratio": active_member_ratio,
    })
}

fn member_identity_keys(row: &Value) -> Vec<String> {
    [
        "display_name",
        "name",
        "real_name",
        "rtx_name",
        "account",
        "colleague_remark",
        "email",
        "biz_mail",
    ]
    .into_iter()
    .filter_map(|key| row.get(key).and_then(Value::as_str))
    .filter_map(normalized_identity_key)
    .collect()
}

fn normalized_identity_key(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() || value.eq_ignore_ascii_case("unknown") {
        None
    } else {
        Some(value.to_lowercase())
    }
}

fn limit_or_all(value: usize) -> Value {
    if value == 0 {
        Value::Null
    } else {
        json!(value)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn summarizes_decoded_history_payload() {
        let payload = json!({
            "conversation_id": "R:0000000001",
            "conversation_name": "Synthetic Team",
            "total_message_ids": 3,
            "offset": 0,
            "exported_count": 3,
            "conversation_last_message_id": 3,
            "messages": [
                {
                    "message_id": 1,
                    "sender_display": "Synthetic Alice",
                    "content_type": 1,
                    "send_time": 1779037200.0,
                    "send_time_text": "2026-05-18 01:00:00",
                    "text": "Alpha",
                    "is_read": true,
                    "is_revoke": false,
                    "has_quote_message": false
                },
                {
                    "message_id": 2,
                    "sender_display": "Synthetic Bob",
                    "content_type": 2,
                    "send_time": 1779123600.0,
                    "send_time_text": "2026-05-19 01:00:00",
                    "text": "",
                    "display_text": "[image]",
                    "is_read": false,
                    "is_revoke": true,
                    "has_quote_message": false
                },
                {
                    "message_id": 3,
                    "sender_display": "Synthetic Alice",
                    "content_type": 1,
                    "send_time": 1779127200.0,
                    "send_time_text": "2026-05-19 02:00:00",
                    "text": "Gamma",
                    "is_read": true,
                    "is_revoke": false,
                    "has_quote_message": true
                }
            ]
        });

        let result = summarize_payload(payload, 1000).unwrap();
        let stats = &result["stats"];
        assert_eq!(result["scan_limit"], 1000);
        assert_eq!(result["scanned_count"], 3);
        assert_eq!(stats["message_count"], 3);
        assert_eq!(stats["text_message_count"], 2);
        assert_eq!(stats["non_text_message_count"], 1);
        assert_eq!(stats["read_count"], 2);
        assert_eq!(stats["unread_count"], 1);
        assert_eq!(stats["revoked_count"], 1);
        assert_eq!(stats["quoted_count"], 1);
        assert_eq!(stats["sender_count"], 2);
        assert_eq!(stats["first_send_time_text"], "2026-05-18 01:00:00");
        assert_eq!(stats["last_send_time_text"], "2026-05-19 02:00:00");
        assert_eq!(stats["by_sender"][0]["sender_display"], "Synthetic Alice");
        assert_eq!(stats["by_sender"][0]["message_count"], 2);
        assert_eq!(stats["by_content_type"][0]["content_type"], 1);
        assert_eq!(stats["by_day"][1]["day"], "2026-05-19");
        assert!(result.get("messages").is_none());
        assert!(result.get("offset").is_none());
        assert!(result.get("exported_count").is_none());
    }

    #[test]
    fn reports_unbounded_scan_limit_as_null() {
        let payload = json!({
            "messages": []
        });

        let result = summarize_payload(payload, 0).unwrap();
        assert!(result["scan_limit"].is_null());
        assert_eq!(result["scanned_count"], 0);
        assert_eq!(result["stats"]["message_count"], 0);
    }

    #[test]
    fn summarizes_member_participation_without_returning_member_rows() {
        let payload = json!({
            "conversation_id": "R:0000000001",
            "conversation_name": "Synthetic Team",
            "messages": [
                {
                    "sender_display": "Synthetic Alice",
                    "content_type": 1,
                    "text": "Alpha"
                },
                {
                    "sender_display": "Synthetic Bob",
                    "content_type": 1,
                    "text": "Beta"
                },
                {
                    "sender_display": "Synthetic Outside",
                    "content_type": 1,
                    "text": "Gamma"
                }
            ]
        });
        let members = json!({
            "member_count": 3,
            "members": [
                {
                    "display_name": "Synthetic Alice",
                    "account": "alice"
                },
                {
                    "display_name": "Synthetic Bob",
                    "account": "bob"
                },
                {
                    "display_name": "Synthetic Carol",
                    "account": "carol"
                }
            ]
        });

        let result = summarize_payload_with_members(payload, 100, &members).unwrap();
        let participation = &result["stats"]["member_participation"];
        assert_eq!(participation["member_count"], 3);
        assert_eq!(participation["active_sender_count"], 3);
        assert_eq!(participation["matched_active_member_count"], 2);
        assert_eq!(participation["unmatched_sender_count"], 1);
        assert_eq!(participation["silent_member_count"], 1);
        assert!(participation["active_member_ratio"].as_f64().unwrap() > 0.66);
        assert!(result.get("members").is_none());
        assert!(result["stats"].get("members").is_none());
    }
}
