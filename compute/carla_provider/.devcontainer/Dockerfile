#FROM docker.io/rust:bullseye
FROM docker.io/catthehacker/ubuntu:rust-20.04

RUN apt-get update && apt-get install --no-install-recommends -y cmake \
        clang-12 libclang-12-dev\
        && apt-get clean && rm -rf /var/lib/apt/lists/*