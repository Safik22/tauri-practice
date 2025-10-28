use windows::Win32::{
    Foundation::HANDLE,
    Security::{
        AdjustTokenPrivileges, LookupPrivilegeValueW, SE_PRIVILEGE_ENABLED,
        TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TOKEN_QUERY,
    },
    System::{
        Threading::{GetCurrentProcess, OpenProcessToken},
    },
};

use windows::core::PCWSTR;


pub fn enable_shutdown_privilege() -> Result<(), String> {
    unsafe {
        let mut token: HANDLE = HANDLE::default();
        // Получаем токен текущего процесса
        OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token,
        )
        .map_err(|_| "Не удалось открыть токен процесса".to_string())?;

        // Получаем LUID для привилегии SeShutdownPrivilege
        let mut luid = Default::default();
        let name = widestring::U16CString::from_str("SeShutdownPrivilege").unwrap();
        LookupPrivilegeValueW(
            None,
            PCWSTR(name.as_ptr()),
            &mut luid,
        )
        .map_err(|_| "Не удалось найти LUID привилегии SeShutdownPrivilege".to_string())?;

        let tp = TOKEN_PRIVILEGES {
            PrivilegeCount: 1,
            Privileges: [windows::Win32::Security::LUID_AND_ATTRIBUTES {
                Luid: luid,
                Attributes: SE_PRIVILEGE_ENABLED,
            }],
        };

        AdjustTokenPrivileges(
            token,
            false,
            Some(&tp as *const _),
            0,
            None,
            None,
        )
        .map_err(|_| "Не удалось включить привилегию SeShutdownPrivilege".to_string())?;

        // Проверяем, что ошибка последнего вызова AdjustTokenPrivileges равна ERROR_SUCCESS
        if windows::Win32::Foundation::GetLastError().0 != 0 {
            return Err("AdjustTokenPrivileges вернул ошибку".to_string());
        }
    }
    Ok(())
}