use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::process::Command;

// -- Color helpers --

pub fn info(msg: &str) {
    eprintln!("\x1b[0;34m[INFO]\x1b[0m {msg}");
}

pub fn success(msg: &str) {
    eprintln!("\x1b[0;32m[OK]\x1b[0m {msg}");
}

pub fn warn(msg: &str) {
    eprintln!("\x1b[1;33m[WARNING]\x1b[0m {msg}");
}

pub fn error(msg: &str) {
    eprintln!("\x1b[0;31m[ERROR]\x1b[0m {msg}");
}

pub fn step(msg: &str) {
    eprintln!("\x1b[0;36m[STEP]\x1b[0m {msg}");
}

pub fn debug(msg: &str) {
    eprintln!("\x1b[0;90m[DEBUG]\x1b[0m {msg}");
}

// -- Paths --

/// Returns the data directory for vivado-mac.
/// Defaults to `~/.local/share/vivado-mac/`, overridable via `VIVADO_MAC_DATA_DIR`.
pub fn data_dir() -> PathBuf {
    if let Ok(dir) = env::var("VIVADO_MAC_DATA_DIR") {
        return PathBuf::from(dir);
    }
    dirs::data_dir()
        .expect("Could not determine data directory")
        .join("vivado-mac")
}

pub fn openfpgaloader_path() -> PathBuf {
    if let Ok(path) = env::var("OPENFPGALOADER") {
        return PathBuf::from(path);
    }

    // Check if openFPGALoader is on PATH (e.g. installed via Homebrew)
    if let Ok(output) = Command::new("which").arg("openFPGALoader").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return PathBuf::from(path);
        }
    }

    error("openFPGALoader not found. Run `vivado-mac install` to install it.");
    std::process::exit(1);
}

// -- Installer version detection --

pub fn known_versions() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        ("abe838aa2e2d3d9b10fea94165e9a303", "202502"),
        ("20c806793b3ea8d79273d5138fbd195f", "202402"),
        ("8b0e99a41b851b50592d5d6ef1b1263d", "202401"),
        ("b8c785d03b754766538d6cde1277c4f0", "202302"),
    ])
}

pub fn md5_of_file(path: &PathBuf) -> Option<String> {
    let output = Command::new("md5sum").arg(path).output().ok()?;
    if !output.status.success() {
        // macOS fallback
        let output = Command::new("md5").args(["-q"]).arg(path).output().ok()?;
        let s = String::from_utf8_lossy(&output.stdout);
        return Some(s.trim().to_string());
    }
    let s = String::from_utf8_lossy(&output.stdout);
    Some(s.split_whitespace().next()?.to_string())
}

// -- X11 --

pub fn setup_x11() {
    info("Setting up X11 display...");
    let _ = Command::new("/opt/X11/bin/xhost")
        .args(["+", "localhost"])
        .env("DISPLAY", ":0")
        .status();
}

// -- XVC / openFPGALoader --

pub fn check_xvc_running() -> bool {
    let output = Command::new("pgrep")
        .args(["-f", "openFPGALoader.*--xvc"])
        .output();
    matches!(output, Ok(o) if o.status.success())
}
