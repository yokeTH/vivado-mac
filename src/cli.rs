use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "vivado-mac", about = "Vivado on macOS orchestrator")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Override the data directory (default: ~/.local/share/vivado-mac)
    #[arg(long, global = true, env = "VIVADO_MAC_DATA_DIR")]
    pub data_dir: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install host dependencies (openFPGALoader, XQuartz)
    Install,
    /// Setup Vivado: pull Docker image and install Vivado from installer binary
    Setup {
        /// Path to the Vivado installer .bin file
        installer: Option<String>,
    },
    /// Start XVC server and Vivado container
    Start {
        /// FPGA board name
        #[arg(short, long, default_value = "basys3")]
        board: String,
    },
    /// Start only the XVC server (openFPGALoader --xvc)
    Xvc {
        /// FPGA board name
        #[arg(short, long, default_value = "basys3")]
        board: String,
    },
    /// Start only the Vivado Docker container
    Vivado,
    /// Program bitstream directly to the FPGA board
    Program {
        /// Path to the bitstream file (.bit)
        bitstream: String,
        /// FPGA board name
        #[arg(short, long, default_value = "basys3")]
        board: String,
    },
    /// Show status of XVC server and Vivado container
    Status,
    /// Stop the Vivado container
    Stop,
    /// Remove Vivado installation, data directory, and optionally Docker image
    Uninstall,
}
