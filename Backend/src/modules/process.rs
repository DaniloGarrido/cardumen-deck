use std::collections::HashMap;
use std::io::Write;
use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use windows_sys::Win32::Foundation::{CloseHandle, FILETIME, HANDLE, STILL_ACTIVE};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
    SetInformationJobObject,
};
use windows_sys::Win32::System::Threading::{
    GetExitCodeProcess, GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
};

use crate::modules::config::{ConfigError, load_runtime};

#[derive(Debug, thiserror::Error)]
pub enum ProcessError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Config error: {0}")]
    Config(#[from] ConfigError),
    #[error("Process already running")]
    AlreadyRunning,
    #[error("Windows API error: {0}")]
    Windows(String),
}

pub type ProcessResult<T, E = ProcessError> = std::result::Result<T, E>;

#[derive(Debug, Clone)]
pub struct ProcessHandle {
    pub pid: u32,
    pub job_handle: Option<usize>,
    pub run_id: String,
    pub log_path: String,
    pub process_created_at: Option<u64>,
}

pub static PROCESSES: once_cell::sync::Lazy<Arc<Mutex<HashMap<String, ProcessHandle>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

pub fn resolve_cwd(cwd: &str) -> std::path::PathBuf {
    let trimmed = cwd.trim();
    if trimmed.is_empty() {
        dirs_next::home_dir()
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| std::path::PathBuf::from("."))
    } else {
        std::path::PathBuf::from(trimmed)
    }
}

pub fn detect_venv(cwd: &std::path::Path) -> Option<std::path::PathBuf> {
    for name in [".venv", "venv", "env"] {
        let candidate = cwd.join(name);
        if candidate.join("Scripts").join("python.exe").is_file()
            && candidate.join("pyvenv.cfg").is_file()
        {
            return Some(candidate);
        }
    }
    None
}

fn filetime_to_u64(filetime: FILETIME) -> u64 {
    ((filetime.dwHighDateTime as u64) << 32) | filetime.dwLowDateTime as u64
}

pub fn get_process_creation_time(pid: u32) -> Option<u64> {
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if handle.is_null() {
        return None;
    }

    let mut creation: FILETIME = unsafe { std::mem::zeroed() };
    let mut exit: FILETIME = unsafe { std::mem::zeroed() };
    let mut kernel: FILETIME = unsafe { std::mem::zeroed() };
    let mut user: FILETIME = unsafe { std::mem::zeroed() };
    let success =
        unsafe { GetProcessTimes(handle, &mut creation, &mut exit, &mut kernel, &mut user) };
    unsafe { CloseHandle(handle) };

    if success == 0 {
        None
    } else {
        Some(filetime_to_u64(creation))
    }
}

pub fn is_process_alive_with_created_at(pid: u32, expected_created_at: Option<u64>) -> bool {
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if handle.is_null() {
        return false;
    }

    let mut exit_code: u32 = 0;
    let exit_success = unsafe { GetExitCodeProcess(handle, &mut exit_code) };
    if exit_success == 0 || exit_code != STILL_ACTIVE as u32 {
        unsafe { CloseHandle(handle) };
        return false;
    }

    let mut creation: FILETIME = unsafe { std::mem::zeroed() };
    let mut exit: FILETIME = unsafe { std::mem::zeroed() };
    let mut kernel: FILETIME = unsafe { std::mem::zeroed() };
    let mut user: FILETIME = unsafe { std::mem::zeroed() };
    let times_success =
        unsafe { GetProcessTimes(handle, &mut creation, &mut exit, &mut kernel, &mut user) };
    unsafe { CloseHandle(handle) };

    if times_success == 0 {
        return false;
    }

    if let Some(expected) = expected_created_at {
        if filetime_to_u64(creation) != expected {
            return false;
        }
    }

    true
}

fn create_job_object() -> ProcessResult<isize> {
    let job = unsafe { CreateJobObjectW(std::ptr::null_mut(), std::ptr::null()) };
    if job.is_null() {
        return Err(ProcessError::Windows("CreateJobObjectW failed".into()));
    }

    let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
    info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

    let result = unsafe {
        SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            &info as *const _ as *const _,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        )
    };

    if result == 0 {
        let error = std::io::Error::last_os_error();
        unsafe { CloseHandle(job) };
        return Err(ProcessError::Windows(format!(
            "SetInformationJobObject failed: {error}"
        )));
    }

    Ok(job as isize)
}

fn assign_process_to_job(job_handle: isize, pid: u32) -> ProcessResult<()> {
    let process_handle = unsafe { OpenProcess(0x1F0FFF, 0, pid) };
    if process_handle.is_null() {
        return Err(ProcessError::Windows(format!(
            "OpenProcess failed for pid {}",
            pid
        )));
    }

    let result =
        unsafe { AssignProcessToJobObject(job_handle as HANDLE, process_handle as HANDLE) };
    unsafe { CloseHandle(process_handle) };

    if result == 0 {
        return Err(ProcessError::Windows(format!(
            "AssignProcessToJobObject failed for pid {}",
            pid
        )));
    }

    Ok(())
}

pub fn start_service(
    service_id: &str,
    cwd: &str,
    command: &str,
    args: &[String],
    env_vars: &std::collections::HashMap<String, String>,
) -> ProcessResult<ProcessHandle> {
    let mut processes = PROCESSES.lock().unwrap();

    if let Some(existing) = processes.get(service_id) {
        if is_process_alive_with_created_at(existing.pid, existing.process_created_at) {
            return Err(ProcessError::AlreadyRunning);
        }
    }

    let resolved_cwd = resolve_cwd(cwd);
    if !resolved_cwd.is_dir() {
        return Err(ProcessError::Windows(format!(
            "Working directory not found: {}",
            resolved_cwd.display()
        )));
    }

    let venv = detect_venv(&resolved_cwd);
    let mut envs = env_vars.clone();
    if let Some(venv) = venv.as_ref() {
        let scripts = venv.join("Scripts");
        let current_path = envs
            .get("PATH")
            .cloned()
            .or_else(|| std::env::var("PATH").ok())
            .unwrap_or_default();
        envs.insert(
            "VIRTUAL_ENV".to_string(),
            venv.to_string_lossy().to_string(),
        );
        envs.insert(
            "PATH".to_string(),
            format!("{};{}", scripts.display(), current_path),
        );
    }
    envs.entry("PYTHONUNBUFFERED".to_string())
        .or_insert_with(|| "1".to_string());
    envs.entry("PYTHONIOENCODING".to_string())
        .or_insert_with(|| "utf-8".to_string());

    let run_id = Uuid::new_v4().to_string();
    let log_dir = crate::modules::config::service_log_dir(service_id)?;
    std::fs::create_dir_all(&log_dir)?;
    let log_path = log_dir.join(format!("{}.log", run_id));
    let mut log_file = std::fs::File::create(&log_path)?;
    writeln!(
        log_file,
        "[Cardumen Deck] Iniciando: {} {}",
        command,
        args.join(" ")
    )?;
    writeln!(log_file, "[Cardumen Deck] cwd: {}", resolved_cwd.display())?;
    if let Some(venv) = venv.as_ref() {
        writeln!(log_file, "[Cardumen Deck] VIRTUAL_ENV: {}", venv.display())?;
    }
    log_file.flush()?;
    let log_file_stderr = log_file.try_clone()?;

    let mut job_handle = match create_job_object() {
        Ok(handle) => Some(handle),
        Err(error) => {
            eprintln!("No se pudo crear el Job Object: {error}");
            None
        }
    };

    let mut cmd = Command::new("cmd");
    cmd.arg("/C");
    cmd.arg(format!("{} {}", command, args.join(" ")));
    cmd.current_dir(&resolved_cwd);
    cmd.envs(envs.iter().map(|(k, v)| (k.as_str(), v.as_str())));
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::from(log_file));
    cmd.stderr(Stdio::from(log_file_stderr));
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

    let child = cmd.spawn()?;
    let pid = child.id();
    let process_created_at = get_process_creation_time(pid);

    if let Some(job) = job_handle {
        if let Err(error) = assign_process_to_job(job, pid) {
            eprintln!("No se pudo asignar el proceso al Job Object: {error}");
            unsafe { CloseHandle(job as HANDLE) };
            job_handle = None;
        }
    }

    let log_path_string = log_path.to_string_lossy().to_string();
    let log_path_for_exit = log_path_string.clone();
    let process_handle = ProcessHandle {
        pid,
        job_handle: job_handle.map(|handle| handle as usize),
        run_id,
        log_path: log_path_string,
        process_created_at,
    };

    processes.insert(service_id.to_string(), process_handle.clone());

    // Spawn a thread to wait for the process and clean up
    let service_id_owned = service_id.to_string();
    std::thread::spawn(move || {
        let mut child = child;
        let exit_code = child.wait().ok().and_then(|status| status.code());

        {
            let mut processes = PROCESSES.lock().unwrap();
            if let Some(handle) = processes.get_mut(&service_id_owned) {
                handle.job_handle = None;
            }
        }

        if let Err(error) =
            crate::modules::runtime::mark_service_exited(&service_id_owned, exit_code)
        {
            eprintln!("No se pudo actualizar el estado del servicio: {error}");
        }

        if let Ok(mut log_file) = std::fs::OpenOptions::new()
            .append(true)
            .open(&log_path_for_exit)
        {
            let message = match exit_code {
                Some(0) => {
                    "[Cardumen Deck] Proceso finalizado correctamente (código 0)".to_string()
                }
                Some(code) => format!("[Cardumen Deck] Proceso finalizado con código {}", code),
                None => "[Cardumen Deck] Proceso finalizado sin código de salida".to_string(),
            };
            let _ = writeln!(log_file, "\n{}", message);
            let _ = log_file.flush();
        }
    });

    Ok(process_handle)
}

pub fn stop_service(service_id: &str) -> ProcessResult<()> {
    let mut processes = PROCESSES.lock().unwrap();
    let mut handled = false;

    if let Some(handle) = processes.remove(service_id) {
        handled = true;
        if is_process_alive_with_created_at(handle.pid, handle.process_created_at) {
            let _ = Command::new("taskkill")
                .args(["/PID", &handle.pid.to_string(), "/T", "/F"])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }

        if let Some(job_handle) = handle.job_handle {
            unsafe { CloseHandle(job_handle as HANDLE) };
        }
    }

    drop(processes);

    if !handled {
        if let Ok(runtime) = load_runtime() {
            if let Some(entry) = runtime.iter().find(|entry| entry.service_id == service_id) {
                if is_process_alive_with_created_at(entry.pid, entry.process_created_at) {
                    let _ = Command::new("taskkill")
                        .args(["/PID", &entry.pid.to_string(), "/T", "/F"])
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status();
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
pub fn is_process_alive(pid: u32) -> bool {
    is_process_alive_with_created_at(pid, None)
}

pub fn get_service_pid(service_id: &str) -> Option<u32> {
    {
        let processes = PROCESSES.lock().unwrap();
        if let Some(handle) = processes.get(service_id) {
            if is_process_alive_with_created_at(handle.pid, handle.process_created_at) {
                return Some(handle.pid);
            }
        }
    }

    if let Ok(runtime) = load_runtime() {
        if let Some(entry) = runtime.iter().find(|entry| entry.service_id == service_id) {
            if is_process_alive_with_created_at(entry.pid, entry.process_created_at) {
                return Some(entry.pid);
            }
        }
    }

    None
}

pub fn is_service_alive(service_id: &str) -> bool {
    {
        let processes = PROCESSES.lock().unwrap();
        if let Some(handle) = processes.get(service_id) {
            if is_process_alive_with_created_at(handle.pid, handle.process_created_at) {
                return true;
            }
        }
    }

    if let Ok(runtime) = load_runtime() {
        if let Some(entry) = runtime.iter().find(|entry| entry.service_id == service_id) {
            return is_process_alive_with_created_at(entry.pid, entry.process_created_at);
        }
    }

    false
}

pub fn adopt_processes() -> ProcessResult<()> {
    let runtime = load_runtime()?;
    let mut processes = PROCESSES.lock().unwrap();

    for entry in runtime {
        if entry.status == "running"
            && is_process_alive_with_created_at(entry.pid, entry.process_created_at)
            && !processes.contains_key(&entry.service_id)
        {
            let handle = ProcessHandle {
                pid: entry.pid,
                job_handle: None,
                run_id: entry.run_id.clone(),
                log_path: entry.log_path.clone(),
                process_created_at: entry.process_created_at,
            };
            processes.insert(entry.service_id.clone(), handle);
        }
    }

    Ok(())
}

pub fn process_start_service(
    service_id: &str,
    cwd: &str,
    command: &str,
    args: &[String],
    env_vars: &std::collections::HashMap<String, String>,
) -> ProcessResult<ProcessHandle> {
    start_service(service_id, cwd, command, args, env_vars)
}

pub fn process_stop_service(service_id: &str) -> ProcessResult<()> {
    stop_service(service_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_empty_cwd_to_home() {
        let resolved = resolve_cwd("   ");
        assert!(resolved.is_absolute());
    }

    #[test]
    fn validates_process_identity_by_creation_time() {
        let pid = std::process::id();
        let created_at = get_process_creation_time(pid).expect("creation time");
        assert!(is_process_alive_with_created_at(pid, Some(created_at)));
        assert!(!is_process_alive_with_created_at(pid, Some(created_at + 1)));
    }

    #[test]
    fn creates_job_object_with_extended_limit_information() {
        let job = create_job_object().expect("job object should be created");
        unsafe { CloseHandle(job as HANDLE) };
    }

    #[test]
    fn starts_and_stops_service_process() {
        let service_id = format!("test-{}", Uuid::new_v4());
        let cwd = std::env::temp_dir();
        let args = vec!["-n".to_string(), "30".to_string(), "127.0.0.1".to_string()];
        let env_vars = std::collections::HashMap::new();

        let handle = start_service(&service_id, cwd.to_str().unwrap(), "ping", &args, &env_vars)
            .expect("service should start");

        assert!(is_process_alive(handle.pid));

        stop_service(&service_id).expect("service should stop");
        std::thread::sleep(std::time::Duration::from_millis(300));
        assert!(!is_process_alive(handle.pid));

        let log_contents = std::fs::read_to_string(&handle.log_path).unwrap_or_default();
        assert!(log_contents.contains("[Cardumen Deck] Iniciando"));
        assert!(log_contents.contains("[Cardumen Deck] Proceso finalizado"));

        if let Ok(log_dir) = crate::modules::config::service_log_dir(&service_id) {
            let _ = std::fs::remove_dir_all(log_dir);
        }
    }
}
