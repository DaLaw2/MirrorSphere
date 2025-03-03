use std::time::SystemTime;
use windows_acl::acl::ACL;

#[derive(Clone, PartialEq, Eq)]
pub struct Attributes {
    pub read_only: bool,
    pub hidden: bool,
    pub system: bool,
    pub archive: bool,
    pub creation_time: SystemTime,
    pub last_access_time: SystemTime,
    pub change_time: SystemTime,
}

#[derive(Clone, PartialEq, Eq)]
pub struct AdvancedAttributes {
    pub read_only: bool,
    pub hidden: bool,
    pub system: bool,
    pub archive: bool,
    pub compression: bool,
    pub index: bool,
    pub encryption: bool,
    pub creation_time: SystemTime,
    pub last_access_time: SystemTime,
    pub change_time: SystemTime,
}

pub struct PermissionAttributes {
    pub owner: String,
    pub access_control_list: ACL,
}
