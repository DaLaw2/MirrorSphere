use std::process::Command;
use std::{env, io};

pub fn elevate() -> io::Result<()> {
    let exe = env::current_exe()?;
    let args: Vec<String> = env::args().skip(1).collect();

    Command::new("sudo")
        .arg("--")
        .arg(exe)
        .args(&args)
        .status()?;

    Ok(())
}
