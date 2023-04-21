#!/bin/sh

./build.sh

echo ">> Deploying contract"

npx near dev-deploy --wasmFile ./target/wasm32-unknown-unknown/release/contract.wasm
