FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive

# Toolchain & essentials (+ openssh-client and xz-utils for rustup)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      clang-12 libclang-12-dev llvm-12 llvm-12-dev \
      build-essential cmake git pkg-config \
      libssl-dev \
      curl ca-certificates \
      openssh-client xz-utils && \
    rm -rf /var/lib/apt/lists/*

# Non-interactive host key acceptance for first-time git fetches
ENV GIT_SSH_COMMAND="ssh -o StrictHostKeyChecking=accept-new"

# Pre-populate known_hosts for common git forges (optional, speeds up first fetch)
RUN mkdir -p /etc/ssh && \
    ssh-keyscan -H github.com gitlab.com ssh.dev.azure.com 2>/dev/null >> /etc/ssh/ssh_known_hosts

# Default to Clang 12
ENV CC=clang-12 \
    CXX=clang++-12

# Add both common cargo bin paths to PATH (works for root or $USERNAME)
ENV LLVM_CONFIG_PATH=/usr/bin/llvm-config-12 \
    LIBCLANG_PATH=/usr/lib/llvm-12/lib \
    LIBCLANG_STATIC_PATH=/usr/lib/llvm-12/lib \
    CLANG_PATH=/usr/bin/clang-12
    
# Install rustup (full default profile) and extras
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y 
COPY ./ ${WORKSPACE_DIR}
WORKDIR ${WORKSPACE_DIR}
RUN --mount=type=cache,target=${WORKSPACE_DIR}/target/release cargo build --release \
    && cp ${WORKSPACE_DIR}/target/release/app /usr/local/bin/

ENV RUST_BACKTRACE=full
ENV RUST_LOG=trace

ENTRYPOINT ["/usr/local/bin/app"]