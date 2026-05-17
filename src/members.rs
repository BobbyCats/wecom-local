use anyhow::{anyhow, Result};
use serde_json::{json, Value};

pub fn normalize_payload(mut raw: Value) -> Result<Value> {
    if let Some(error) = raw.get("error").and_then(Value::as_str) {
        if !error.trim().is_empty() {
            return Err(anyhow!("WeCom member query failed: {}", error));
        }
    }

    let member_count = {
        let rows = raw
            .get_mut("members")
            .and_then(Value::as_array_mut)
            .ok_or_else(|| anyhow!("runtime export does not contain a members array"))?;

        for row in rows.iter_mut() {
            normalize_member_row(row);
        }

        rows.len()
    };

    if let Some(obj) = raw.as_object_mut() {
        obj.insert("member_count".to_string(), json!(member_count));
        obj.remove("error");
        obj.remove("source");
    }

    Ok(raw)
}

fn normalize_member_row(row: &mut Value) {
    let Some(obj) = row.as_object_mut() else {
        return;
    };

    if !obj.contains_key("display_name") {
        let display_name = ["real_name", "name", "rtx_name", "account"]
            .into_iter()
            .filter_map(|key| obj.get(key).and_then(Value::as_str))
            .map(str::trim)
            .find(|value| !value.is_empty())
            .unwrap_or_default()
            .to_string();
        obj.insert("display_name".to_string(), json!(display_name));
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn normalizes_member_payload() {
        let payload = json!({
            "conversation_id": "R:0000000001",
            "conversation_name": "Synthetic Team",
            "members": [
                {
                    "user_id": 1,
                    "name": "Synthetic Alice",
                    "real_name": "",
                    "rtx_name": "",
                    "account": ""
                },
                {
                    "user_id": 2,
                    "name": "",
                    "real_name": "Synthetic Bob",
                    "rtx_name": "",
                    "account": ""
                }
            ]
        });

        let result = normalize_payload(payload).unwrap();
        assert_eq!(result["member_count"], 2);
        assert_eq!(result["members"][0]["display_name"], "Synthetic Alice");
        assert_eq!(result["members"][1]["display_name"], "Synthetic Bob");
        assert!(result.get("error").is_none());
        assert!(result.get("source").is_none());
    }

    #[test]
    fn rejects_runtime_error() {
        let payload = json!({
            "error": "member selector unavailable",
            "members": []
        });

        let error = normalize_payload(payload).unwrap_err();
        assert!(error.to_string().contains("member selector unavailable"));
    }
}
