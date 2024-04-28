#!/bin/bash
export DATABASE_URL="sqlite:///home/michael/Development/Omnipedia Inc./wikidex/wikidex/sqlite_dummy.db"
export CUDA="/opt/cuda";
export CC="$CUDA/bin/gcc";
export CXX="$CUDA/bin/g++";
export RUST_LOG="info,async_openai=error"
export RUSTFLAGS="-C target-cpu=native"

cargo test --package wikidex --bin wikidex --no-default-features --features sqlite,server,ingest -- --exact --show-output  --nocapture
