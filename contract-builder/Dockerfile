FROM rust:1.67.1

LABEL description="Container for builds"

RUN rustup default 1.67.1
RUN rustup target add wasm32-unknown-unknown
RUN rustup component add rustfmt

RUN apt-get -y update && apt-get install -y \
    git \
    less \
    vim \
    llvm \
    clang \
    && rm -rf /var/lib/apt/lists/*
