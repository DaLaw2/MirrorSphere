use libc::{gid_t, uid_t};
use std::time::SystemTime;

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
            && self.creation_time == other.creation_time
            && self.change_time == other.change_time
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Permissions {
    pub uid: uid_t,
    pub gid: gid_t,
    pub mode: u32,
    pub is_sticky: bool,
    pub is_setuid: bool,
    pub is_setgid: bool,
}
