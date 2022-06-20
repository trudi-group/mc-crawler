#!/bin/bash
# This script compiles the binary in docker and then copies the binary to the local FS
mkdir -p out

docker build -t mc-crawler-builder -f Dockerfile.builder .
docker create --name extract mc-crawler-builder
docker cp extract:/mc-crawler/target/release/mc-crawler ./out

docker rm extract
