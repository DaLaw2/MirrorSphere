use std::time::SystemTime;
use libc::{gid_t, uid_t, mode_t};

#[derive(Debug, Clone, PartialEq)]
pub struct Attributes {
    pub attributes: u32,
    pub creation_time: SystemTime,
    pub last_access_time: SystemTime,
    pub change_time: SystemTime,
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

impl Permissions {
    pub fn new(uid: uid_t, gid: gid_t, mode: mode_t) -> Self {
        Self {
            uid,
            gid,
            mode: mode as u32,
            is_sticky: (mode & libc::S_ISVTX as mode_t) != 0,
            is_setuid: (mode & libc::S_ISUID as mode_t) != 0,
            is_setgid: (mode & libc::S_ISGID as mode_t) != 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExtendedAttribute {
    pub name: String,
    pub value: Vec<u8>,
}

impl ExtendedAttribute {
    pub fn new(name: String, value: Vec<u8>) -> Self {
        Self { name, value }
    }
}
