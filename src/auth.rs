use std::{
    process::{Command, Stdio},
    thread,
    time::Duration,
};

use anyhow::{bail, Context, Result};
use serde::Serialize;

const MAX_KEEPALIVE_MINUTES: u16 = 60;

#[derive(Debug, Clone, Serialize)]
pub struct AuthStatusReport {
    pub platform: String,
    pub status: String,
    pub authorization_method: String,
    pub sudo_timestamp_cached: bool,
    pub running_as_root: bool,
    pub password_stored: bool,
    pub can_prepare: bool,
    pub prepare_command: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthPrepareReport {
    pub prepared: bool,
    pub keepalive_minutes: u16,
    pub keepalive_refresh_count: u16,
    pub password_stored: bool,
    pub status_before: AuthStatusReport,
    pub status_after: AuthStatusReport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SudoProbe {
    Cached,
    NeedsAuthorization,
    Unavailable,
}

pub fn status() -> AuthStatusReport {
    build_status_report(check_sudo_timestamp(), running_as_root())
}

pub fn prepare(keepalive_minutes: u16) -> Result<AuthPrepareReport> {
    if keepalive_minutes > MAX_KEEPALIVE_MINUTES {
        bail!("--keepalive-minutes must be between 0 and {MAX_KEEPALIVE_MINUTES}");
    }

    let status_before = status();
    let prepared = if running_as_root() {
        true
    } else {
        Command::new(sudo_path())
            .arg("-v")
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .with_context(|| format!("failed to run {}", sudo_path()))?
            .success()
    };

    let mut keepalive_refresh_count = 0;
    if prepared && keepalive_minutes > 0 {
        keepalive_refresh_count = keepalive(keepalive_minutes);
    }

    let status_after = status();
    Ok(AuthPrepareReport {
        prepared: status_after.status == "ready",
        keepalive_minutes,
        keepalive_refresh_count,
        password_stored: false,
        status_before,
        status_after,
    })
}

fn keepalive(minutes: u16) -> u16 {
    let mut refresh_count = 0;
    for _ in 0..minutes {
        thread::sleep(Duration::from_secs(60));
        if check_sudo_timestamp() != SudoProbe::Cached {
            break;
        }
        refresh_count += 1;
    }
    refresh_count
}

fn build_status_report(sudo_probe: SudoProbe, running_as_root: bool) -> AuthStatusReport {
    if running_as_root {
        return AuthStatusReport {
            platform: std::env::consts::OS.to_string(),
            status: "ready".to_string(),
            authorization_method: "effective_root".to_string(),
            sudo_timestamp_cached: true,
            running_as_root,
            password_stored: false,
            can_prepare: false,
            prepare_command: "wecom-local auth prepare".to_string(),
            detail: "process already has root privileges".to_string(),
        };
    }

    match sudo_probe {
        SudoProbe::Cached => AuthStatusReport {
            platform: std::env::consts::OS.to_string(),
            status: "ready".to_string(),
            authorization_method: "sudo_timestamp".to_string(),
            sudo_timestamp_cached: true,
            running_as_root,
            password_stored: false,
            can_prepare: true,
            prepare_command: "wecom-local auth prepare".to_string(),
            detail: "sudo timestamp is cached for this sudo timestamp scope".to_string(),
        },
        SudoProbe::NeedsAuthorization => AuthStatusReport {
            platform: std::env::consts::OS.to_string(),
            status: "needs_authorization".to_string(),
            authorization_method: "sudo_prompt_required".to_string(),
            sudo_timestamp_cached: false,
            running_as_root,
            password_stored: false,
            can_prepare: true,
            prepare_command: "wecom-local auth prepare".to_string(),
            detail: concat!(
                "authorize in the same interactive sudo timestamp scope that will run runtime ",
                "queries; do not send passwords to an Agent"
            )
            .to_string(),
        },
        SudoProbe::Unavailable => AuthStatusReport {
            platform: std::env::consts::OS.to_string(),
            status: "unavailable".to_string(),
            authorization_method: "sudo_unavailable".to_string(),
            sudo_timestamp_cached: false,
            running_as_root,
            password_stored: false,
            can_prepare: false,
            prepare_command: "wecom-local auth prepare".to_string(),
            detail: "sudo command is unavailable".to_string(),
        },
    }
}

fn check_sudo_timestamp() -> SudoProbe {
    let output = Command::new(sudo_path())
        .arg("-n")
        .arg("-v")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output();

    match output {
        Ok(output) if output.status.success() => SudoProbe::Cached,
        Ok(_) => SudoProbe::NeedsAuthorization,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => SudoProbe::Unavailable,
        Err(_) => SudoProbe::Unavailable,
    }
}

fn sudo_path() -> &'static str {
    "/usr/bin/sudo"
}

#[cfg(unix)]
fn running_as_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

#[cfg(not(unix))]
fn running_as_root() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_reports_no_password_storage_when_sudo_cached() {
        let report = build_status_report(SudoProbe::Cached, false);

        assert_eq!(report.status, "ready");
        assert_eq!(report.authorization_method, "sudo_timestamp");
        assert!(report.sudo_timestamp_cached);
        assert!(!report.password_stored);
        assert!(report.can_prepare);
    }

    #[test]
    fn status_reports_prepare_path_when_authorization_is_needed() {
        let report = build_status_report(SudoProbe::NeedsAuthorization, false);

        assert_eq!(report.status, "needs_authorization");
        assert_eq!(report.authorization_method, "sudo_prompt_required");
        assert!(!report.sudo_timestamp_cached);
        assert!(!report.password_stored);
        assert!(report.can_prepare);
    }

    #[test]
    fn status_reports_root_without_prompting() {
        let report = build_status_report(SudoProbe::NeedsAuthorization, true);

        assert_eq!(report.status, "ready");
        assert_eq!(report.authorization_method, "effective_root");
        assert!(report.running_as_root);
        assert!(!report.password_stored);
        assert!(!report.can_prepare);
    }
}
