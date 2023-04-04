#!/bin/bash
cargo fmt --all
RUSTFLAGS='-C link-arg=-s' cargo build --all --target wasm32-unknown-unknown --release

mkdir -p "res"

cp target/wasm32-unknown-unknown/release/*.wasm ./res/
