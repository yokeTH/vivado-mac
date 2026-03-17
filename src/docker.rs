#![allow(deprecated)]

use bollard::container::{
    AttachContainerOptions, Config, CreateContainerOptions, KillContainerOptions,
    LogsOptions, RemoveContainerOptions, StartContainerOptions, WaitContainerOptions,
};
use bollard::image::CreateImageOptions;
use bollard::models::{HostConfig, Mount, MountTypeEnum};
use bollard::Docker;
use crossterm::terminal;
use futures_util::StreamExt;
use std::path::Path;
use thiserror::Error;
use tokio::io::{self, AsyncWriteExt};

use crate::progress::PullProgress;

pub const IMAGE_NAME: &str = "ghcr.io/yoketh/vivado-mac";
const CONTAINER_NAME_VIVADO: &str = "vivado";
const CONTAINER_NAME_SETUP: &str = "vivado_x11";

#[derive(Error, Debug)]
pub enum DockerError {
    #[error("Docker API error: {0}")]
    Api(#[from] bollard::errors::Error),

    #[error("Container exited with non-zero status: {0}")]
    NonZeroExit(i64),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    #[allow(dead_code)]
    Other(String),
}

pub type Result<T> = std::result::Result<T, DockerError>;

/// Connect to Docker daemon (auto-detects socket path).
pub fn connect() -> Result<Docker> {
    Ok(Docker::connect_with_local_defaults()?)
}

/// Check if the image exists locally.
pub async fn image_exists(docker: &Docker, image: &str) -> bool {
    docker.inspect_image(image).await.is_ok()
}

/// Pull image with per-layer progress bars.
pub async fn pull_image(docker: &Docker, image: &str) -> Result<()> {
    let options = CreateImageOptions {
        from_image: image,
        platform: "linux/amd64",
        ..Default::default()
    };

    let mut stream = docker.create_image(Some(options), None, None);
    let mut progress = PullProgress::new();

    while let Some(result) = stream.next().await {
        match result {
            Ok(info) => progress.update(&info),
            Err(e) => {
                progress.finish();
                return Err(DockerError::Api(e));
            }
        }
    }

    progress.finish();
    Ok(())
}

/// Run the interactive setup container (install.sh).
pub async fn run_setup_container(docker: &Docker, data_dir: &Path) -> Result<()> {
    let mount_source = data_dir.to_string_lossy().to_string();

    // Remove any leftover container with same name
    let _ = docker
        .remove_container(
            CONTAINER_NAME_SETUP,
            Some(RemoveContainerOptions {
                force: true,
                ..Default::default()
            }),
        )
        .await;

    let config = Config {
        image: Some(IMAGE_NAME),
        cmd: Some(vec!["bash", "scripts/install.sh"]),
        tty: Some(true),
        open_stdin: Some(true),
        attach_stdin: Some(true),
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        host_config: Some(HostConfig {
            init: Some(true),
            auto_remove: Some(true),
            mounts: Some(vec![Mount {
                target: Some("/home/user".to_string()),
                source: Some(mount_source),
                typ: Some(MountTypeEnum::BIND),
                ..Default::default()
            }]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let options = CreateContainerOptions {
        name: CONTAINER_NAME_SETUP,
        platform: Some("linux/amd64"),
    };

    docker.create_container(Some(options), config).await?;

    // Attach before starting to not miss any output
    let attach_options = AttachContainerOptions::<String> {
        stdin: Some(true),
        stdout: Some(true),
        stderr: Some(true),
        stream: Some(true),
        ..Default::default()
    };

    let mut attach = docker
        .attach_container(CONTAINER_NAME_SETUP, Some(attach_options))
        .await?;

    docker
        .start_container(CONTAINER_NAME_SETUP, None::<StartContainerOptions<String>>)
        .await?;

    // Enter raw mode for interactive terminal
    terminal::enable_raw_mode()?;

    let result = run_attached_io(&mut attach).await;

    terminal::disable_raw_mode()?;

    // Wait for container to finish
    let mut wait_stream = docker.wait_container(
        CONTAINER_NAME_SETUP,
        Some(WaitContainerOptions {
            condition: "not-running",
        }),
    );

    // Container might already be gone (auto_remove), so ignore errors
    if let Some(Ok(wait_result)) = wait_stream.next().await
        && wait_result.status_code != 0
    {
        return Err(DockerError::NonZeroExit(wait_result.status_code));
    }

    result
}

/// Bridge stdin/stdout between host terminal and attached container.
async fn run_attached_io(
    attach: &mut bollard::container::AttachContainerResults,
) -> Result<()> {
    let mut stdout = io::stdout();
    let mut stdin = io::stdin();

    loop {
        tokio::select! {
            // Container output → host stdout
            chunk = attach.output.next() => {
                match chunk {
                    Some(Ok(output)) => {
                        stdout.write_all(&output.into_bytes()).await?;
                        stdout.flush().await?;
                    }
                    Some(Err(e)) => return Err(DockerError::Api(e)),
                    None => break, // Stream ended
                }
            }
            // Host stdin → container
            result = tokio::io::copy(&mut stdin, &mut attach.input) => {
                match result {
                    Ok(_) => break, // stdin closed
                    Err(e) => return Err(DockerError::Io(e)),
                }
            }
        }
    }

    Ok(())
}

/// Create and start the Vivado runtime container.
pub async fn start_vivado(docker: &Docker, data_dir: &Path) -> Result<()> {
    let mount_source = data_dir.to_string_lossy().to_string();

    // Remove any leftover container
    let _ = docker
        .remove_container(
            CONTAINER_NAME_VIVADO,
            Some(RemoveContainerOptions {
                force: true,
                ..Default::default()
            }),
        )
        .await;

    let config = Config {
        image: Some(IMAGE_NAME),
        cmd: Some(vec![
            "sudo", "-H", "-u", "user", "bash", "scripts/startup.sh",
        ]),
        env: Some(vec!["DISPLAY=host.docker.internal:0"]),
        host_config: Some(HostConfig {
            init: Some(true),
            auto_remove: Some(true),
            network_mode: Some("host".to_string()),
            mounts: Some(vec![Mount {
                target: Some("/home/user".to_string()),
                source: Some(mount_source),
                typ: Some(MountTypeEnum::BIND),
                ..Default::default()
            }]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let options = CreateContainerOptions {
        name: CONTAINER_NAME_VIVADO,
        platform: Some("linux/amd64"),
    };

    docker.create_container(Some(options), config).await?;

    docker
        .start_container(CONTAINER_NAME_VIVADO, None::<StartContainerOptions<String>>)
        .await?;

    Ok(())
}

/// Stream container logs to host stdout/stderr.
#[allow(dead_code)]
pub async fn stream_logs(docker: &Docker) {
    let options = LogsOptions::<String> {
        follow: true,
        stdout: true,
        stderr: true,
        ..Default::default()
    };

    let mut stream = docker.logs(CONTAINER_NAME_VIVADO, Some(options));

    while let Some(Ok(output)) = stream.next().await {
        eprint!("{output}");
    }
}

/// Wait for the Vivado container to exit. Returns the exit code.
pub async fn wait_vivado(docker: &Docker) -> Result<i64> {
    let mut stream = docker.wait_container(
        CONTAINER_NAME_VIVADO,
        Some(WaitContainerOptions {
            condition: "not-running",
        }),
    );

    match stream.next().await {
        Some(Ok(result)) => Ok(result.status_code),
        Some(Err(e)) => Err(DockerError::Api(e)),
        None => Ok(0),
    }
}

/// Kill the Vivado container.
pub async fn kill_vivado(docker: &Docker) -> Result<()> {
    docker
        .kill_container(
            CONTAINER_NAME_VIVADO,
            Some(KillContainerOptions { signal: "SIGKILL" }),
        )
        .await
        .ok(); // Ignore errors (container might not be running)
    Ok(())
}

/// Check if the Vivado container is currently running.
pub async fn is_vivado_running(docker: &Docker) -> bool {
    match docker
        .inspect_container(CONTAINER_NAME_VIVADO, None::<bollard::container::InspectContainerOptions>)
        .await
    {
        Ok(info) => info.state.and_then(|s| s.running).unwrap_or(false),
        Err(_) => false,
    }
}
