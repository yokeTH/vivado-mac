#!/bin/bash

script_dir=$(dirname -- "$(readlink -nf $0)")

# Color codes
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly PURPLE='\033[0;35m'
readonly CYAN='\033[0;36m'
readonly GREY='\033[0;90m'
readonly NC='\033[0m' # No Color

# Docker
readonly IMAGE_NAME="ghcr.io/yoketh/vivado-mac"
readonly INSTALLATION_BIN_LOG_PATH="${script_dir}/installation_location.txt"

# Function to print error message in red
error() {
    echo -e "${RED}[ERROR] $*${NC}" >&2
}

# Function to print success message in green
success() {
    echo -e "${GREEN}[SUCCESS] $*${NC}"
}

# Function to print warning message in yellow
warning() {
    echo -e "${YELLOW}[WARNING] $*${NC}"
}

# Function to print info message in blue
info() {
    echo -e "${BLUE}[INFO] $*${NC}"
}

# Function to print debug message in grey
debug() {
    echo -e "${GREY}[DEBUG] $*${NC}"
}

# Function to print step message in cyan
step() {
    echo -e "${CYAN}[STEP] $*${NC}"
}

# Function to print important message in purple
important() {
    echo -e "${PURPLE}[IMPORTANT] $*${NC}"
}


# Drag and drop file function with path logging
# Usage: drag_and_drop_files
drag_and_drop_files() {
    local prompt="${1:-Drag and Drop Files}"
    local copy_mode="${2:-copy}"  # Default to copy, can be set to 'move'
    local log_file="${3:-drag_drop_files.txt}"  # Default log file name

    # Ensure log file can be created/written
    touch "$log_file" 2>/dev/null
    if [[ ! -w "$log_file" ]]; then
        echo "Error: Cannot write to log file $log_file"
        return 1
    fi

    step "$prompt"
    info "----------------------"
    info "Instructions:"
    info "1. Drag and drop file(s) into this terminal window."
    info "2. Press Enter after dropping files."
    info "3. Type 'exit' to quit."

    while true; do
        # Prompt and read input
        echo -ne "\n> "
        read -e input

        # Exit condition
        if [[ "$input" == "exit" ]]; then
            success "Exiting drag-and-drop process."
            break
        fi

        # Check if input is empty
        if [[ -z "$input" ]]; then
            error "No files dropped. Try again."
            continue
        fi

        # Process each file (only once)
        IFS=$'\n'  # Handle filenames with spaces
        for file in $input; do
            # Verify file exists
            if [[ -f "$file" ]]; then
                # Get absolute path
                local abs_path=$(readlink -f "$file")
                local filename=$(basename "$file")
                local dest_path="$script_dir/../$filename"

                # Perform copy or move based on mode
                if [[ "$copy_mode" == "copy" ]]; then
                    if cp "$abs_path" "$dest_path"; then
                        success "Copied: $filename (from $abs_path)"
                    else
                        error "Failed to copy: $filename"
                    fi
                else
                    if mv "$abs_path" "$dest_path"; then
                        success "Moved: $filename (from $abs_path)"
                    else
                        error "Failed to move: $filename"
                    fi
                fi

                # Log the new file path to the log file
                echo "../$filename" >> "$log_file"
                info "Logged path: $dest_path to $log_file"
                break 2 # Stop after processing the first file
            else
                error "Error: $file is not a valid file"
            fi
        done
        unset IFS  # Reset IFS to default
    done
}
