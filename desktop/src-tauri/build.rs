fn main() {
    // macOS: compile COSOUND's isolated Audio Unit host (auhost/main.swift) with the Command Line Tools.
    // It is placed in binaries/ for the app bundle (tauri.macos.conf.json → externalBin) and its path is
    // passed to the code for `cargo test` and development builds.
    println!("cargo:rerun-if-changed=auhost/main.swift");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        let target = std::env::var("TARGET").unwrap();
        let dir = std::path::Path::new("binaries");
        let _ = std::fs::create_dir_all(dir);
        let out = dir.join(format!("cosound-auhost-{target}"));
        let arch = if target.starts_with("aarch64") { "arm64" } else { "x86_64" };
        let status = std::process::Command::new("xcrun")
            .args(["swiftc", "-O", "-target", &format!("{arch}-apple-macos12.0"), "auhost/main.swift", "-o"])
            .arg(&out)
            .status();
        match status {
            Ok(s) if s.success() => {
                let abs = std::fs::canonicalize(&out).unwrap();
                println!("cargo:rustc-env=COSOUND_AUHOST_BUILT={}", abs.display());
            }
            _ => println!("cargo:warning=cosound-auhost was not built (Swift compiler missing?): plug-in hosting will be unavailable"),
        }
    }
    tauri_build::build()
}
