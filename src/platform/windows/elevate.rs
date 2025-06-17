use crate::model::error::misc::MiscError;
use crate::model::error::Error;
use crate::model::error::system::SystemError;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::{env, mem};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, GetLastError, HANDLE, LUID};
use windows::Win32::Security::{
    AdjustTokenPrivileges, LookupPrivilegeValueW, LUID_AND_ATTRIBUTES, SE_PRIVILEGE_ENABLED,
    SE_SECURITY_NAME, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TOKEN_QUERY,
};
use windows::Win32::System::Com::{
    CoInitializeEx, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::Win32::UI::Shell::{ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW};
use windows::Win32::UI::WindowsAndMessaging::SW_NORMAL;

pub fn elevate() -> Result<(), Error> {
    let exe = env::current_exe()
        .map_err(|_| SystemError::UnknownError)?;
    let args: Vec<String> = env::args().skip(1).collect();
    let params = args.join(" ");

    let file = OsStr::new(exe.to_str().unwrap())
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();

    let params = OsStr::new(&params)
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();

    unsafe {
        win_runas(file, params)?;
        adjust_token_privileges()
    }
}

unsafe fn win_runas(cmd: Vec<u16>, args: Vec<u16>) -> Result<(), Error> {
    unsafe {
        let mut sei: SHELLEXECUTEINFOW = mem::zeroed();
        let verb = "runas\0".encode_utf16().collect::<Vec<u16>>();

        if CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE).is_err() {
            Err(SystemError::RunAsAdminFailed)?
        }

        sei.fMask = SEE_MASK_NOCLOSEPROCESS;
        sei.cbSize = size_of::<SHELLEXECUTEINFOW>() as u32;
        sei.lpVerb = PCWSTR(verb.as_ptr());
        sei.lpFile = PCWSTR(cmd.as_ptr());
        sei.lpParameters = PCWSTR(args.as_ptr());
        sei.nShow = SW_NORMAL.0;

        if ShellExecuteExW(&mut sei).is_err() || sei.hProcess.is_invalid() {
            Err(SystemError::RunAsAdminFailed)?
        }

        Ok(())
    }
}

unsafe fn adjust_token_privileges() -> Result<(), Error> {
    unsafe {
        let mut token_handle: HANDLE = HANDLE::default();

        OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token_handle,
        )
        .map_err(|_| SystemError::AdjustTokenPrivilegesFailed)?;

        let mut luid = LUID {
            LowPart: 0,
            HighPart: 0,
        };

        LookupPrivilegeValueW(PCWSTR::null(), SE_SECURITY_NAME, &mut luid)
            .map_err(|_| SystemError::AdjustTokenPrivilegesFailed)?;

        let mut token_privilege = TOKEN_PRIVILEGES {
            PrivilegeCount: 1,
            Privileges: [LUID_AND_ATTRIBUTES {
                Luid: luid,
                Attributes: SE_PRIVILEGE_ENABLED,
            }],
        };

        AdjustTokenPrivileges(
            token_handle,
            false,
            Some(&mut token_privilege),
            0,
            None,
            None,
        )
        .map_err(|_| SystemError::AdjustTokenPrivilegesFailed)?;

        CloseHandle(token_handle).map_err(|_| MiscError::ObjectFreeFailed)?;

        if GetLastError().is_err() {
            Err(SystemError::AdjustTokenPrivilegesFailed)?
        }

        Ok(())
    }
}
