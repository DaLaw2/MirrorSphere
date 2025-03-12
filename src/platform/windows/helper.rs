use crate::utils::log_entry::io::IOEntry;
use crate::utils::log_entry::system::SystemEntry;
use chrono::{DateTime, Datelike, Timelike};
use std::ffi::OsString;
use std::os::windows::prelude::*;
use std::path::PathBuf;
use std::time::SystemTime;
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{LocalFree, ERROR_SUCCESS, FILETIME, HLOCAL, SYSTEMTIME};
use windows::Win32::Security::Authorization::{
    GetNamedSecurityInfoW, SE_FILE_OBJECT, SE_OBJECT_TYPE,
};
use windows::Win32::Security::{
    CopySid, GetLengthSid, LookupAccountSidW, OWNER_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR,
    PSID, SID_NAME_USE,
};
use windows::Win32::System::Time::SystemTimeToFileTime;

