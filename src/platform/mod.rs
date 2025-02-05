mod linux;
mod windows;

#[cfg(target_os="windows")]
pub use windows::*;
#[cfg(target_os="linux")]
pub use linux::*;
