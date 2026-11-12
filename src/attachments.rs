use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::runtime_bridge;

pub fn resolve_local_files(payload: &mut Value) {
    let root = runtime_bridge::wecom_container_tmp()
        .map(|tmp| tmp.with_file_name("Documents").join("Profiles"));
    resolve_in_profiles(payload, root.as_deref().ok());
}

fn resolve_in_profiles(payload: &mut Value, root: Option<&Path>) {
    let Some(rows) = payload.get_mut("messages").and_then(Value::as_array_mut) else {
        return;
    };
    if !rows.iter().any(|row| row.get("attachment").is_some()) {
        return;
    }
    let months = root.map(cache_months).transpose();
    for row in rows {
        let Some(attachment) = row.get_mut("attachment") else {
            continue;
        };
        let (Some(name), Some(md5)) = (
            attachment.get("name").and_then(Value::as_str),
            attachment.get("content_md5").and_then(Value::as_str),
        ) else {
            attachment["local_state"] = json!("metadata_unavailable");
            continue;
        };
        let Ok(Some(months)) = &months else {
            attachment["local_state"] = json!("cache_unavailable");
            continue;
        };
        let expected_size = attachment.get("size_bytes").and_then(Value::as_u64);
        let mut unavailable = false;
        let mut found = None;
        for month in months {
            let path = month.join(md5).join(name);
            match fs::symlink_metadata(&path) {
                Ok(metadata) if metadata.is_file() => {
                    if expected_size.is_none_or(|size| size == metadata.len()) {
                        found = Some(path);
                        break;
                    }
                }
                Ok(_) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(_) => unavailable = true,
            }
        }
        attachment["local_state"] = json!(match (&found, unavailable) {
            (Some(_), _) => "available",
            (None, true) => "cache_unavailable",
            (None, false) => "not_downloaded",
        });
        attachment["local_path"] = json!(found);
    }
}

fn cache_months(root: &Path) -> io::Result<Vec<PathBuf>> {
    let mut months = Vec::new();
    for profile in fs::read_dir(root)? {
        let profile = profile?;
        if !profile.file_type()?.is_dir() {
            continue;
        }
        let files = match fs::read_dir(profile.path().join("Caches/Files")) {
            Ok(files) => files,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
        for month in files {
            let month = month?;
            if month.file_type()?.is_dir() {
                months.push(month.path());
            }
        }
    }
    months.sort();
    Ok(months)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_cache_digest_and_size_and_reports_missing_files() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "wecom-attachment-test-{}-{unique}",
            std::process::id()
        ));
        let folder = root.join("profile/Caches/Files/2026-01/0123456789abcdef0123456789abcdef");
        fs::create_dir_all(&folder).unwrap();
        let path = folder.join("example.md");
        fs::write(&path, b"example").unwrap();
        let mut payload = json!({"messages": [{"attachment": {
            "name": "example.md",
            "content_md5": "0123456789abcdef0123456789abcdef",
            "size_bytes": 7
        }}]});

        resolve_in_profiles(&mut payload, Some(&root));
        assert_eq!(
            payload["messages"][0]["attachment"]["local_state"],
            "available"
        );
        assert_eq!(
            payload["messages"][0]["attachment"]["local_path"],
            json!(path)
        );

        payload["messages"][0]["attachment"]["size_bytes"] = json!(99);
        resolve_in_profiles(&mut payload, Some(&root));
        assert_eq!(
            payload["messages"][0]["attachment"]["local_state"],
            "not_downloaded"
        );
        assert!(payload["messages"][0]["attachment"]["local_path"].is_null());

        payload["messages"][0]["attachment"]["size_bytes"] = json!(7);
        payload["messages"][0]["attachment"]["content_md5"] =
            json!("fedcba9876543210fedcba9876543210");
        resolve_in_profiles(&mut payload, Some(&root));
        assert_eq!(
            payload["messages"][0]["attachment"]["local_state"],
            "not_downloaded"
        );

        fs::remove_dir_all(&root).unwrap();
        resolve_in_profiles(&mut payload, Some(&root));
        assert_eq!(
            payload["messages"][0]["attachment"]["local_state"],
            "cache_unavailable"
        );
    }
}
