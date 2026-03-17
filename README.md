
> [!NOTE]  
> Tested on MacOS 15 Sequoia, MacOS 26 Tahoe.
> MacOS 14 is not supported.

# Vivado on macOS via Docker

This repository provides a solution to run Xilinx Vivado on macOS using Docker containerization.
## Support Version
- 2025.2
- 2024.2
- 2023.2

## Normal Vivado Workflow

The typical FPGA development workflow in Vivado consists of:
1. RTL Design (Verilog/VHDL)
2. Synthesis
3. Implementation
4. Generate Bitstream
5. Program to [Basys3](https://digilent.com/reference/_media/basys3:basys3_rm.pdf?srsltid=AfmBOorSKF2T_MfS024F4IiVmQr1ViDkssoCMtlG48_RoII45ntqSTt2) Board

## Table of Contents
1. [Prerequisites](#prerequisites)
2. [Installation](#installation)
3. [Usage](#usage)
4. [Troubleshooting](#troubleshooting)

## Prerequisites
0. **Disk Space**
    - Ensure you have at least 120GB of free disk space:
        - ~80GB for Vivado download and Extract (this space will be freed after installation)
        - ~40GB for program data
1. **Homebrew**
    - Install Homebrew by running:
        ```bash
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
		```
    - Follow any additional setup instructions provided by the installer

2. **Docker Desktop**
    - Install Docker Desktop for macOS from [docker.com](https://www.docker.com/products/docker-desktop)
    - Alternatively, install via Homebrew:
        ```bash
        brew install --cask docker
	  ```
	- (Recommended) Alternative to Docker you can use OrbStack:
		```bash
		brew install --cask orbstack
		```

3. **XQuartz**
    - Install via Homebrew:
        ```bash
        brew install --cask xquartz
        ```
    - After installation, restart your computer
    - Open XQuartz and enable "Allow connections from network clients" in XQuartz preferences
    - Navigate to XQuartz (next to Apple logo on top-left cornor) -> Settings -> Security -> Allow connections from network clients

4. **OpenFPGALoader**
    ```bash
    brew install openfpgaloader
    ```

6. **Vivado Installer**
    - Download Vivado installer for Linux from [AMD/Xilinx website](https://www.xilinx.com/support/download.html)

## Installation

1. **Get the Repository**
    ```bash
    git clone https://github.com/yokeTH/vivado-mac.git
    # or download and extract the ZIP file
    ```

2. **Run Setup Script**
    ```bash
    cd vivado-mac
    ./scripts/setup.sh
    ```

3. **Install Vivado**
    - When prompted, drag and drop the downloaded `Vivado installer` (from prerequisites no.4) into the terminal
    - Follow the installation instructions in the Vivado installer
    - Select desired Vivado components

## Usage
0. **Ensure Display Setup**
    - Check [X11 Display Issues](#x11-display-issues) if you encounter problems
    - XQuartz must be running before starting Vivado

1. Start Xilinx Virtual Cable (XVC)
Firstly, you have to plug the Basys3 in to your computer.
    ```bash
    openFPGALoader -b basys3 --xvc
    ```
3. Launch Vivado container:
Open another terminal,
    ```bash
    ./scripts/start_container.sh
    ```
4. Vivado GUI will appear in XQuartz window

## Troubleshooting

### Common Issues

1. **X11 Display Issues**
    - Ensure XQuartz is running
    - In XQuartz preferences:
      - Go to Security tab
      - Check "Allow connections from network clients"
    - Try restarting XQuartz
    - Run `xhost + localhost` before starting container
2. **For permission issues**, ensure setup script has executable permissions (`chmod +x scripts/setup.sh`)
3. **100 Killed Error**
    If you encounter the following error:
    ```
    100 Killed ${X_JAVA_HOME} /bin/java ${ARGS} -cp ${X_CLASS_PATH}    comxilinx.installerapi.InstallerLauncher "$@"
    ```
    try to increase Docker memory limit: Open Docker Dashboard > Click on settings > Resource > Advanced you will see the Memory limitation

## License

This project is licensed under the BSD 3-Clause License - see the LICENSE file for details.

## Vivado License

Vivado requires a license from AMD/Xilinx. Please obtain appropriate licensing from AMD/Xilinx website.

## OpenFPGALoader License

~~This repository contains the built binary of [OpenFPGALoader](https://github.com/trabucayre/openFPGALoader) that enable XVC feature for mac~~
Now official openfpgaloader is enable xilinx virtual cable.

## Disclaimer

This repository only provides the environment setup to run Vivado on Apple Silicon Macs via Docker. It does not include Vivado software itself. Users must:
- Download Vivado separately from AMD/Xilinx
- Comply with AMD/Xilinx's licensing terms
- Use at their own risk
