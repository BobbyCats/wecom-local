use anyhow::{anyhow, Result};
use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedConversation {
    pub conversation_id: String,
    pub conversation_name: String,
}

impl ResolvedConversation {
    pub fn from_id(conversation_id: &str) -> Result<Self> {
        let conversation_id = conversation_id.trim();
        if conversation_id.is_empty() {
            return Err(anyhow!("conversation reference cannot be empty"));
        }

        Ok(Self {
            conversation_id: conversation_id.to_string(),
            conversation_name: String::new(),
        })
    }
}

pub fn normalize_payload(mut raw: Value, query: Option<&str>) -> Result<Value> {
    if let Some(error) = raw.get("error").and_then(Value::as_str) {
        if !error.trim().is_empty() {
            return Err(anyhow!("WeCom conversation discovery failed: {}", error));
        }
    }

    let query = query.map(str::trim).filter(|value| !value.is_empty());
    let total_count = raw
        .get("total_count")
        .and_then(Value::as_u64)
        .unwrap_or_else(|| {
            raw.get("conversations")
                .and_then(Value::as_array)
                .map(|rows| rows.len() as u64)
                .unwrap_or(0)
        });

    let matched_count = {
        let rows = raw
            .get_mut("conversations")
            .and_then(Value::as_array_mut)
            .ok_or_else(|| anyhow!("runtime export does not contain a conversations array"))?;

        for row in rows.iter_mut() {
            normalize_row(row);
        }

        if let Some(query) = query {
            rows.retain(|row| matches_query(row, query));
        }

        rows.len()
    };

    if let Some(obj) = raw.as_object_mut() {
        obj.insert("query".to_string(), json!(query));
        obj.insert("total_count".to_string(), json!(total_count));
        obj.insert("matched_count".to_string(), json!(matched_count));
        obj.remove("error");
    }

    Ok(raw)
}

pub fn looks_like_conversation_id(value: &str) -> bool {
    let value = value.trim();
    (value.starts_with("R:") || value.starts_with("S:"))
        && value.len() > 2
        && !value.chars().any(char::is_whitespace)
}

pub fn resolve_one(payload: &Value, reference: &str) -> Result<ResolvedConversation> {
    let reference = reference.trim();
    if reference.is_empty() {
        return Err(anyhow!("conversation reference cannot be empty"));
    }

    let candidates = candidates_from_payload(payload);
    if candidates.is_empty() {
        return Err(anyhow!(
            "conversation reference '{}' did not match any locally visible conversation. Run `wecom-local conversations --query \"{}\"` to inspect candidates.",
            reference,
            reference
        ));
    }

    let exact_id_matches: Vec<_> = candidates
        .iter()
        .filter(|candidate| candidate.id_matches(reference))
        .cloned()
        .collect();

    match exact_id_matches.len() {
        1 => return Ok(exact_id_matches[0].clone()),
        2.. => {
            return Err(ambiguous_reference_error(reference, &exact_id_matches));
        }
        0 => {}
    }

    let exact_name_matches: Vec<_> = candidates
        .iter()
        .filter(|candidate| candidate.name_matches(reference))
        .cloned()
        .collect();

    match exact_name_matches.len() {
        1 => return Ok(exact_name_matches[0].clone()),
        2.. => {
            return Err(ambiguous_reference_error(reference, &exact_name_matches));
        }
        0 => {}
    }

    if candidates.len() == 1 {
        return Ok(candidates[0].clone());
    }

    Err(ambiguous_reference_error(reference, &candidates))
}

fn normalize_row(row: &mut Value) {
    let Some(obj) = row.as_object_mut() else {
        return;
    };

    if !obj.contains_key("conversation_name") {
        let name = obj
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        obj.insert("conversation_name".to_string(), json!(name));
    }

    obj.remove("name");
}

fn candidates_from_payload(payload: &Value) -> Vec<ResolvedConversation> {
    payload
        .get("conversations")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|row| {
            let conversation_id = row
                .get("conversation_id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .trim();
            if conversation_id.is_empty() {
                return None;
            }

            Some(ResolvedConversation {
                conversation_id: conversation_id.to_string(),
                conversation_name: row
                    .get("conversation_name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .trim()
                    .to_string(),
            })
        })
        .collect()
}

fn ambiguous_reference_error(
    reference: &str,
    candidates: &[ResolvedConversation],
) -> anyhow::Error {
    let rendered = candidates
        .iter()
        .take(5)
        .map(render_candidate)
        .collect::<Vec<_>>()
        .join(", ");
    anyhow!(
        "conversation reference '{}' matched {} conversations; refine it with `wecom-local conversations --query \"{}\"`. Candidates: {}",
        reference,
        candidates.len(),
        reference,
        rendered
    )
}

fn render_candidate(candidate: &ResolvedConversation) -> String {
    if candidate.conversation_name.is_empty() {
        return candidate.conversation_id.clone();
    }

    format!(
        "{} ({})",
        candidate.conversation_name, candidate.conversation_id
    )
}

fn normalized_match_key(value: &str) -> String {
    value.trim().to_lowercase()
}

impl ResolvedConversation {
    fn id_matches(&self, reference: &str) -> bool {
        let reference = normalized_match_key(reference);
        normalized_match_key(&self.conversation_id) == reference
    }

    fn name_matches(&self, reference: &str) -> bool {
        let reference = normalized_match_key(reference);
        !self.conversation_name.is_empty()
            && normalized_match_key(&self.conversation_name) == reference
    }
}

fn matches_query(row: &Value, query: &str) -> bool {
    let query = query.to_lowercase();
    ["conversation_id", "conversation_name"]
        .into_iter()
        .filter_map(|key| row.get(key).and_then(Value::as_str))
        .map(str::to_lowercase)
        .any(|value| value.contains(&query))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn normalizes_unfiltered_conversation_payload() {
        let raw = json!({
            "total_count": 2,
            "matched_count": 2,
            "query": "",
            "conversations": [
                {
                    "conversation_id": "R:0000000001",
                    "name": "Synthetic Alpha",
                    "conversation_type": 2
                },
                {
                    "conversation_id": "R:0000000002",
                    "conversation_name": "Synthetic Beta",
                    "conversation_type": 2
                }
            ]
        });

        let payload = normalize_payload(raw, None).unwrap();
        assert_eq!(payload["query"], Value::Null);
        assert_eq!(payload["total_count"], 2);
        assert_eq!(payload["matched_count"], 2);
        assert_eq!(
            payload["conversations"][0]["conversation_name"],
            "Synthetic Alpha"
        );
        assert!(payload["conversations"][0].get("name").is_none());
    }

    #[test]
    fn filters_conversations_by_name_or_id() {
        let raw = json!({
            "total_count": 3,
            "matched_count": 3,
            "conversations": [
                {
                    "conversation_id": "R:0000000001",
                    "conversation_name": "Synthetic Alpha"
                },
                {
                    "conversation_id": "R:0000000002",
                    "conversation_name": "Synthetic Beta"
                },
                {
                    "conversation_id": "R:0000000003",
                    "conversation_name": "Synthetic Query"
                }
            ]
        });

        let payload = normalize_payload(raw, Some("query")).unwrap();
        let rows = payload["conversations"].as_array().unwrap();
        assert_eq!(payload["query"], "query");
        assert_eq!(payload["total_count"], 3);
        assert_eq!(payload["matched_count"], 1);
        assert_eq!(rows[0]["conversation_id"], "R:0000000003");
    }

    #[test]
    fn resolves_single_fuzzy_conversation_reference() {
        let payload = json!({
            "conversations": [
                {
                    "conversation_id": "R:0000000001",
                    "conversation_name": "Synthetic Alpha"
                }
            ]
        });

        let resolved = resolve_one(&payload, "Alpha").unwrap();
        assert_eq!(resolved.conversation_id, "R:0000000001");
        assert_eq!(resolved.conversation_name, "Synthetic Alpha");
    }

    #[test]
    fn resolves_exact_id_before_exact_name() {
        let payload = json!({
            "conversations": [
                {
                    "conversation_id": "R:0000000001",
                    "conversation_name": "Synthetic Alpha"
                },
                {
                    "conversation_id": "R:0000000002",
                    "conversation_name": "R:0000000001"
                }
            ]
        });

        let resolved = resolve_one(&payload, "R:0000000001").unwrap();
        assert_eq!(resolved.conversation_id, "R:0000000001");
    }

    #[test]
    fn resolves_exact_conversation_reference_before_fuzzy_matches() {
        let payload = json!({
            "conversations": [
                {
                    "conversation_id": "R:0000000001",
                    "conversation_name": "Synthetic Alpha"
                },
                {
                    "conversation_id": "R:0000000002",
                    "conversation_name": "Synthetic Alpha Ops"
                }
            ]
        });

        let resolved = resolve_one(&payload, "Synthetic Alpha").unwrap();
        assert_eq!(resolved.conversation_id, "R:0000000001");
    }

    #[test]
    fn resolves_exact_conversation_reference_case_insensitively() {
        let payload = json!({
            "conversations": [
                {
                    "conversation_id": "R:0000000001",
                    "conversation_name": "Synthetic Alpha"
                },
                {
                    "conversation_id": "R:0000000002",
                    "conversation_name": "Synthetic Alpha Ops"
                }
            ]
        });

        let resolved = resolve_one(&payload, "synthetic alpha").unwrap();
        assert_eq!(resolved.conversation_id, "R:0000000001");
    }

    #[test]
    fn resolves_conversation_id_when_name_is_missing() {
        let payload = json!({
            "conversations": [
                {
                    "conversation_id": "R:0000000001"
                }
            ]
        });

        let resolved = resolve_one(&payload, "R:0000000001").unwrap();
        assert_eq!(resolved.conversation_id, "R:0000000001");
    }

    #[test]
    fn rejects_ambiguous_conversation_references() {
        let payload = json!({
            "conversations": [
                {
                    "conversation_id": "R:0000000001",
                    "conversation_name": "Synthetic Alpha"
                },
                {
                    "conversation_id": "R:0000000002",
                    "conversation_name": "Synthetic Alpha Ops"
                }
            ]
        });

        let error = resolve_one(&payload, "Alpha").unwrap_err();
        assert!(error.to_string().contains("matched 2 conversations"));
        assert!(error.to_string().contains("Synthetic Alpha"));
    }

    #[test]
    fn rejects_missing_conversation_references() {
        let payload = json!({
            "conversations": []
        });

        let error = resolve_one(&payload, "Missing").unwrap_err();
        assert!(error.to_string().contains("did not match"));
    }

    #[test]
    fn rejects_blank_conversation_references() {
        let payload = json!({
            "conversations": [
                {
                    "conversation_id": "R:0000000001",
                    "conversation_name": "Synthetic Alpha"
                }
            ]
        });

        let error = resolve_one(&payload, "   ").unwrap_err();
        assert!(error.to_string().contains("cannot be empty"));
    }

    #[test]
    fn identifies_direct_conversation_ids() {
        assert!(looks_like_conversation_id("R:0000000001"));
        assert!(looks_like_conversation_id("S:0000000001"));
        assert!(looks_like_conversation_id(" R:0000000001 "));
        assert!(!looks_like_conversation_id("Synthetic Alpha"));
        assert!(!looks_like_conversation_id("R:"));
        assert!(!looks_like_conversation_id("R:0000 0001"));
    }

    #[test]
    fn surfaces_runtime_errors() {
        let raw = json!({
            "error": "conversation list selector unavailable",
            "conversations": []
        });

        let error = normalize_payload(raw, None).unwrap_err();
        assert!(error
            .to_string()
            .contains("conversation list selector unavailable"));
    }
}
