use std::time::SystemTime;
use windows_acl::acl::ACL;

#[derive(Clone, PartialEq, Eq)]
pub struct Attributes {
    pub read_only: bool,
    pub hidden: bool,
    pub archive: bool,
    pub normal: bool,
    pub index: bool,
    pub creation_time: SystemTime,
    pub last_access_time: SystemTime,
    pub change_time: SystemTime,
}

pub struct PermissionAttributes {
    pub owner: Vec<u8>,
    pub access_control_list: ACL,
}
