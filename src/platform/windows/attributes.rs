use crate::platform::raii_guard::SecurityDescriptorGuard;
use std::time::SystemTime;
use windows::Win32::Security::{ACL, PSID};

#[derive(Debug, Clone, Eq)]
pub struct Attributes {
    pub attributes: u32,
    pub creation_time: SystemTime,
    pub last_access_time: SystemTime,
    pub change_time: SystemTime,
}

impl PartialEq for Attributes {
    fn eq(&self, other: &Self) -> bool {
        self.attributes == other.attributes
    }
}

#[derive(Debug)]
pub struct Permissions {
    pub owner: PSID,
    pub primary_group: PSID,
    pub dacl: *mut ACL,
    pub sacl: *mut ACL,
    pub security_descriptor: SecurityDescriptorGuard,
}

unsafe impl Send for Permissions {}
