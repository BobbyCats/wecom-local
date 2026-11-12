use std::cmp::Reverse;
use std::collections::HashSet;

use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{Local, TimeZone};
use serde_json::{json, Value};

pub fn decode_payload(mut raw: Value) -> Result<Value> {
    let rows = raw
        .get_mut("messages")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| anyhow!("runtime export does not contain a messages array"))?;

    for row in rows.iter_mut() {
        decode_message_row(row)?;
    }

    rows.sort_by(|a, b| {
        let a_time = a.get("send_time").and_then(Value::as_f64).unwrap_or(0.0);
        let b_time = b.get("send_time").and_then(Value::as_f64).unwrap_or(0.0);
        let a_id = a.get("message_id").and_then(Value::as_u64).unwrap_or(0);
        let b_id = b.get("message_id").and_then(Value::as_u64).unwrap_or(0);
        a_time
            .partial_cmp(&b_time)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a_id.cmp(&b_id))
    });

    Ok(raw)
}

fn decode_message_row(row: &mut Value) -> Result<()> {
    let content_b64 = row
        .get("content_base64")
        .and_then(Value::as_str)
        .unwrap_or("");
    let content = BASE64.decode(content_b64).unwrap_or_default();
    let candidates = collect_text_candidates(&content, 0, 8);
    let mut seen = HashSet::new();
    let mut scored: Vec<(i64, String)> = candidates
        .into_iter()
        .map(|candidate| {
            let score = candidate_score(&candidate.text, candidate.depth);
            (score, clean_text_candidate(&candidate.text))
        })
        .filter(|(_, text)| !text.is_empty())
        .collect();
    scored.sort_by_key(|score| Reverse(score.0));
    let cleaned: Vec<String> = scored
        .into_iter()
        .filter_map(|(_, text)| seen.insert(text.clone()).then_some(text))
        .take(8)
        .collect();

    let best = cleaned.first().cloned().unwrap_or_default();
    let mut display = display_text_for(row, &best);
    let mut text = if looks_encoded_payload(&best) || is_url(&best) {
        String::new()
    } else {
        best
    };
    let sender_display = sender_display(row);

    if row.get("content_type").and_then(Value::as_i64) == Some(8) {
        text.clear();
        display = "[file]".to_string();
        if row.get("is_revoke").and_then(Value::as_bool) == Some(true) {
            display = "[revoked file]".to_string();
        } else {
            let raw_content = BASE64
                .decode(
                    row.get("raw_content_base64")
                        .and_then(Value::as_str)
                        .unwrap_or(""),
                )
                .unwrap_or_default();
            if let Some(attachment) = decode_file_attachment(&raw_content) {
                text = attachment["name"].as_str().unwrap_or_default().to_string();
                display = format!("[file] {}", text);
                row["attachment"] = attachment;
            } else {
                row["attachment"] = json!({"local_state": "metadata_unavailable"});
            }
        }
    }

    if let Some(obj) = row.as_object_mut() {
        obj.insert("text".to_string(), json!(text));
        obj.insert("display_text".to_string(), json!(display));
        obj.insert("sender_display".to_string(), json!(sender_display));
        if let Some(ts) = obj.get("send_time").and_then(Value::as_f64) {
            obj.insert("send_time_text".to_string(), json!(format_timestamp(ts)));
        }
        obj.remove("content_base64");
        obj.remove("raw_content_base64");
    }
    Ok(())
}

fn decode_file_attachment(data: &[u8]) -> Option<Value> {
    let mut pos = 0;
    let mut name = None;
    let mut size = None;
    let mut md5 = None;
    while pos < data.len() {
        let (tag, next) = read_varint(data, pos).ok()?;
        pos = next;
        match tag & 7 {
            0 => {
                let (value, next) = read_varint(data, pos).ok()?;
                pos = next;
                if tag >> 3 == 4 {
                    size = Some(value);
                }
            }
            2 => {
                let (length, next) = read_varint(data, pos).ok()?;
                let end = next.checked_add(usize::try_from(length).ok()?)?;
                let value = data.get(next..end)?;
                pos = end;
                match tag >> 3 {
                    2 => name = std::str::from_utf8(value).ok().map(str::to_string),
                    10 => md5 = std::str::from_utf8(value).ok().map(str::to_ascii_lowercase),
                    _ => {}
                }
            }
            1 => pos = pos.checked_add(8)?,
            5 => pos = pos.checked_add(4)?,
            _ => return None,
        }
        if pos > data.len() {
            return None;
        }
    }
    let name = name.filter(|value| {
        !value.is_empty()
            && value != "."
            && value != ".."
            && !value.chars().any(|ch| matches!(ch, '/' | '\\' | '\0'))
    })?;
    let md5 =
        md5.filter(|value| value.len() == 32 && value.bytes().all(|ch| ch.is_ascii_hexdigit()));
    Some(json!({
        "name": name,
        "size_bytes": size,
        "content_md5": md5,
        "local_path": null,
        "local_state": "unresolved"
    }))
}

#[derive(Debug)]
struct TextCandidate {
    text: String,
    depth: usize,
}

fn collect_text_candidates(data: &[u8], depth: usize, max_depth: usize) -> Vec<TextCandidate> {
    if depth > max_depth || data.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut pos = 0usize;
    let mut parsed_any = false;
    while pos < data.len() {
        let Ok((tag, next)) = read_varint(data, pos) else {
            break;
        };
        pos = next;
        match tag & 0x07 {
            0 => {
                let Ok((_, next)) = read_varint(data, pos) else {
                    break;
                };
                pos = next;
                parsed_any = true;
            }
            1 => {
                pos = pos.saturating_add(8);
                if pos > data.len() {
                    break;
                }
                parsed_any = true;
            }
            2 => {
                let Ok((size, next)) = read_varint(data, pos) else {
                    break;
                };
                pos = next;
                let size = size as usize;
                if pos + size > data.len() {
                    break;
                }
                let chunk = &data[pos..pos + size];
                pos += size;
                parsed_any = true;
                if let Some(text) = printable_utf8(chunk) {
                    out.push(TextCandidate { text, depth });
                }
                out.extend(collect_text_candidates(chunk, depth + 1, max_depth));
            }
            5 => {
                pos = pos.saturating_add(4);
                if pos > data.len() {
                    break;
                }
                parsed_any = true;
            }
            _ => break,
        }
    }
    if !parsed_any {
        if let Some(text) = printable_utf8(data) {
            out.push(TextCandidate { text, depth });
        }
    }
    out
}

fn read_varint(data: &[u8], mut pos: usize) -> Result<(u64, usize), ()> {
    let mut value = 0u64;
    let mut shift = 0u32;
    while pos < data.len() {
        let byte = data[pos];
        pos += 1;
        if shift >= 64 {
            break;
        }
        value |= ((byte & 0x7f) as u64) << shift;
        if byte & 0x80 == 0 {
            return Ok((value, pos));
        }
        shift += 7;
    }
    Err(())
}

fn printable_utf8(data: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(data).ok()?;
    if text.is_empty() {
        return None;
    }
    if text
        .chars()
        .any(|ch| ch.is_control() && ch != '\n' && ch != '\r' && ch != '\t')
    {
        return None;
    }
    Some(text.to_string())
}

fn clean_text_candidate(value: &str) -> String {
    value
        .replace('\u{2005}', " ")
        .trim()
        .trim_start_matches(|ch: char| ch.is_control())
        .trim()
        .to_string()
}

fn candidate_score(value: &str, depth: usize) -> i64 {
    let text = clean_text_candidate(value);
    if text.is_empty() {
        return -10_000;
    }
    if text.chars().all(|ch| ch.is_ascii_digit()) && text.len() >= 8 {
        return -1_000;
    }
    let cjk = text
        .chars()
        .filter(|ch| ('\u{4e00}'..='\u{9fff}').contains(ch))
        .count() as i64;
    let visible = text.chars().filter(|ch| !ch.is_whitespace()).count() as i64;
    let controls = value
        .chars()
        .filter(|ch| ch.is_control() && *ch != '\n' && *ch != '\r' && *ch != '\t')
        .count() as i64;
    let url_penalty = if is_url(&text) { 40 } else { 0 };
    cjk * 5 + visible + depth as i64 * 2 - controls * 10 - url_penalty
}

fn display_text_for(row: &Value, best: &str) -> String {
    let content_type = row
        .get("content_type")
        .and_then(Value::as_i64)
        .unwrap_or_default();
    if !best.is_empty() && !looks_encoded_payload(best) && !is_url(best) {
        return best.to_string();
    }
    if matches!(content_type, 5 | 7 | 13) || is_url(best) {
        return "[media]".to_string();
    }
    if content_type == 100001 {
        return "[emoji]".to_string();
    }
    if content_type == 1002 {
        return "[system]".to_string();
    }
    if content_type == 2 {
        return "[image]".to_string();
    }
    format!("[content_type={}]", content_type)
}

fn looks_encoded_payload(text: &str) -> bool {
    let compact: String = text.chars().filter(|ch| !ch.is_whitespace()).collect();
    if compact.len() < 16 {
        return false;
    }
    let base64ish = compact
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '/' | '=' | '_' | '-'));
    let hexish = compact.chars().all(|ch| ch.is_ascii_hexdigit()) && compact.len() >= 32;
    base64ish || hexish
}

fn is_url(text: &str) -> bool {
    text.starts_with("http://") || text.starts_with("https://")
}

fn sender_display(row: &Value) -> String {
    for key in [
        "sender_name",
        "real_sender_name",
        "wx_nick_name",
        "name",
        "normal_user_name",
    ] {
        if let Some(value) = row.get(key).and_then(Value::as_str) {
            if !value.trim().is_empty() {
                return value.to_string();
            }
        }
    }
    row.get("sender_id")
        .map(Value::to_string)
        .unwrap_or_default()
}

fn format_timestamp(ts: f64) -> String {
    let secs = ts.trunc() as i64;
    Local
        .timestamp_opt(secs, 0)
        .single()
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| secs.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_file_payload(name: &str) -> Vec<u8> {
        let mut data = vec![0x0a, 22];
        data.extend_from_slice(b"private-download-token");
        data.extend_from_slice(&[0x12, name.len() as u8]);
        data.extend_from_slice(name.as_bytes());
        data.extend_from_slice(&[0x20, 7, 0x52, 32]);
        data.extend_from_slice(b"0123456789abcdef0123456789abcdef");
        data
    }

    #[test]
    fn decodes_file_metadata_and_discards_download_credentials() {
        let mut row = json!({
            "content_type": 8,
            "raw_content_base64": BASE64.encode(synthetic_file_payload("example.md"))
        });
        decode_message_row(&mut row).unwrap();
        assert_eq!(row["text"], "example.md");
        assert_eq!(row["display_text"], "[file] example.md");
        assert_eq!(row["attachment"]["size_bytes"], 7);
        assert_eq!(
            row["attachment"]["content_md5"],
            "0123456789abcdef0123456789abcdef"
        );
        assert!(!row.to_string().contains("private-download-token"));
        assert!(row.get("raw_content_base64").is_none());
    }

    #[test]
    fn revoked_files_have_no_attachment_or_filename() {
        let mut row = json!({
            "content_type": 8,
            "is_revoke": true,
            "raw_content_base64": BASE64.encode(synthetic_file_payload("example.md"))
        });
        decode_message_row(&mut row).unwrap();
        assert_eq!(row["display_text"], "[revoked file]");
        assert!(row.get("attachment").is_none());
        assert!(!row.to_string().contains("example.md"));
    }

    #[test]
    fn rejects_unsafe_names_and_truncated_file_metadata() {
        for name in ["../example.md", "folder\\example.md", ".", "..", ""] {
            assert!(decode_file_attachment(&synthetic_file_payload(name)).is_none());
        }
        let mut truncated = synthetic_file_payload("example.md");
        truncated.pop();
        assert!(decode_file_attachment(&truncated).is_none());
    }

    #[test]
    fn decodes_synthetic_text_content() {
        let synthetic = "Synthetic decoder message";
        let mut bytes = vec![0x12, synthetic.len() as u8];
        bytes.extend_from_slice(synthetic.as_bytes());
        let candidates = collect_text_candidates(&bytes, 0, 8);
        let best = candidates
            .iter()
            .max_by_key(|candidate| candidate_score(&candidate.text, candidate.depth))
            .map(|candidate| clean_text_candidate(&candidate.text))
            .unwrap();
        assert_eq!(best, synthetic);
    }

    #[test]
    fn labels_encoded_non_text_payloads() {
        let row = json!({"content_type": 100001});
        assert_eq!(
            display_text_for(&row, "CIGABBCpq5zQBhiUhorLj4CAAyCSAg=="),
            "[emoji]"
        );
    }
}
