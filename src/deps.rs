use std::io::{self, BufRead, Write};
use std::process::Command;

use crate::util::*;

pub fn check_brew() -> bool {
    Command::new("which")
        .arg("brew")
        .output()
        .is_ok_and(|o| o.status.success())
}

pub fn check_openfpgaloader() -> bool {
    Command::new("which")
        .arg("openFPGALoader")
        .output()
        .is_ok_and(|o| o.status.success())
}

pub fn check_xquartz() -> bool {
    std::path::Path::new("/opt/X11/bin/xhost").exists()
}

pub fn install_openfpgaloader() {
    let status = Command::new("brew")
        .args(["install", "openfpgaloader"])
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status();

    match status {
        Ok(s) if s.success() => success("openFPGALoader installed."),
        Ok(s) => {
            error(&format!("brew install failed with status: {s}"));
            std::process::exit(1);
        }
        Err(e) => {
            error(&format!("Failed to run brew: {e}"));
            std::process::exit(1);
        }
    }
}

pub fn install_xquartz() {
    let status = Command::new("brew")
        .args(["install", "--cask", "xquartz"])
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status();

    match status {
        Ok(s) if s.success() => success("XQuartz installed."),
        Ok(s) => {
            error(&format!("brew install failed with status: {s}"));
            std::process::exit(1);
        }
        Err(e) => {
            error(&format!("Failed to run brew: {e}"));
            std::process::exit(1);
        }
    }
}

pub fn prompt_yn(msg: &str) -> bool {
    eprint!("{msg} [y/N] ");
    io::stderr().flush().ok();
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line).unwrap_or(0);
    matches!(line.trim(), "y" | "Y" | "yes" | "Yes")
}

/// Warn if host dependencies are missing. Does not block execution.
pub fn check_host_deps() {
    if !check_openfpgaloader() {
        warn("openFPGALoader not found. Run `vivado-mac install` to install it.");
    }
    if !check_xquartz() {
        warn("XQuartz not found. Run `vivado-mac install` to install it.");
    }
}
