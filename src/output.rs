use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde_json::Value;

use crate::auth::{AuthPrepareReport, AuthStatusReport};
use crate::doctor::DoctorReport;

#[derive(Debug, Clone, Copy)]
pub enum Format {
    Json,
    Markdown,
}

pub fn print_doctor_report(report: &DoctorReport, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(report)?);
        return Ok(());
    }

    println!("platform: {}", report.platform);
    println!("lldb: {}", check_line(report.lldb.ok, &report.lldb.detail));
    println!(
        "wecom_process: {}",
        check_line(report.wecom_process.ok, &report.wecom_process.detail)
    );
    println!(
        "container_tmp: {}",
        check_line(report.container_tmp.ok, &report.container_tmp.detail)
    );
    println!("status: {:?}", report.status);
    Ok(())
}

pub fn print_auth_status(report: &AuthStatusReport) -> Result<()> {
    println!("status: {}", report.status);
    println!("authorization_method: {}", report.authorization_method);
    println!("sudo_timestamp_cached: {}", report.sudo_timestamp_cached);
    println!("running_as_root: {}", report.running_as_root);
    println!("password_stored: {}", report.password_stored);
    println!("can_prepare: {}", report.can_prepare);
    println!("prepare_command: {}", report.prepare_command);
    println!("detail: {}", report.detail);
    Ok(())
}

pub fn print_auth_prepare(report: &AuthPrepareReport) -> Result<()> {
    println!("prepared: {}", report.prepared);
    println!("keepalive_minutes: {}", report.keepalive_minutes);
    println!(
        "keepalive_refresh_count: {}",
        report.keepalive_refresh_count
    );
    println!("password_stored: {}", report.password_stored);
    println!("status_before: {}", report.status_before.status);
    println!("status_after: {}", report.status_after.status);
    println!("detail: {}", report.status_after.detail);
    Ok(())
}

pub fn print_json(payload: &Value) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(payload)?);
    Ok(())
}

pub fn write_payload(payload: &Value, format: Format, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create output directory: {}", parent.display()))?;
    }
    let body = match format {
        Format::Json => serde_json::to_string_pretty(payload)?,
        Format::Markdown => to_markdown(payload),
    };
    fs::write(path, body).with_context(|| format!("failed to write {}", path.display()))
}

pub fn to_markdown(payload: &Value) -> String {
    let title = non_empty_str(payload, "conversation_name")
        .or_else(|| non_empty_str(payload, "conversation_id"))
        .unwrap_or("WeCom Conversation");
    let conversation_id = non_empty_str(payload, "conversation_id").unwrap_or("");
    let total = payload
        .get("total_message_ids")
        .map(Value::to_string)
        .unwrap_or_else(|| "0".to_string());
    let exported = payload
        .get("exported_count")
        .map(Value::to_string)
        .unwrap_or_else(|| "0".to_string());

    let mut lines = vec![
        format!("# {} Chat History", title),
        String::new(),
        format!("- conversation_id: `{}`", conversation_id),
        format!("- total_message_ids: `{}`", total),
        format!("- exported_count: `{}`", exported),
        String::new(),
    ];

    if let Some(rows) = payload.get("messages").and_then(Value::as_array) {
        for row in rows {
            let time = non_empty_str(row, "send_time_text").unwrap_or("");
            let id = row
                .get("message_id")
                .map(Value::to_string)
                .unwrap_or_default();
            let sender = non_empty_str(row, "sender_display").unwrap_or("");
            let text = non_empty_str(row, "display_text").unwrap_or("[empty]");
            lines.push(format!("- `{}` `{}` {}: {}", time, id, sender, text));
        }
    }

    lines.join("\n") + "\n"
}

fn check_line(ok: bool, detail: &str) -> String {
    let status = if ok { "ok" } else { "failed" };
    format!("{} ({})", status, detail)
}

fn non_empty_str<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn markdown_contains_message_rows() {
        let payload = json!({
            "conversation_name": "Synthetic Team",
            "conversation_id": "R:0000000001",
            "total_message_ids": 1,
            "exported_count": 1,
            "messages": [{
                "send_time_text": "2026-05-18 01:00:00",
                "message_id": 42,
                "sender_display": "Alice",
                "display_text": "hello"
            }]
        });
        let md = to_markdown(&payload);
        assert!(md.contains("# Synthetic Team Chat History"));
        assert!(md.contains("Alice: hello"));
    }
}
