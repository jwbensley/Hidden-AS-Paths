#!/bin/bash

set -e


function print_help {
    echo ""
    echo "You must choose exactly one of the following options"
    echo ""
    echo "-c    Copy files to remove server"
    echo "-i    Install rust and dependencies on remote server"
    echo ""
    echo "$0 -i 10.0.0.1 user123"
    echo "$0 -c 10.0.0.1 user123"
    echo ""
}

if [[ $# -ne 3 ]]
then
    print_help
    exit 1
fi

set -eu

SERVER="$2"
USER="$3"
BASE="~/as_paths/"

if [ "$1" == "-c" ]
then
    echo "Copying files to server..."
    ssh ${USER}@${SERVER} "mkdir -p ${BASE} && mkdir -p ${BASE}/src"
    scp *.lock *.toml *.md ${USER}@${SERVER}:${BASE}
    scp src/* ${USER}@${SERVER}:${BASE}/src/
    echo "Done"
elif [ "$1" == "-i" ]
then
    echo "Installing dependencies on server..."
    ssh ${USER}@${SERVER} "
    sudo apt update

    sudo apt-get install --no-install-recommends -y \
    curl \
    build-essential \
    pkg-config

    curl -o rustup.sh https://sh.rustup.rs
    chmod a+x rustup.sh
    ./rustup.sh -y
    
    .cargo/bin/cargo -V
    "
    echo "Done"
fi
