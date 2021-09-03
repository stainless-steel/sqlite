#!/usr/bin/env bash
set -o errexit -o nounset -o pipefail

# set current working directory to script directory to run script from everywhere
cd "$(dirname "$0")"

# build test.wasm
marine build --release --bin test

# copy .wasm to artifacts
rm -f artifacts/*
mkdir -p artifacts
cp target/wasm32-wasi/release/test.wasm artifacts/

# download SQLite 3 to use in tests
curl -L https://github.com/fluencelabs/sqlite/releases/download/v0.15.0_w/sqlite3.wasm -o artifacts/sqlite3.wasm
