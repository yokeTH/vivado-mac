use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

const SCRIPTS: &[(&str, &str)] = &[
    ("install.sh", include_str!("../scripts/install.sh")),
    ("startup.sh", include_str!("../scripts/startup.sh")),
    ("headers.sh", include_str!("../scripts/headers.sh")),
    (
        "auth_token_gen.exp",
        include_str!("../scripts/auth_token_gen.exp"),
    ),
    (
        "vivado_settings_202502.txt",
        include_str!("../scripts/vivado_settings_202502.txt"),
    ),
    (
        "vivado_settings_202402.txt",
        include_str!("../scripts/vivado_settings_202402.txt"),
    ),
    (
        "vivado_settings_202302.txt",
        include_str!("../scripts/vivado_settings_202302.txt"),
    ),
];

/// Ensure all embedded scripts are extracted to `data_dir/scripts/`.
/// Re-extracts when the CLI version changes.
pub fn ensure_scripts(data_dir: &Path) -> io::Result<()> {
    let scripts_dir = data_dir.join("scripts");
    fs::create_dir_all(&scripts_dir)?;

    let version_file = scripts_dir.join(".version");
    let current_version = env!("CARGO_PKG_VERSION");

    let needs_extract = match fs::read_to_string(&version_file) {
        Ok(v) => v.trim() != current_version,
        Err(_) => true,
    };

    if !needs_extract {
        return Ok(());
    }

    for (name, content) in SCRIPTS {
        let path = scripts_dir.join(name);
        fs::write(&path, content)?;

        if name.ends_with(".sh") || name.ends_with(".exp") {
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755))?;
        }
    }

    fs::write(&version_file, current_version)?;

    Ok(())
}
