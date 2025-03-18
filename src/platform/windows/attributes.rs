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

impl PartialEq for Attributes {
    fn eq(&self, other: &Self) -> bool {
        self.read_only == other.read_only
            && self.hidden == other.hidden
            && self.archive == other.archive
            && self.normal == other.normal
            && self.index == other.index
            && self.creation_time == other.creation_time
            && self.last_access_time == other.last_access_time
    }
}

pub struct Permissions {
    pub owner: Option<Vec<u8>>,
    pub dacl: Option<Vec<u8>>,
}

pub struct AdvancedPermissions {
    pub owner: Option<Vec<u8>>,
    pub dacl: Option<Vec<u8>>,
    pub primary_group: Option<Vec<u8>>,
    pub sacl: Option<Vec<u8>>,
}
