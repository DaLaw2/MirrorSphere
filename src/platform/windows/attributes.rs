use std::time::SystemTime;
use windows_acl::acl::ACL;

pub struct Attributes {
    pub read_only: bool,
    pub hidden: bool,
    pub system: bool,
    pub archive: bool,
    pub creation_time: SystemTime,
    pub last_access_time: SystemTime,
    pub change_time: SystemTime,
}

pub struct AdvancedAttributes {
    pub read_only: bool,
    pub hidden: bool,
    pub system: bool,
    pub archive: bool,
    pub compression: bool,
    pub encryption: bool,
    pub index: bool,
    pub creation_time: SystemTime,
    pub last_access_time: SystemTime,
    pub change_time: SystemTime,
    pub owner: String,
    pub access_control_list: ACL,
}
