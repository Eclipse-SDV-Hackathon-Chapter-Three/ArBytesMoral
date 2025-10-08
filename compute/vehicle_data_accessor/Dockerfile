# SPDX-License-Identifier: Apache-2.0
FROM docker.io/rust:latest

RUN apt-get update && apt-get install --no-install-recommends -y cmake \
        && apt-get clean && rm -rf /var/lib/apt/lists/*

COPY ./ ${WORKSPACE_DIR}
WORKDIR ${WORKSPACE_DIR}
RUN --mount=type=cache,target=${WORKSPACE_DIR}/target/release cargo build --release \
    && cp ${WORKSPACE_DIR}/target/release/app /usr/local/bin/

ENV RUST_BACKTRACE=full
ENV RUST_LOG=trace

ENTRYPOINT ["/usr/local/bin/app"]