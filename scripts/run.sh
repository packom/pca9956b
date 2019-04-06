Currently broken

#!/bin/bash
set -e

function output_args {
    echo "Usage: run.sh <arch> <release|debug> [port]"
    echo "  <arch> = x86_64|arm|armv7"
    exit 1
}

ARCH=$1
if [[ ! $ARCH ]];
  then
    output_args
fi

TYPE=$2
if [[ ! $TYPE ]];
  then
    output_args
fi
if [[ $TYPE == "release" ]];
  then
    TYPE="release"
    BUILD_TYPE="--release"
elif [[ $TYPE == "debug" ]]
  then
    TYPE="debug"
    BUILD_TYPE=""
else
  output_args
fi

VERSION="$(awk '/^version = /{print $3}' Cargo.toml | sed 's/"//g' | sed 's/\r$//')"
if [[ ! $VERSION ]];
  then
    echo "Couldn't get version from Cargo.toml"
    exit 1
fi

BIN="$(awk '/^name = /{print $3}' Cargo.toml | sed 's/"//g' | sed 's/\r$//')"
if [[ ! $BIN ]];
  then
    echo "Couldn't get binary from Cargo.toml"
    exit 1
fi

TAG=$BIN-$TYPE-$ARCH:$VERSION
NAME=$BIN-$TYPE-$ARCH

echo "Running container"
echo "  Binary:  $BIN"
echo "  Arch:    $ARCH"
echo "  Type:    $TYPE"
echo "  Version: $VERSION"
echo "  Tag:     $TAG"
echo "  Name:    $NAME"

docker run --rm --name $NAME -d -e RUST_LOG=info -e I2CBUS_IP=pi-esp32 -p 8081:8080 pca9956b-debug-x86_64:0.1.1

echo "docker run --name $NAME -d -e GFA_I2CBUS_IP=pi-esp32 -e GFA_I2CBUS_PORT=8080 -p $PORT:8080 $TAG"
docker run --name $NAME -d -e GFA_I2CBUS_IP=pi-esp32 -e GFA_I2CBUS_PORT=8080 -p $PORT:8080 $TAG