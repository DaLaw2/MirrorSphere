use std::{env, io};
pub use privilege::user::privileged;

pub fn elevate() -> io::Result<()> {
    #[cfg(target_os = "windows")]
    return windows::elevate();
    #[cfg(not(target_os = "windows"))]
    return unix::elevate();
}

#[cfg(target_os = "windows")]
mod windows {
    use super::*;
    use std::ffi::OsStr;
    use std::mem;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use windows_sys::Win32::System::Com::*;
    use windows_sys::Win32::UI::Shell::*;
    use windows_sys::Win32::UI::WindowsAndMessaging::*;

    pub fn elevate() -> io::Result<()> {
        let exe = env::current_exe()?;
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

        unsafe { win_runas(file.as_ptr(), params.as_ptr()) }
    }

    unsafe fn win_runas(cmd: *const u16, args: *const u16) -> io::Result<()> {
        let mut sei: SHELLEXECUTEINFOW = mem::zeroed();
        let verb = "runas\0".encode_utf16().collect::<Vec<u16>>();

        CoInitializeEx(
            ptr::null(),
            (COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) as u32,
        );

        sei.fMask = SEE_MASK_NOCLOSEPROCESS;
        sei.cbSize = size_of::<SHELLEXECUTEINFOW>() as _;
        sei.lpVerb = verb.as_ptr();
        sei.lpFile = cmd;
        sei.lpParameters = args;
        sei.nShow = SW_NORMAL;

        if ShellExecuteExW(&mut sei) == 0 || sei.hProcess == ptr::null_mut() {
            return Err(io::Error::last_os_error());
        }

        Ok(())
    }
}

#[cfg(not(target_os = "windows"))]
mod unix {
    use super::*;
    use std::process::Command;

    pub fn elevate() -> io::Result<()> {
        let exe = env::current_exe()?;
        let args: Vec<String> = env::args().skip(1).collect();

        Command::new("sudo")
            .arg("--")
            .arg(exe)
            .args(&args)
            .status()?;

        Ok(())
    }
}
