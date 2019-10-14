#!/bin/bash
version=$(cat Cargo.toml| grep "^version" | awk -v x=3 '{print $x}' | sed -e 's/^"//' -e 's/"$//')
docker build -t tsatiz/myscontroller-rs:${version} .
echo "$DOCKER_PASSWORD" | docker login -u "$DOCKER_USERNAME" --password-stdin
docker push tsatiz/myscontroller-rs:${version}