#!/bin/bash

# Find directory: checks New Pattern (* / Vivado) and Old Pattern (Vivado / *)
V_DIR=$(ls -d /home/user/Xilinx/*/Vivado /home/user/Xilinx/Vivado/* 2>/dev/null | head -n 1)

if [ -d "$V_DIR" ]
then
	$V_DIR/bin/hw_server -e "set auto-open-servers xilinx-xvc:host.docker.internal:3721" &
	. $V_DIR/settings64.sh
	$V_DIR/bin/vivado -nolog -nojournal
else
	echo "The installation is incomplete."
fi
