#!/bin/bash

script_dir=$(dirname -- "$(readlink -nf $0)";)
source "$script_dir/headers.sh"

function stop_container {
    docker kill vivado > /dev/null 2>&1
    echo "Stopped Docker container"
    exit 0
}
trap 'stop_container' INT

export export DISPLAY=:0
/opt/X11/bin/xhost + localhost

docker run --init --rm --network=host \
    -e DISPLAY=host.docker.internal:0 \
    --name vivado \
    --mount type=bind,source="$script_dir/../",target="/home/user" \
    --platform linux/amd64 $IMAGE_NAME \
    sudo -H -u user bash scripts/startup.sh &

# monitor vivado container
sleep 10
while [[ $(docker ps) == *vivado* ]]
do
    sleep 1
done
stop_container
