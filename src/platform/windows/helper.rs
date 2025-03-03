use std::ffi::OsString;
use std::os::windows::prelude::*;
use std::path::{Path, PathBuf};
use std::ptr;
use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::Security::*,
    Win32::Security::Authorization,
    Win32::Storage::FileSystem::*,
    Win32::System::Memory::*,
};
use windows::Win32::Security::Authorization::{GetNamedSecurityInfoW, SE_FILE_OBJECT, SE_OBJECT_TYPE};
use crate::utils::log_entry::io::IOEntry;

pub fn get_security_descriptor(path: PathBuf) -> anyhow::Result<()> {
    let file_path_wild: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();

    let p_owner: *mut PSID = ptr::null_mut();
    let mut p_security_descriptor: PSECURITY_DESCRIPTOR = PSECURITY_DESCRIPTOR::default();

    unsafe {
        let result = GetNamedSecurityInfoW(
            PCWSTR(file_path_wild.as_ptr()),
            SE_OBJECT_TYPE(SE_FILE_OBJECT.0),
            OWNER_SECURITY_INFORMATION,
            Some(p_owner),
            None,
            None,
            None,
            &mut p_security_descriptor,
        );

        if result != ERROR_SUCCESS {
            Err(IOEntry::GetMetadataFailed)?
        }

        LookupAccountSidW()
    }

    Ok(())
}

fn lookup_account_sid(sid: PSID) -> Result<(String, String)> {

}
