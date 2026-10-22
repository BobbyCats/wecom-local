use std::{
    process::{Command, Stdio},
    thread,
    time::Duration,
};

use anyhow::{bail, Context, Result};
use serde::Serialize;

use crate::runtime_bridge::{self, RuntimePermissionDenied};

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
    pub runtime_access: String,
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

pub fn probe_status() -> AuthStatusReport {
    apply_runtime_probe(status(), runtime_bridge::probe_access())
}

fn apply_runtime_probe(mut report: AuthStatusReport, result: Result<()>) -> AuthStatusReport {
    match result {
        Ok(()) => {
            report.status = "ready".into();
            report.authorization_method = if report.running_as_root {
                "effective_root_runtime"
            } else {
                "current_user_runtime"
            }
            .into();
            report.runtime_access = "verified".into();
            report.can_prepare = false;
            report.detail = "current process reached the WeCom query service without reading chat rows; this verifies current access, not a permanent authorization grant".into();
        }
        Err(error) if error.is::<RuntimePermissionDenied>() => {
            report.runtime_access = "denied".into();
            report.status = if report.running_as_root {
                "needs_attention"
            } else {
                "needs_authorization"
            }
            .into();
            report.can_prepare =
                !report.running_as_root && report.authorization_method != "sudo_unavailable";
            report.detail = error.to_string();
        }
        Err(error) => {
            report.runtime_access = "unavailable".into();
            report.status = "needs_attention".into();
            report.can_prepare = false;
            report.detail = error.to_string();
        }
    }
    report
}

pub fn prepare(keepalive_minutes: u16) -> Result<AuthPrepareReport> {
    if keepalive_minutes > MAX_KEEPALIVE_MINUTES {
        bail!("--keepalive-minutes must be between 0 and {MAX_KEEPALIVE_MINUTES}");
    }

    let status_before = probe_status();
    if status_before.runtime_access == "verified" {
        return Ok(AuthPrepareReport {
            prepared: true,
            keepalive_minutes: 0,
            keepalive_refresh_count: 0,
            password_stored: false,
            status_after: status_before.clone(),
            status_before,
        });
    }
    if !status_before.can_prepare {
        bail!("runtime access is not ready: {}", status_before.detail);
    }
    // First reuse an existing timestamp without ever prompting in an Agent session.
    let prepared = if check_sudo_timestamp() == SudoProbe::Cached {
        true
    } else {
        use std::io::IsTerminal;
        if !std::io::stdin().is_terminal() || !std::io::stderr().is_terminal() {
            bail!("macOS runtime authorization is required; run wecom-local auth prepare in the same interactive terminal as the query. Do not send passwords to an Agent");
        }
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
        prepared: prepared && status_after.status == "ready",
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
            detail: "process already has root privileges; runtime access is not checked"
                .to_string(),
            runtime_access: "not_checked".into(),
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
            detail: "sudo timestamp is cached for this sudo timestamp scope; runtime access is not checked".to_string(),
            runtime_access: "not_checked".into(),
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
                "sudo timestamp is not cached; this does not mean runtime access is denied. ",
                "Try the query without sudo, or use auth status --probe --json"
            )
            .to_string(),
            runtime_access: "not_checked".into(),
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
            runtime_access: "not_checked".into(),
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
    fn direct_access_is_ready_without_a_sudo_timestamp() {
        for probe in [SudoProbe::NeedsAuthorization, SudoProbe::Unavailable] {
            let report = apply_runtime_probe(build_status_report(probe, false), Ok(()));
            assert_eq!(report.status, "ready");
            assert_eq!(report.runtime_access, "verified");
            assert_eq!(report.authorization_method, "current_user_runtime");
            assert!(!report.sudo_timestamp_cached);
            assert!(!report.can_prepare);
            assert!(!report.password_stored);
        }
    }

    #[test]
    fn cached_sudo_does_not_hide_a_runtime_failure() {
        let report = apply_runtime_probe(
            build_status_report(SudoProbe::Cached, false),
            Err(anyhow::anyhow!("query service unavailable")),
        );
        assert_eq!(report.status, "needs_attention");
        assert_eq!(report.runtime_access, "unavailable");
        assert!(!report.can_prepare);
    }

    #[test]
    fn only_explicit_permission_denial_allows_preparation() {
        let report = apply_runtime_probe(
            build_status_report(SudoProbe::NeedsAuthorization, false),
            Err(RuntimePermissionDenied.into()),
        );
        assert_eq!(report.status, "needs_authorization");
        assert_eq!(report.runtime_access, "denied");
        assert!(report.can_prepare);

        let root = apply_runtime_probe(
            build_status_report(SudoProbe::Cached, true),
            Err(RuntimePermissionDenied.into()),
        );
        assert_eq!(root.status, "needs_attention");
        assert!(!root.can_prepare);
    }

    #[test]
    fn sudo_cache_status_does_not_claim_runtime_verification() {
        for probe in [
            SudoProbe::Cached,
            SudoProbe::NeedsAuthorization,
            SudoProbe::Unavailable,
        ] {
            assert_eq!(
                build_status_report(probe, false).runtime_access,
                "not_checked"
            );
        }
    }

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
