use std::{
    collections::BTreeMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Result;
use serde::Serialize;

const SQLITE_HEADER: &[u8; 16] = b"SQLite format 3\0";
const IMPORTANT_DB_NAMES: &[&str] = &[
    "message.db",
    "message_lookup.db",
    "session.db",
    "user.db",
    "company.db",
];

#[derive(Debug, Serialize)]
pub struct StoreProbeReport {
    pub platform: String,
    pub data_root: DataRootProbe,
    pub db_files: DbFileSummary,
    pub important_files: Vec<ImportantFileProbe>,
    pub format_probe: FormatProbe,
    pub schema_probe: SchemaProbe,
    pub key_probe: KeyProbe,
    pub page_validation_probe: PageValidationProbe,
    pub privacy: PrivacyProbe,
}

#[derive(Debug, Serialize)]
pub struct DataRootProbe {
    pub found: bool,
    pub redacted_path: String,
    pub account_dir_count: usize,
}

#[derive(Debug, Default, Serialize)]
pub struct DbFileSummary {
    pub total: usize,
    pub plain_sqlite: usize,
    pub wxsqlite3_like_header: usize,
    pub opaque_or_other: usize,
    pub unreadable: usize,
}

#[derive(Debug, Serialize)]
pub struct ImportantFileProbe {
    pub name: String,
    pub total: usize,
    pub plain_sqlite: usize,
    pub wxsqlite3_like_header: usize,
    pub opaque_or_other: usize,
    pub unreadable: usize,
}

#[derive(Debug, Serialize)]
pub struct SchemaProbe {
    pub attempted: bool,
    pub sqlite3_available: bool,
    pub plain_sqlite_files_checked: usize,
    pub plain_sqlite_files_with_schema: usize,
    pub total_table_count: usize,
    pub error_count: usize,
}

#[derive(Debug, Serialize)]
pub struct FormatProbe {
    pub attempted: bool,
    pub files_checked: usize,
    pub header_bytes_per_file: usize,
    pub salt_prefix_bytes: usize,
    pub sqlite_header_pattern_count: usize,
    pub wxsqlite3_header_pattern_count: usize,
    pub salt_prefix_checked_count: usize,
    pub salt_prefix_nonzero_count: usize,
    pub salt_prefix_all_zero_count: usize,
    pub page_size_candidates: Vec<PageSizeCandidate>,
    pub error_count: usize,
}

#[derive(Debug, Serialize)]
pub struct PageSizeCandidate {
    pub page_size: usize,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct KeyProbe {
    pub attempted: bool,
    pub result: String,
    pub candidate_count: usize,
    pub matched_count: usize,
    pub validated: bool,
    pub error_kind: String,
}

#[derive(Debug, Serialize)]
pub struct PageValidationProbe {
    pub attempted: bool,
    pub algorithm_label: String,
    pub page_size: Option<usize>,
    pub validated: bool,
    pub error_kind: String,
}

#[derive(Debug, Serialize)]
pub struct PrivacyProbe {
    pub row_values_read: bool,
    pub message_content_read: bool,
    pub member_values_read: bool,
    pub real_paths_emitted: bool,
    pub keys_emitted: bool,
    pub memory_bytes_emitted: bool,
    pub decrypted_files_written: bool,
    pub memory_dump_written: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DbKind {
    PlainSqlite,
    Wxsqlite3LikeHeader,
    OpaqueOrOther,
    Unreadable,
}

#[derive(Debug, Clone, Copy)]
struct HeaderProbe {
    kind: DbKind,
    page_size: Option<usize>,
    salt_prefix_all_zero: Option<bool>,
}

#[derive(Debug, Default)]
struct FormatAccumulator {
    files_checked: usize,
    sqlite_header_pattern_count: usize,
    wxsqlite3_header_pattern_count: usize,
    salt_prefix_checked_count: usize,
    salt_prefix_nonzero_count: usize,
    salt_prefix_all_zero_count: usize,
    page_size_counts: BTreeMap<usize, usize>,
    error_count: usize,
}

pub fn run() -> StoreProbeReport {
    let root = default_data_root();
    run_for_root(&root)
}

fn run_for_root(root: &Path) -> StoreProbeReport {
    let mut summary = DbFileSummary::default();
    let mut important = important_map();
    let mut format = FormatAccumulator::default();
    let mut plain_sqlite_paths = Vec::new();

    if root.exists() {
        collect_db_files(root, &mut |path| {
            let header = probe_db_header(path);
            let kind = header.kind;
            increment_summary(&mut summary, kind);
            accumulate_format(&mut format, header);
            if kind == DbKind::PlainSqlite {
                plain_sqlite_paths.push(path.to_path_buf());
            }
            if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
                if let Some(probe) = important.get_mut(file_name) {
                    probe.total += 1;
                    increment_important(probe, kind);
                }
            }
        });
    }

    StoreProbeReport {
        platform: std::env::consts::OS.to_string(),
        data_root: DataRootProbe {
            found: root.exists(),
            redacted_path: redacted_data_root(),
            account_dir_count: count_account_dirs(root),
        },
        db_files: summary,
        important_files: important.into_values().collect(),
        format_probe: format.into_probe(),
        schema_probe: probe_plain_sqlite_schema(&plain_sqlite_paths),
        key_probe: KeyProbe {
            attempted: false,
            result: "not_attempted_by_store_probe".to_string(),
            candidate_count: 0,
            matched_count: 0,
            validated: false,
            error_kind: "not_attempted_by_store_probe".to_string(),
        },
        page_validation_probe: PageValidationProbe {
            attempted: false,
            algorithm_label: "not_attempted".to_string(),
            page_size: None,
            validated: false,
            error_kind: "no_validated_key".to_string(),
        },
        privacy: PrivacyProbe {
            row_values_read: false,
            message_content_read: false,
            member_values_read: false,
            real_paths_emitted: false,
            keys_emitted: false,
            memory_bytes_emitted: false,
            decrypted_files_written: false,
            memory_dump_written: false,
        },
    }
}

fn default_data_root() -> PathBuf {
    home_dir().join(
        "Library/Containers/com.tencent.WeWorkMac/Data/Library/Application Support/WXWork/Data",
    )
}

fn redacted_data_root() -> String {
    "~/Library/Containers/com.tencent.WeWorkMac/Data/Library/Application Support/WXWork/Data"
        .to_string()
}

fn home_dir() -> PathBuf {
    #[cfg(unix)]
    {
        use std::ffi::{CStr, CString};
        if let Ok(sudo_user) = std::env::var("SUDO_USER") {
            if let Ok(c_user) = CString::new(sudo_user) {
                unsafe {
                    let pwd = libc::getpwnam(c_user.as_ptr());
                    if !pwd.is_null() && !(*pwd).pw_dir.is_null() {
                        if let Ok(path) = CStr::from_ptr((*pwd).pw_dir).to_str() {
                            return PathBuf::from(path);
                        }
                    }
                }
            }
        }
    }
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"))
}

fn count_account_dirs(root: &Path) -> usize {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return 0,
    };
    entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir() && path.join("Data").is_dir())
        .count()
}

fn collect_db_files(root: &Path, visit: &mut impl FnMut(&Path)) {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_db_files(&path, visit);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("db") {
            visit(&path);
        }
    }
}

fn probe_db_header(path: &Path) -> HeaderProbe {
    let mut header = [0u8; 24];
    let mut file = match fs::File::open(path) {
        Ok(file) => file,
        Err(_) => return unreadable_header_probe(),
    };
    if file.read_exact(&mut header).is_err() {
        return unreadable_header_probe();
    }
    classify_header(&header)
}

fn unreadable_header_probe() -> HeaderProbe {
    HeaderProbe {
        kind: DbKind::Unreadable,
        page_size: None,
        salt_prefix_all_zero: None,
    }
}

fn classify_header(header: &[u8; 24]) -> HeaderProbe {
    let page_size = parse_sqlite_page_size(header);
    if &header[..16] == SQLITE_HEADER {
        return HeaderProbe {
            kind: DbKind::PlainSqlite,
            page_size,
            salt_prefix_all_zero: None,
        };
    }
    if has_wxsqlite3_plain_header_fragment(header) {
        return HeaderProbe {
            kind: DbKind::Wxsqlite3LikeHeader,
            page_size,
            salt_prefix_all_zero: Some(header[..16].iter().all(|byte| *byte == 0)),
        };
    }
    HeaderProbe {
        kind: DbKind::OpaqueOrOther,
        page_size: None,
        salt_prefix_all_zero: None,
    }
}

fn has_wxsqlite3_plain_header_fragment(header: &[u8; 24]) -> bool {
    parse_sqlite_page_size(header).is_some()
        && header[21] == 0x40
        && header[22] == 0x20
        && header[23] == 0x20
}

fn parse_sqlite_page_size(header: &[u8; 24]) -> Option<usize> {
    let mut page_size = u16::from_be_bytes([header[16], header[17]]) as usize;
    if page_size == 1 {
        page_size = 65_536;
    }

    ((512..=65_536).contains(&page_size) && page_size.is_power_of_two()).then_some(page_size)
}

fn important_map() -> BTreeMap<String, ImportantFileProbe> {
    IMPORTANT_DB_NAMES
        .iter()
        .map(|name| {
            (
                (*name).to_string(),
                ImportantFileProbe {
                    name: (*name).to_string(),
                    total: 0,
                    plain_sqlite: 0,
                    wxsqlite3_like_header: 0,
                    opaque_or_other: 0,
                    unreadable: 0,
                },
            )
        })
        .collect()
}

fn increment_summary(summary: &mut DbFileSummary, kind: DbKind) {
    summary.total += 1;
    match kind {
        DbKind::PlainSqlite => summary.plain_sqlite += 1,
        DbKind::Wxsqlite3LikeHeader => summary.wxsqlite3_like_header += 1,
        DbKind::OpaqueOrOther => summary.opaque_or_other += 1,
        DbKind::Unreadable => summary.unreadable += 1,
    }
}

fn increment_important(probe: &mut ImportantFileProbe, kind: DbKind) {
    match kind {
        DbKind::PlainSqlite => probe.plain_sqlite += 1,
        DbKind::Wxsqlite3LikeHeader => probe.wxsqlite3_like_header += 1,
        DbKind::OpaqueOrOther => probe.opaque_or_other += 1,
        DbKind::Unreadable => probe.unreadable += 1,
    }
}

fn accumulate_format(accumulator: &mut FormatAccumulator, header: HeaderProbe) {
    match header.kind {
        DbKind::PlainSqlite => {
            accumulator.files_checked += 1;
            accumulator.sqlite_header_pattern_count += 1;
            if let Some(page_size) = header.page_size {
                *accumulator.page_size_counts.entry(page_size).or_default() += 1;
            }
        }
        DbKind::Wxsqlite3LikeHeader => {
            accumulator.files_checked += 1;
            accumulator.wxsqlite3_header_pattern_count += 1;
            accumulator.salt_prefix_checked_count += 1;
            match header.salt_prefix_all_zero {
                Some(true) => accumulator.salt_prefix_all_zero_count += 1,
                Some(false) => accumulator.salt_prefix_nonzero_count += 1,
                None => {}
            }
            if let Some(page_size) = header.page_size {
                *accumulator.page_size_counts.entry(page_size).or_default() += 1;
            }
        }
        DbKind::OpaqueOrOther => {
            accumulator.files_checked += 1;
        }
        DbKind::Unreadable => {
            accumulator.error_count += 1;
        }
    }
}

impl FormatAccumulator {
    fn into_probe(self) -> FormatProbe {
        FormatProbe {
            attempted: true,
            files_checked: self.files_checked,
            header_bytes_per_file: 24,
            salt_prefix_bytes: 16,
            sqlite_header_pattern_count: self.sqlite_header_pattern_count,
            wxsqlite3_header_pattern_count: self.wxsqlite3_header_pattern_count,
            salt_prefix_checked_count: self.salt_prefix_checked_count,
            salt_prefix_nonzero_count: self.salt_prefix_nonzero_count,
            salt_prefix_all_zero_count: self.salt_prefix_all_zero_count,
            page_size_candidates: self
                .page_size_counts
                .into_iter()
                .map(|(page_size, total)| PageSizeCandidate { page_size, total })
                .collect(),
            error_count: self.error_count,
        }
    }
}

fn probe_plain_sqlite_schema(paths: &[PathBuf]) -> SchemaProbe {
    let sqlite3_available = Command::new("sqlite3").arg("-version").output().is_ok();
    if !sqlite3_available {
        return SchemaProbe {
            attempted: false,
            sqlite3_available,
            plain_sqlite_files_checked: 0,
            plain_sqlite_files_with_schema: 0,
            total_table_count: 0,
            error_count: 0,
        };
    }

    let mut files_with_schema = 0;
    let mut total_table_count = 0;
    let mut error_count = 0;
    for path in paths {
        match plain_sqlite_table_count(path) {
            Ok(count) => {
                files_with_schema += 1;
                total_table_count += count;
            }
            Err(_) => error_count += 1,
        }
    }

    SchemaProbe {
        attempted: true,
        sqlite3_available,
        plain_sqlite_files_checked: paths.len(),
        plain_sqlite_files_with_schema: files_with_schema,
        total_table_count,
        error_count,
    }
}

fn plain_sqlite_table_count(path: &Path) -> Result<usize> {
    let output = Command::new("sqlite3")
        .arg(path)
        .arg("SELECT COUNT(*) FROM sqlite_master WHERE type='table';")
        .output()?;
    if !output.status.success() {
        anyhow::bail!("sqlite3 failed");
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let count = text.trim().parse::<usize>()?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_plain_sqlite_header() {
        let mut header = [0u8; 24];
        header[..16].copy_from_slice(SQLITE_HEADER);
        let probe = classify_header(&header);
        assert_eq!(probe.kind, DbKind::PlainSqlite);
        assert_eq!(probe.page_size, None);
        assert_eq!(probe.salt_prefix_all_zero, None);
    }

    #[test]
    fn classify_wxsqlite3_like_header() {
        let header = wxsqlite3_header();
        let probe = classify_header(&header);
        assert_eq!(probe.kind, DbKind::Wxsqlite3LikeHeader);
        assert_eq!(probe.page_size, Some(4096));
        assert_eq!(probe.salt_prefix_all_zero, Some(false));
    }

    #[test]
    fn store_probe_does_not_emit_sensitive_material() {
        let root = unique_temp_dir();
        fs::create_dir_all(root.join("1000000000/Data")).unwrap();
        fs::write(root.join("1000000000/Data/message.db"), wxsqlite3_header()).unwrap();

        let report = run_for_root(&root);

        assert!(report.data_root.found);
        assert_eq!(report.data_root.redacted_path, redacted_data_root());
        assert_eq!(report.data_root.account_dir_count, 1);
        assert_eq!(report.db_files.wxsqlite3_like_header, 1);
        assert_eq!(report.format_probe.wxsqlite3_header_pattern_count, 1);
        assert_eq!(report.format_probe.salt_prefix_nonzero_count, 1);
        assert_eq!(report.format_probe.page_size_candidates.len(), 1);
        assert_eq!(report.format_probe.page_size_candidates[0].page_size, 4096);
        assert!(!report.key_probe.attempted);
        assert!(!report.key_probe.validated);
        assert!(!report.page_validation_probe.attempted);
        assert!(!report.page_validation_probe.validated);
        assert!(!report.privacy.row_values_read);
        assert!(!report.privacy.keys_emitted);
        assert!(!report.privacy.memory_bytes_emitted);
        assert!(!report.privacy.decrypted_files_written);
        assert!(!report.privacy.memory_dump_written);

        let serialized = serde_json::to_string(&report).unwrap();
        assert!(!serialized.contains("1000000000"));
        assert!(!serialized.contains(root.to_string_lossy().as_ref()));

        let _ = fs::remove_dir_all(root);
    }

    fn wxsqlite3_header() -> [u8; 24] {
        let mut header = [0u8; 24];
        header[..16].copy_from_slice(&[0xA5; 16]);
        header[16] = 0x10;
        header[17] = 0x00;
        header[21] = 0x40;
        header[22] = 0x20;
        header[23] = 0x20;
        header
    }

    fn unique_temp_dir() -> PathBuf {
        std::env::temp_dir().join(format!(
            "wecom-local-store-probe-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }
}
