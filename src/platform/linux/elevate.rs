use crate::model::error::Error;
use crate::model::error::system::SystemError;
use std::process::Command;
use std::env;

#[allow(dead_code)]
pub fn elevate() -> Result<(), Error> {
    let exe = env::current_exe()
        .map_err(|_| SystemError::RunAsAdminFailed)?;
    let args: Vec<String> = env::args().skip(1).collect();

    Command::new("sudo")
        .arg("--")
        .arg(exe)
        .args(&args)
        .status()
        .map_err(|_| SystemError::RunAsAdminFailed)?;

    Ok(())
}
