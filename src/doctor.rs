use serde::Serialize;

use crate::runtime_bridge;

#[derive(Debug, Serialize)]
pub struct DoctorReport {
    pub platform: String,
    pub lldb: Check,
    pub wecom_process: Check,
    pub container_tmp: Check,
    pub status: DoctorStatus,
}

#[derive(Debug, Serialize)]
pub struct Check {
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DoctorStatus {
    Ready,
    NeedsAttention,
}

pub fn run() -> DoctorReport {
    let lldb = check_lldb();
    let wecom_process = check_wecom_process();
    let container_tmp = check_container_tmp();
    let ready = lldb.ok && wecom_process.ok && container_tmp.ok && cfg!(target_os = "macos");

    DoctorReport {
        platform: std::env::consts::OS.to_string(),
        lldb,
        wecom_process,
        container_tmp,
        status: if ready {
            DoctorStatus::Ready
        } else {
            DoctorStatus::NeedsAttention
        },
    }
}

fn check_lldb() -> Check {
    let path = std::path::Path::new("/usr/bin/lldb");
    Check {
        ok: cfg!(target_os = "macos") && path.exists(),
        detail: if path.exists() {
            path.display().to_string()
        } else {
            "lldb not found at /usr/bin/lldb".to_string()
        },
    }
}

fn check_wecom_process() -> Check {
    match runtime_bridge::find_wecom_process() {
        Ok(process) => Check {
            ok: true,
            detail: format!("{} pid={}", process.name, process.pid),
        },
        Err(err) => Check {
            ok: false,
            detail: err.to_string(),
        },
    }
}

fn check_container_tmp() -> Check {
    match runtime_bridge::wecom_container_tmp() {
        Ok(path) => Check {
            ok: true,
            detail: path.display().to_string(),
        },
        Err(err) => Check {
            ok: false,
            detail: err.to_string(),
        },
    }
}
