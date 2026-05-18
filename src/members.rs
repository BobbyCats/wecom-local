use anyhow::{anyhow, Result};
use serde_json::{json, Value};

const BASIC_MEMBER_FIELDS: &[&str] = &["display_name", "name", "real_name"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemberDetailScope {
    Basic,
    Full,
}

impl MemberDetailScope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Basic => "basic",
            Self::Full => "full",
        }
    }

    fn includes_sensitive_fields(self) -> bool {
        self == Self::Full
    }
}

pub fn normalize_payload(mut raw: Value, scope: MemberDetailScope) -> Result<Value> {
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
            normalize_member_row(row, scope);
        }

        rows.len()
    };

    if let Some(obj) = raw.as_object_mut() {
        obj.insert("member_count".to_string(), json!(member_count));
        obj.insert("member_detail_scope".to_string(), json!(scope.as_str()));
        obj.insert(
            "sensitive_fields_included".to_string(),
            json!(scope.includes_sensitive_fields()),
        );
        obj.remove("error");
        obj.remove("source");
    }

    Ok(raw)
}

fn normalize_member_row(row: &mut Value, scope: MemberDetailScope) {
    let Some(obj) = row.as_object_mut() else {
        return;
    };

    let display_name = display_name(obj, scope);
    if scope == MemberDetailScope::Basic
        || !display_name.is_empty()
        || !obj.contains_key("display_name")
    {
        obj.insert("display_name".to_string(), json!(display_name));
    }

    if scope == MemberDetailScope::Basic {
        obj.retain(|key, _| BASIC_MEMBER_FIELDS.contains(&key.as_str()));
    }
}

fn display_name(obj: &serde_json::Map<String, Value>, scope: MemberDetailScope) -> String {
    let fields = match scope {
        MemberDetailScope::Basic => ["real_name", "name", "", ""],
        MemberDetailScope::Full => ["real_name", "name", "rtx_name", "account"],
    };

    fields
        .into_iter()
        .filter(|key| !key.is_empty())
        .filter_map(|key| obj.get(key).and_then(Value::as_str))
        .map(str::trim)
        .find(|value| !value.is_empty())
        .unwrap_or_default()
        .to_string()
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
                    "account": "alice.internal",
                    "email": "alice@example.invalid"
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

        let result = normalize_payload(payload, MemberDetailScope::Basic).unwrap();
        assert_eq!(result["member_count"], 2);
        assert_eq!(result["member_detail_scope"], "basic");
        assert_eq!(result["sensitive_fields_included"], false);
        assert_eq!(result["members"][0]["display_name"], "Synthetic Alice");
        assert_eq!(result["members"][1]["display_name"], "Synthetic Bob");
        assert!(result["members"][0].get("user_id").is_none());
        assert!(result["members"][0].get("account").is_none());
        assert!(result["members"][0].get("email").is_none());
        assert!(result.get("error").is_none());
        assert!(result.get("source").is_none());
    }

    #[test]
    fn full_scope_preserves_sensitive_member_fields() {
        let payload = json!({
            "members": [
                {
                    "user_id": 1,
                    "name": "",
                    "real_name": "",
                    "rtx_name": "",
                    "account": "alice.internal",
                    "email": "alice@example.invalid",
                    "union_id": "synthetic-union"
                }
            ]
        });

        let result = normalize_payload(payload, MemberDetailScope::Full).unwrap();
        assert_eq!(result["member_detail_scope"], "full");
        assert_eq!(result["sensitive_fields_included"], true);
        assert_eq!(result["members"][0]["display_name"], "alice.internal");
        assert_eq!(result["members"][0]["user_id"], 1);
        assert_eq!(result["members"][0]["account"], "alice.internal");
        assert_eq!(result["members"][0]["email"], "alice@example.invalid");
        assert_eq!(result["members"][0]["union_id"], "synthetic-union");
    }

    #[test]
    fn basic_scope_does_not_fallback_to_account_for_display_name() {
        let payload = json!({
            "members": [
                {
                    "display_name": "alice.internal",
                    "name": "",
                    "real_name": "",
                    "rtx_name": "",
                    "account": "alice.internal"
                }
            ]
        });

        let result = normalize_payload(payload, MemberDetailScope::Basic).unwrap();
        assert_eq!(result["members"][0]["display_name"], "");
        assert!(result["members"][0].get("account").is_none());
    }

    #[test]
    fn rejects_runtime_error() {
        let payload = json!({
            "error": "member selector unavailable",
            "members": []
        });

        let error = normalize_payload(payload, MemberDetailScope::Basic).unwrap_err();
        assert!(error.to_string().contains("member selector unavailable"));
    }
}
