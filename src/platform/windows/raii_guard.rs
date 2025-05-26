use crate::model::error::io::IOError;
use fs4::fs_std::FileExt;
use std::ops::Deref;
use std::path::PathBuf;
use windows::Win32::Foundation::{LocalFree, HLOCAL};
use windows::Win32::Security::PSECURITY_DESCRIPTOR;

pub struct SecurityDescriptorGuard {
    descriptor: PSECURITY_DESCRIPTOR,
}

impl SecurityDescriptorGuard {
    pub fn new(descriptor: PSECURITY_DESCRIPTOR) -> Self {
        Self { descriptor }
    }

    pub fn get(&self) -> PSECURITY_DESCRIPTOR {
        self.descriptor
    }
}

impl Deref for SecurityDescriptorGuard {
    type Target = PSECURITY_DESCRIPTOR;

    fn deref(&self) -> &Self::Target {
        &self.descriptor
    }
}

impl Drop for SecurityDescriptorGuard {
    fn drop(&mut self) {
        unsafe {
            if !self.descriptor.is_invalid() {
                let _ = LocalFree(Some(HLOCAL(self.descriptor.0)));
            }
        }
    }
}
