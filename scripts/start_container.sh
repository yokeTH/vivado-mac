script_dir=$(dirname -- "$(readlink -nf $0)";)

function stop_container {
    docker kill vivado > /dev/null 2>&1
    echo "Stopped Docker container"
    kill -9 $OPENFPGALOAD_PID
    exit 0
}
trap 'stop_container' INT

xhost + localhost
docker run --init --rm --network=host \
    -e DISPLAY=host.docker.internal:0 \
    --name vivado \
    --mount type=bind,source="$script_dir/../",target="/home/user" \
    --platform linux/amd64 ubuntu_vivado_env \
    sudo -H -u user bash scripts/startup.sh &

./$script_dir/openFPGALoader --verbose-level -1 -b basys3 --xvc & OPENFPGALOAD_PID=$!

# monitor vivado container
sleep 10
while [[ $(docker ps) == *vivado* ]]
do
    sleep 1
done
stop_container