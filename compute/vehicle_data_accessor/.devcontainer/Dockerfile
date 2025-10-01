FROM docker.io/rust:latest

RUN apt-get update && apt-get install --no-install-recommends -y cmake \
        && apt-get clean && rm -rf /var/lib/apt/lists/*