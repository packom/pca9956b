#!/bin/bash
set -e

function output_args {
    echo "Usage: build-container.sh <arch> <release|debug> [https] [cert_host] "
    echo "  <arch> = x86_64|arm|armv7"
    echo "  [cert_host] = host to install certs for"
    exit 1
}

ARCH=$1
if [[ ! $ARCH ]];
  then
    output_args
fi
if [[ $ARCH == "x86_64" ]];
  then
    TARGET="x86_64-unknown-linux-musl"
elif [[ $ARCH == "arm" ]]
  then
    TARGET="arm-unknown-linux-musleabihf"
elif [[ $ARCH == "armv7" ]]
  then
    TARGET="armv7-unknown-linux-musleabihf"
else
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

HTTPS=$3
if [[ $HTTPS == "https" ]];
  then
    HTTPS="https"
    CERT_HOST=$4
    if [[ ! $CERT_HOST ]];
      then
        output_args
    fi
elif [[ ! $HTTPS ]]
  then
    HTTPS=""
else
  echo "Unexpected arg $HTTPS"
  exit 1
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

DIR=tmp/$BIN-$TYPE-$ARCH-$VERSION
TAG=packom/$BIN-$TYPE-$ARCH:$VERSION

echo "Creating container for"
echo "  Binary:    $BIN"
echo "  Arch:      $ARCH"
echo "  Target:    $TARGET"
echo "  Type:      $TYPE"
echo "  Version:   $VERSION"
echo "  Tag:       $TAG"
echo "  Cert host: $CERT_HOST"


# need to run in build container
# echo "cargo build $BUILD_TYPE --target $TARGET"
# cargo build $BUILD_TYPE --target $TARGET
echo "docker run --rm -ti -v `pwd`:/home/build/builds piersfinlayson/build cargo build $BUILD_TYPE --target $TARGET"
docker run --rm -ti -v `pwd`:/home/build/builds piersfinlayson/build cargo build $BUILD_TYPE --target $TARGET

rm -fr $DIR
mkdir -p $DIR
echo "Getting binary: target/$TARGET/$TYPE/$BIN"
cp target/$TARGET/$TYPE/$BIN $DIR/

echo "wget -O $DIR/api.yaml https://raw.githubusercontent.com/packom/pca9956b-api/master/api/openapi.yaml"
wget -O $DIR/api.yaml https://raw.githubusercontent.com/packom/pca9956b-api/master/api/openapi.yaml

if [[ $HTTPS == "https" ]];
  then
    echo "Getting SSL files: ../certs/host/$CERT_HOST/..."
    cp ../certs/host/$CERT_HOST/*.key $DIR/key.pem
    cp ../certs/host/$CERT_HOST/*.crt $DIR/cert.pem
fi

echo "Generating Dockerfile"
echo "FROM scratch"
echo "FROM scratch" > $DIR/Dockerfile
echo "ADD $TMP/api.yaml /static/"
echo "ADD $TMP/api.yaml /static/" >> $DIR/Dockerfile
if [[ $HTTPS == "https" ]];
  then
    echo "ADD key.pem /ssl/"
    echo "ADD key.pem /ssl/" >> $DIR/Dockerfile
    echo "ADD cert.pem /ssl/"
    echo "ADD cert.pem /ssl/" >> $DIR/Dockerfile
fi
echo "ENV SERVER_PORT=8080"
echo "ENV SERVER_PORT=8080" >> $DIR/Dockerfile
echo "EXPOSE 8080"
echo "EXPOSE 8080" >> $DIR/Dockerfile
# Add binary last as most likely thing to change
echo "ADD $BIN /"
echo "ADD $BIN /" >> $DIR/Dockerfile
echo "CMD [\"/$BIN\"]"
echo "CMD [\"/$BIN\"]" >> $DIR/Dockerfile

echo "docker build -t $TAG $DIR"
docker build -t $TAG $DIR 
rm -fr tmp
