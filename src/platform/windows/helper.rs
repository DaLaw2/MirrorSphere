use crate::utils::log_entry::io::IOEntry;
use std::ffi::OsString;
use std::os::windows::prelude::*;
use std::path::PathBuf;
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{LocalFree, ERROR_SUCCESS, HLOCAL};
use windows::Win32::Security::Authorization::{
    GetNamedSecurityInfoW, SE_FILE_OBJECT, SE_OBJECT_TYPE,
};
use windows::Win32::Security::{
    CopySid, GetLengthSid, LookupAccountSidW, OWNER_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, PSID, SID_NAME_USE,
};

pub fn get_owner_psid(path: PathBuf) -> anyhow::Result<Vec<u8>> {
    let file_path_wild: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();

    let mut p_sid = PSID::default();
    let mut p_security_descriptor: PSECURITY_DESCRIPTOR = PSECURITY_DESCRIPTOR::default();

    unsafe {
        let result = GetNamedSecurityInfoW(
            PCWSTR(file_path_wild.as_ptr()),
            SE_OBJECT_TYPE(SE_FILE_OBJECT.0),
            OWNER_SECURITY_INFORMATION,
            Some(&mut p_sid),
            None,
            None,
            None,
            &mut p_security_descriptor,
        );

        if result != ERROR_SUCCESS {
            Err(IOEntry::GetMetadataFailed)?
        }

        let sid_len = GetLengthSid(p_sid);

        let mut sid_copy = vec![0u8; sid_len as usize];

        let result = CopySid(sid_len, PSID(sid_copy.as_mut_ptr() as *mut _), p_sid);
        let security_descriptor_handle = HLOCAL(p_security_descriptor.0 as *mut _);
        LocalFree(Some(security_descriptor_handle));

        result.map_err(IOEntry::GetMetadataFailed)?;

        Ok(sid_copy)
    }
}

pub fn get_owner_name(path: PathBuf) -> anyhow::Result<String> {
    let sid_vec = get_owner_psid(path)?;

    unsafe {
        let p_sid = PSID(sid_vec.as_ptr() as *mut _);

        let mut name_size: u32 = 0;
        let mut domain_size: u32 = 0;
        let mut sid_type = SID_NAME_USE::default();

        let _ = LookupAccountSidW(
            PCWSTR::null(),
            p_sid,
            None,
            &mut name_size,
            None,
            &mut domain_size,
            &mut sid_type,
        );

        let mut name_buffer = vec![0u16; name_size as usize];
        let mut domain_buffer = vec![0u16; domain_size as usize];

        let lookup_result = LookupAccountSidW(
            PCWSTR::null(),
            p_sid,
            Some(PWSTR(name_buffer.as_mut_ptr())),
            &mut name_size,
            Some(PWSTR(domain_buffer.as_mut_ptr())),
            &mut domain_size,
            &mut sid_type,
        );

        if lookup_result.is_err() {
            Err(IOEntry::GetMetadataFailed)?
        }

        if name_size > 0 {
            name_buffer.truncate(name_size as usize - 1);
        }
        if domain_size > 0 {
            domain_buffer.truncate(domain_size as usize - 1);
        }

        let account_name = OsString::from_wide(&name_buffer)
            .to_string_lossy()
            .to_string();
        let domain_name = OsString::from_wide(&domain_buffer)
            .to_string_lossy()
            .to_string();

        if domain_name.is_empty() {
            Ok(account_name)
        } else {
            Ok(format!("{}\\{}", domain_name, account_name))
        }
    }
}
