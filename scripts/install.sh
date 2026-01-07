#!/bin/bash

script_dir=$(dirname -- "$(readlink -nf $0)";)
source "$script_dir/headers.sh"

declare -A VERSIONS=(
    ["202502"]="abe838aa2e2d3d9b10fea94165e9a303"
    ["202402"]="20c806793b3ea8d79273d5138fbd195f"
    ["202401"]="8b0e99a41b851b50592d5d6ef1b1263d"
    ["202302"]="b8c785d03b754766538d6cde1277c4f0"
)

get_version_from_hash() {
    local hash="$1"

    for version in "${!VERSIONS[@]}"; do
        if [ "${VERSIONS[$version]}" == "$hash" ]; then
            echo "$version"
            return 0
        fi
    done

    echo ""
    return 1
}

get_credentials() {
    local secret_file="$1"
    local secret_dir=$(dirname "$secret_file")

    # Create directory if it doesn't exist
    mkdir -p "$secret_dir"

    # Prompt for credentials
    echo -n "Enter your email address: "
    read email
    echo -n "Enter your password: "
    read -s password
    echo  # New line after password input

    # Save credentials to file
    echo "$email" > "$secret_file"
    echo "$password" >> "$secret_file"

    echo "Credentials saved to $secret_file"
}

SECRET_FILE="$script_dir/secret.txt"

INSTALLATION_FILE_PATH=$(cat "$INSTALLATION_BIN_LOG_PATH" | xargs)

INSTALLER_HASH=($(md5sum "$script_dir/$INSTALLATION_FILE_PATH"))
VERSION=$(get_version_from_hash "$INSTALLER_HASH")

if [ $VERSION == "" ]; then
    error The installer $INSTALLATION_FILE_PATH hash not match. please make sure you download linux installer and support version.
    exit 1
fi

if [ $VERSION == "202401" ]; then
    error version $VERSION is not support please use latest version of year.
    exit 1
fi

info The installer is version $VERSION

step "try to find $INSTALLATION_FILE_PATH"

if [ -f "$script_dir/$INSTALLATION_FILE_PATH" ]; then
    success "File exists: $INSTALLATION_FILE_PATH"
else
    error "File does not exist: $INSTALLATION_FILE_PATH"
    error "cleaning up cache files please run this script again"
    rm $script_dir/installation_location.txt
    exit
fi

# cat $SECRET_FILE
if ! [ -d "$script_dir/../installer" ]; then
    step "start extract installer"
    chmod u+x "$script_dir/$INSTALLATION_FILE_PATH"
    eval "\"$script_dir/$INSTALLATION_FILE_PATH\" --target \"$script_dir/../installer\" --noexec"
else
    debug "The installer already extracted"
fi

step "Generate AuthTokenGen"

GENERATED_TOKEN=false

if [ -f "$SECRET_FILE" ]; then
        info "Credentials file found."
        if ! expect -f $HOME/scripts/auth_token_gen.exp /home/user/installer/xsetup "$SECRET_FILE"; then
            error secret.txt corrupt. removing $SECRET_FILE
            rm $SECRET_FILE
        else
            GENERATED_TOKEN=true
        fi
fi

if ! $GENERATED_TOKEN && ! /home/user/installer/xsetup -b AuthTokenGen
then
    warning "Can't Generate AuthTokenGen"
    step "now using expect method"
    step "Checking for credentials..."
    if ! [ -f "$SECRET_FILE" ]; then
        warning "Credentials file not found."
        get_credentials "$SECRET_FILE"
    fi

    # Check if secret.txt is readable and not empty
    if ! [ -r "$SECRET_FILE" ] || ! [ -s "$SECRET_FILE" ]; then
        warning "Error: Cannot read credentials file or file is empty"
        get_credentials "$SECRET_FILE"
    fi

    step "Generate AuthTokenGen"

    expect -f $HOME/scripts/auth_token_gen.exp /home/user/installer/xsetup "$SECRET_FILE"
else
    GENERATED_TOKEN=true
fi

if $GENERATED_TOKEN; then
    step "Start Download and Installing"
    /home/user/installer/xsetup -c "/home/user/scripts/vivado_settings_$VERSION.txt" -b Install -a XilinxEULA,3rdPartyEULA
fi
