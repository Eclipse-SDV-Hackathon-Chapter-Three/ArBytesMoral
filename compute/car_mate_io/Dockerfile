FROM ghcr.io/astral-sh/uv:python3.12-bookworm as base

WORKDIR /app

RUN apt-get update && apt-get install --no-install-recommends -y espeak-ng \
        && apt-get clean && rm -rf /var/lib/apt/lists/*

RUN ln -sf /usr/bin/espeak-ng /usr/bin/espeak

COPY . /app

RUN uv venv \
    && echo "ENV PATH=/app/.venv/bin:$PATH" >> /uv_vars.sh \
    && uv sync

ENV PATH="/app/.venv/bin:$PATH"

RUN    git clone --recurse-submodules https://github.com/eclipse-uprotocol/up-python.git \
        && cd up-python \
        && git submodule update --init --recursive \
        && uv add setuptools  \
        && uv pip install setuptools grpcio-tools \
        && uv pip install . 

RUN    git clone https://github.com/eclipse-uprotocol/up-transport-zenoh-python.git \
        && cd up-transport-zenoh-python/ \
        && uv pip install .

CMD [ "python", "-u", "./car_mate_io.py" ]