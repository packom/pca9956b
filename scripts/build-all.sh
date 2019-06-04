#!/bin/bash
set -e

docker login -u packom
scripts/build-container.sh arm release
scripts/build-container.sh armv7 release
scripts/build-container.sh x86_64 release
docker push packom/pca9956b-release-arm:0.1.1
docker push packom/pca9956b-release-armv7:0.1.1
docker push packom/pca9956b-release-x86_64:0.1.1
