mod assets;
mod cli;
mod deps;
mod docker;
mod log;
mod progress;
mod util;

use std::fs;
use std::io::{self, BufRead, Write as _};
use std::path::PathBuf;
use std::process::Stdio;

use clap::Parser;
use tokio::process::Command;
use tokio::signal;

use cli::{Cli, Commands};
use util::*;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Thread --data-dir flag into env so data_dir() picks it up
    if let Some(ref dir) = cli.data_dir {
        // SAFETY: main() runs before any threads are spawned by tokio
        unsafe { std::env::set_var("VIVADO_MAC_DATA_DIR", dir) };
    }

    match cli.command {
        Commands::Install => run_install(),
        Commands::Setup { installer } => run_setup(installer).await,
        Commands::Start { board } => run_start(&board).await,
        Commands::Xvc { board } => run_xvc_only(&board).await,
        Commands::Vivado => run_vivado_only().await,
        Commands::Program { bitstream, board } => run_program(&bitstream, &board),
        Commands::Status => run_status().await,
        Commands::Stop => run_stop().await,
        Commands::Uninstall => run_uninstall().await,
    }
}

// -- Install --

fn run_install() {
    step("Checking for Homebrew...");
    if !deps::check_brew() {
        error("Homebrew not found. Install it from https://brew.sh and try again.");
        std::process::exit(1);
    }
    success("Homebrew found.");

    step("Checking for openFPGALoader...");
    if deps::check_openfpgaloader() {
        success("openFPGALoader found.");
    } else {
        warn("openFPGALoader not found.");
        if deps::prompt_yn("  Install openFPGALoader via Homebrew? (brew install openfpgaloader)")
        {
            deps::install_openfpgaloader();
        }
    }

    step("Checking for XQuartz...");
    if deps::check_xquartz() {
        success("XQuartz found.");
    } else {
        warn("XQuartz not found.");
        if deps::prompt_yn("  Install XQuartz via Homebrew? (brew install --cask xquartz)") {
            deps::install_xquartz();
        }
    }
}

// -- Setup --

fn prompt_installer_path() -> String {
    step("Please provide the path to your Vivado installer .bin file");
    info("You can drag and drop the file into this terminal window.");
    eprint!("\n> ");
    io::stderr().flush().ok();
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line).unwrap();
    line.trim().to_string()
}

async fn run_setup(installer_arg: Option<String>) {
    let root = data_dir();
    fs::create_dir_all(&root).ok();

    log::log(&format!("setup started, data_dir={}", root.display()));

    if let Err(e) = assets::ensure_scripts(&root) {
        log::log(&format!("failed to extract scripts: {e}"));
        log::fatal(&format!("Failed to extract scripts: {e}"));
    }
    log::log("scripts extracted");

    deps::check_host_deps();

    let scripts = root.join("scripts");
    let installation_log = scripts.join("installation_location.txt");

    // Check for existing installation
    if root.join("Xilinx").is_dir() {
        warn("A previous Vivado installation was found.");
        if deps::prompt_yn("  Remove it and reinstall?") {
            step("Removing previous installation...");
            log::log("removing previous Xilinx installation");
            if let Err(e) = fs::remove_dir_all(root.join("Xilinx")) {
                log::log(&format!("failed to remove Xilinx dir: {e}"));
                log::fatal(&format!("Failed to remove Xilinx directory: {e}"));
            }
            success("Previous installation removed.");
        } else {
            info("Setup cancelled.");
            std::process::exit(0);
        }
    }

    // Resolve installer source path
    let installer_input = if let Some(path) = installer_arg {
        log::log(&format!("installer arg: {path}"));
        path
    } else if installation_log.is_file() {
        info("Found previous installer path log.");
        let content = fs::read_to_string(&installation_log).unwrap_or_default();
        let rel = content.trim().to_string();
        if !rel.is_empty() {
            let resolved = scripts.join(&rel);
            if resolved.is_file() {
                log::log(&format!("using cached installer: {}", resolved.display()));
                info(&format!("Using cached installer: {}", resolved.display()));
                return setup_verify_and_install(&resolved, &root, &installation_log).await;
            }
        }
        prompt_installer_path()
    } else {
        prompt_installer_path()
    };

    // Resolve to absolute path
    let src = PathBuf::from(&installer_input);
    if !src.is_file() {
        log::log(&format!("installer file not found: {installer_input}"));
        error(&format!("File does not exist: {installer_input}"));
        if installation_log.is_file()
            && deps::prompt_yn("  Remove cached installer path and retry?")
        {
            fs::remove_file(&installation_log).ok();
            info("Cache cleared. Please run setup again.");
        }
        log::fatal("Installer file does not exist.");
    }

    // Copy installer to data dir root
    let filename = src.file_name().unwrap();
    let dest = root.join(filename);

    if src != dest {
        step("Copying installer to data directory...");
        log::log(&format!("copying {} -> {}", src.display(), dest.display()));
        if let Err(e) = fs::copy(&src, &dest) {
            log::log(&format!("copy failed: {e}"));
            log::fatal(&format!("Failed to copy installer: {e}"));
        }
        success(&format!("Copied: {}", filename.to_string_lossy()));
    }

    // Write relative path to log
    let rel = format!("../{}", filename.to_string_lossy());
    fs::write(&installation_log, &rel).ok();

    setup_verify_and_install(&dest, &root, &installation_log).await;
}

async fn setup_verify_and_install(
    installer_file: &PathBuf,
    root: &PathBuf,
    _installation_log: &PathBuf,
) {
    // Verify installer hash
    step("Verifying installer...");
    let hash = match md5_of_file(installer_file) {
        Some(h) => h,
        None => {
            log::log(&format!(
                "md5 hash failed for {}",
                installer_file.display()
            ));
            log::fatal("Failed to compute MD5 hash of installer.");
        }
    };
    log::log(&format!("installer hash: {hash}"));

    let versions = known_versions();
    let version = match versions.get(hash.as_str()) {
        Some(v) => *v,
        None => {
            log::log(&format!("unknown installer hash: {hash}"));
            log::fatal("Installer hash does not match. Make sure you downloaded the Linux installer for a supported version.");
        }
    };

    if version == "202401" {
        log::log(&format!("unsupported version: {version}"));
        log::fatal(&format!(
            "Version {version} is not supported. Please use the latest version of the year."
        ));
    }

    log::log(&format!("installer version: {version}"));
    info(&format!("Installer version: {version}"));

    // Connect to Docker
    let docker = match docker::connect() {
        Ok(d) => d,
        Err(e) => {
            log::log(&format!("docker connect failed: {e}"));
            log::fatal(&format!("Failed to connect to Docker: {e}"));
        }
    };

    // Pull Docker image if needed
    step("Checking Docker image...");
    if !docker::image_exists(&docker, docker::IMAGE_NAME).await {
        step("Pulling Docker image...");
        log::log(&format!("pulling image {}", docker::IMAGE_NAME));
        if let Err(e) = docker::pull_image(&docker, docker::IMAGE_NAME).await {
            log::log(&format!("image pull failed: {e}"));
            log::fatal(&format!("Docker image pull failed: {e}"));
        }
        log::log("image pulled");
        success("Docker image pulled.");
    } else {
        debug("Docker image already exists.");
    }

    // Run install inside container
    step("Starting container for Vivado installation...");
    log::log("starting setup container");
    match docker::run_setup_container(&docker, root).await {
        Ok(()) => {
            log::log("installation complete");
            success("Vivado installation complete.");
        }
        Err(e) => {
            log::log(&format!("installation failed: {e}"));
            log::fatal(&format!("Installation failed: {e}"));
        }
    }
}

// -- Start (XVC + Vivado) --

async fn run_start(board: &str) {
    let root = data_dir();
    if let Err(e) = assets::ensure_scripts(&root) {
        error(&format!("Failed to extract scripts: {e}"));
        std::process::exit(1);
    }
    deps::check_host_deps();

    let docker = match docker::connect() {
        Ok(d) => d,
        Err(e) => {
            error(&format!("Failed to connect to Docker: {e}"));
            std::process::exit(1);
        }
    };

    let mut xvc = match start_xvc_process(board) {
        Ok(child) => child,
        Err(e) => {
            error(&format!("Failed to start XVC server: {e}"));
            std::process::exit(1);
        }
    };

    info("Waiting for XVC server to initialize...");
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    setup_x11();
    if let Err(e) = docker::start_vivado(&docker, &root).await {
        error(&format!("Failed to start Vivado container: {e}"));
        let _ = xvc.kill().await;
        std::process::exit(1);
    }

    info("Vivado container started. Press Ctrl-C to stop.");

    tokio::select! {
        _ = signal::ctrl_c() => {
            info("Ctrl-C received, shutting down...");
        }
        result = docker::wait_vivado(&docker) => {
            match result {
                Ok(code) => info(&format!("Vivado container exited (code {code}).")),
                Err(e) => warn(&format!("Error waiting for container: {e}")),
            }
        }
    }

    info("Shutting down...");
    docker::kill_vivado(&docker).await.ok();
    let _ = xvc.kill().await;
    let _ = xvc.wait().await;
    success("All services stopped.");
}

// -- XVC only --

async fn run_xvc_only(board: &str) {
    let mut xvc = match start_xvc_process(board) {
        Ok(child) => child,
        Err(e) => {
            error(&format!("Failed to start XVC server: {e}"));
            std::process::exit(1);
        }
    };

    info("XVC server running. Press Ctrl-C to stop.");

    tokio::select! {
        _ = signal::ctrl_c() => {
            info("Ctrl-C received.");
        }
        status = xvc.wait() => {
            match status {
                Ok(s) => warn(&format!("XVC server exited with status: {s}")),
                Err(e) => error(&format!("Error checking XVC status: {e}")),
            }
        }
    }

    let _ = xvc.kill().await;
    let _ = xvc.wait().await;
    success("XVC server stopped.");
}

// -- Vivado only --

async fn run_vivado_only() {
    let root = data_dir();
    if let Err(e) = assets::ensure_scripts(&root) {
        error(&format!("Failed to extract scripts: {e}"));
        std::process::exit(1);
    }
    deps::check_host_deps();

    let docker = match docker::connect() {
        Ok(d) => d,
        Err(e) => {
            error(&format!("Failed to connect to Docker: {e}"));
            std::process::exit(1);
        }
    };

    setup_x11();
    if let Err(e) = docker::start_vivado(&docker, &root).await {
        error(&format!("Failed to start Vivado container: {e}"));
        std::process::exit(1);
    }

    info("Vivado container started. Press Ctrl-C to stop.");

    tokio::select! {
        _ = signal::ctrl_c() => {
            info("Ctrl-C received, shutting down...");
        }
        result = docker::wait_vivado(&docker) => {
            match result {
                Ok(code) => info(&format!("Vivado container exited (code {code}).")),
                Err(e) => warn(&format!("Error waiting for container: {e}")),
            }
        }
    }

    info("Shutting down...");
    docker::kill_vivado(&docker).await.ok();
    success("Vivado container stopped.");
}

// -- Program --

fn run_program(bitstream: &str, board: &str) {
    let loader = openfpgaloader_path();
    info(&format!("Programming {board} with: {bitstream}"));

    let status = std::process::Command::new(&loader)
        .args(["-b", board, bitstream])
        .status();

    match status {
        Ok(s) if s.success() => success("Programming complete."),
        Ok(s) => {
            error(&format!("Programming failed with status: {s}"));
            std::process::exit(1);
        }
        Err(e) => {
            error(&format!("Failed to run openFPGALoader: {e}"));
            std::process::exit(1);
        }
    }
}

// -- Status --

async fn run_status() {
    let docker = match docker::connect() {
        Ok(d) => d,
        Err(e) => {
            error(&format!("Failed to connect to Docker: {e}"));
            std::process::exit(1);
        }
    };

    let xvc = check_xvc_running();
    let vivado = docker::is_vivado_running(&docker).await;

    println!("Service Status:");
    println!(
        "  XVC Server:       {}",
        if xvc {
            "\x1b[0;32mrunning\x1b[0m"
        } else {
            "\x1b[0;31mstopped\x1b[0m"
        }
    );
    println!(
        "  Vivado Container: {}",
        if vivado {
            "\x1b[0;32mrunning\x1b[0m"
        } else {
            "\x1b[0;31mstopped\x1b[0m"
        }
    );
}

// -- Stop --

async fn run_stop() {
    let docker = match docker::connect() {
        Ok(d) => d,
        Err(e) => {
            error(&format!("Failed to connect to Docker: {e}"));
            std::process::exit(1);
        }
    };

    info("Stopping Vivado container...");
    docker::kill_vivado(&docker).await.ok();
    success("Vivado container stopped.");
}

// -- Uninstall --

async fn run_uninstall() {
    let root = data_dir();

    info(&format!("Data directory: {}", root.display()));

    // Stop running container first
    if let Ok(docker) = docker::connect() {
        if docker::is_vivado_running(&docker).await {
            warn("Vivado container is running.");
            if deps::prompt_yn("  Stop it before uninstalling?") {
                docker::kill_vivado(&docker).await.ok();
                success("Vivado container stopped.");
            } else {
                error("Cannot uninstall while Vivado is running.");
                std::process::exit(1);
            }
        }
    }

    // Remove data directory
    if root.is_dir() {
        warn(&format!(
            "This will remove the entire data directory: {}",
            root.display()
        ));
        info("This includes: Vivado installation (~40GB), scripts, installer binary, and logs.");
        if deps::prompt_yn("  Remove data directory?") {
            step("Removing data directory...");
            if let Err(e) = fs::remove_dir_all(&root) {
                error(&format!("Failed to remove data directory: {e}"));
                std::process::exit(1);
            }
            success("Data directory removed.");
        }
    } else {
        info("Data directory does not exist. Nothing to remove.");
    }

    // Remove Docker image
    if let Ok(docker) = docker::connect() {
        if docker::image_exists(&docker, docker::IMAGE_NAME).await
            && deps::prompt_yn(&format!(
                "  Remove Docker image {}?",
                docker::IMAGE_NAME
            ))
        {
            step("Removing Docker image...");
            match docker.remove_image(docker::IMAGE_NAME, None::<bollard::query_parameters::RemoveImageOptions>, None).await {
                Ok(_) => success("Docker image removed."),
                Err(e) => warn(&format!("Failed to remove Docker image: {e}")),
            }
        }
    }

    success("Uninstall complete.");
}

// -- Helpers --

fn start_xvc_process(board: &str) -> io::Result<tokio::process::Child> {
    let loader = openfpgaloader_path();
    info(&format!("Starting XVC server for board: {board}"));
    debug(&format!("{}", loader.display()));

    Command::new(&loader)
        .args(["-b", board, "--xvc"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
}
