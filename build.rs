#[cfg(target_os = "windows")]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/icon.ico")
        .set("InternalName", "MirrorSphere.exe")
        .set_version_info(winres::VersionInfo::PRODUCTVERSION, 0x0001000000000000)
        .set_language(0x0409);
    if let Err(err) = res.compile() {
        eprintln!("winres error: {err}");
    }
}

#[cfg(target_os = "linux")]
fn main() {
}
