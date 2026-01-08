#!/bin/bash

script_dir=$(dirname -- "$(readlink -nf $0)";)
source "$script_dir/headers.sh"

if [ -d "$script_dir/../Xilinx" ]
then
	error "A previous installation was found. To reinstall, remove the Xilinx folder."
	exit 1
fi

if ! [ -f "$INSTALLATION_BIN_LOG_PATH" ]; then
    drag_and_drop_files "please drags and drop your vivado installer.bin to this terminal" "copy" $INSTALLATION_BIN_LOG_PATH
else
    info "$INSTALLATION_BIN_LOG_PATH found."
fi

if ! [[ $(docker image ls ) == *$IMAGE_NAME* ]]
then
    step "Build The Image"
    if ! docker pull --platform=linux/amd64 $IMAGE_NAME
    then
        error "Docker image pull failed!"
        exit 1
    fi
    success "The Docker image was successfully generated."
else
    debug "The Image already exits"
fi

step "Start container for setup Vivado"
docker run --init --rm -it --name vivado_x11 --mount type=bind,source="$script_dir/..",target="/home/user" --platform linux/amd64 $IMAGE_NAME bash scripts/install.sh
