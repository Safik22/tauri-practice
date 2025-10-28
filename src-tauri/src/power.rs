use thiserror::Error;
use windows::Win32::System::{
    Power::{SetSuspendState},
    Shutdown::{ExitWindowsEx, EWX_REBOOT, EWX_POWEROFF, EWX_FORCEIFHUNG, SHUTDOWN_REASON},
};

use crate::shutdown_privilege::enable_shutdown_privilege;

/// Ошибки управления питанием.
#[derive(Debug, Error)]
enum PowerError {
    #[error("Ошибка вызова SetSuspendState: {0}")]
    WinApi(String),
}

/// Перевести систему в спящий режим.
///
/// - `hibernate = false` — обычный "сон" (Suspend).
/// - `hibernate = true` — гибернация (Hibernate).
fn sleep(hibernate: bool) -> Result<(), PowerError> {
    unsafe {
        let result = SetSuspendState(hibernate, false, false);
        if result {
            Ok(())
        } else {
            Err(PowerError::WinApi("SetSuspendState вернул FALSE".into()))
        }
    }
}

/// Перезагрузить систему.
///
/// ⚠️ Требует прав администратора.
fn reboot() -> Result<(), PowerError> {
    enable_shutdown_privilege().map_err(|e| PowerError::WinApi(e))?;
    unsafe {
        let result = ExitWindowsEx(EWX_REBOOT | EWX_FORCEIFHUNG, SHUTDOWN_REASON(0));
        if result.is_ok() {
            Ok(())
        } else {
            Err(PowerError::WinApi("ExitWindowsEx (reboot) вернул FALSE".into()))
        }
    }
}

/// Выключить систему.
///
/// ⚠️ Требует прав администратора.
fn shutdown() -> Result<(), PowerError> {
    enable_shutdown_privilege().map_err(|e| PowerError::WinApi(e))?;
    unsafe {
        let result = ExitWindowsEx(EWX_POWEROFF | EWX_FORCEIFHUNG, SHUTDOWN_REASON(0));
        if result.is_ok() {
            Ok(())
        } else {
            Err(PowerError::WinApi("ExitWindowsEx (shutdown) вернул FALSE".into()))
        }
    }
}

#[tauri::command]
pub fn sleep_command(hibernate: bool) -> Result<(), String> {
    sleep(hibernate).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reboot_command() -> Result<(), String> {
    reboot().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn shutdown_command() -> Result<(), String> {
    shutdown().map_err(|e| e.to_string())
}