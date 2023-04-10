#!/bin/bash

set -x

cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --out-dir out --target web target/wasm32-unknown-unknown/release/bevy_jam_3.wasm
rm -rf out/assets
cp -R assets out/assets
xdg-open "http://0.0.0.0:8080" &
trap "fuser -k 8080/tcp" SIGINT SIGTERM EXIT
miniserve --index index.html out
