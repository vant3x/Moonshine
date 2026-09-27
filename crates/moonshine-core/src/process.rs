use crate::error::{MoonshineError, Result};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::process::{Child, Command, Stdio};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramRequest {
    pub executable: PathBuf,
}

impl ProgramRequest {
    pub fn new(executable: impl Into<PathBuf>) -> Result<Self> {
        let executable = executable.into();
        if !executable.is_absolute()
            || !executable.is_file()
            || executable.extension().is_none()
        {
            return Err(MoonshineError::InvalidPath(executable));
        }

        Ok(Self { executable })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessLaunchResult {
    pub pid: u32,
}

impl ProcessLaunchResult {
    pub fn new(pid: u32) -> Result<Self> {
        if pid == 0 {
            return Err(MoonshineError::Config("process returned an invalid PID".to_string()));
        }
        Ok(Self { pid })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessState {
    Running,
    Exited(Option<i32>),
    Failed(String),
}

#[derive(Debug, Clone)]
struct ProcessRecord {
    state: Arc<Mutex<ProcessState>>,
    log_path: PathBuf,
}

static PROCESS_REGISTRY: OnceLock<Mutex<HashMap<u32, ProcessRecord>>> = OnceLock::new();

fn registry() -> &'static Mutex<HashMap<u32, ProcessRecord>> {
    PROCESS_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn spawn_tracked(mut command: Command, log_path: PathBuf) -> Result<ProcessLaunchResult> {
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let stdout = OpenOptions::new().create(true).append(true).open(&log_path)?;
    let stderr = stdout.try_clone()?;
    command.stdout(Stdio::from(stdout)).stderr(Stdio::from(stderr));
    let child = command.spawn()?;
    let pid = child.id();
    let result = ProcessLaunchResult::new(pid)?;
    let state = Arc::new(Mutex::new(ProcessState::Running));
    registry().lock().unwrap().insert(pid, ProcessRecord { state: state.clone(), log_path });
    thread::spawn(move || supervise(child, state));
    Ok(result)
}

fn supervise(mut child: Child, state: Arc<Mutex<ProcessState>>) {
    let new_state = match child.wait() {
        Ok(status) => ProcessState::Exited(status.code()),
        Err(error) => ProcessState::Failed(error.to_string()),
    };
    if let Ok(mut current) = state.lock() {
        *current = new_state;
    }
}

pub fn state(pid: u32) -> Option<ProcessState> {
    registry().lock().ok()?.get(&pid)?.state.lock().ok().map(|state| state.clone())
}

pub fn log_path(pid: u32) -> Option<PathBuf> {
    registry().lock().ok()?.get(&pid).map(|record| record.log_path.clone())
}

pub fn state_json(pid: u32) -> String {
    let state = match state(pid) {
        Some(ProcessState::Running) => "running".to_string(),
        Some(ProcessState::Exited(code)) => format!("exited:{}", code.map_or_else(|| "signal".to_string(), |value| value.to_string())),
        Some(ProcessState::Failed(error)) => format!("failed:{}", error),
        None => "unknown".to_string(),
    };
    let log = log_path(pid).map(|path| path.to_string_lossy().to_string()).unwrap_or_default();
    serde_json::json!({ "pid": pid, "state": state, "log_path": log }).to_string()
}

pub fn validate_path_is_file(path: &Path) -> Result<()> {
    if path.is_absolute() && path.is_file() {
        Ok(())
    } else {
        Err(MoonshineError::InvalidPath(path.to_path_buf()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn program_request_accepts_existing_absolute_file() {
        let path = std::env::temp_dir().join(format!("moonshine-program-{}.exe", uuid::Uuid::new_v4()));
        fs::write(&path, b"fixture").unwrap();

        assert_eq!(ProgramRequest::new(&path).unwrap().executable, path);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn program_request_rejects_missing_file() {
        let path = std::env::temp_dir().join("moonshine-missing.exe");
        assert!(matches!(ProgramRequest::new(path), Err(MoonshineError::InvalidPath(_))));
    }

    #[test]
    fn process_result_rejects_zero_pid() {
        assert!(ProcessLaunchResult::new(0).is_err());
    }

    #[test]
    fn tracked_process_reports_nonzero_exit_and_log_path() {
        let log_path = std::env::temp_dir().join(format!("moonshine-process-{}.log", uuid::Uuid::new_v4()));
        let mut command = Command::new("/bin/sh");
        command.args(["-c", "printf launch-log; exit 7"]);
        let process = spawn_tracked(command, log_path.clone()).unwrap();

        for _ in 0..50 {
            if matches!(state(process.pid), Some(ProcessState::Exited(Some(7)))) {
                break;
            }
            thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(state(process.pid), Some(ProcessState::Exited(Some(7))));
        assert_eq!(fs::read_to_string(&log_path).unwrap(), "launch-log");
        fs::remove_file(log_path).unwrap();
    }
}