#[cfg(target_os = "linux")]
use crate::platform::linux::elevate as platform;
#[cfg(target_os = "windows")]
use crate::platform::windows::elevate as platform;
pub use privilege::user::privileged;
use std::io;

pub fn elevate() -> io::Result<()> {
    platform::elevate()
}
