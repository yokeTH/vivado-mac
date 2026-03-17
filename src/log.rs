use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::util::data_dir;

/// Returns the path to the setup log file inside the data directory.
pub fn setup_log_path() -> PathBuf {
    data_dir().join("setup.log")
}

/// Append a timestamped line to the setup log.
pub fn log(msg: &str) {
    let path = setup_log_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    let timestamp = humantime(SystemTime::now());
    let line = format!("[{timestamp}] {msg}\n");
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = f.write_all(line.as_bytes());
    }
}

/// Print error with log location and issue link, then exit.
pub fn fatal(msg: &str) -> ! {
    crate::util::error(msg);
    let log_path = setup_log_path();
    if log_path.exists() {
        eprintln!(
            "\x1b[0;90m  Log file: {}\x1b[0m",
            log_path.display()
        );
    }
    eprintln!(
        "\x1b[0;90m  If this looks like a bug, please open an issue: https://github.com/yoketh/vivado-mac/issues\x1b[0m"
    );
    std::process::exit(1);
}

/// Simple timestamp formatting without pulling in chrono.
fn humantime(t: SystemTime) -> String {
    let d = t
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = d.as_secs();
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    format!("{h:02}:{m:02}:{s:02}")
}
