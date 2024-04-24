#!/bin/bash
export DATABASE_URL="sqlite:///home/michael/Development/Omnipedia Inc./wikidex/wikidex/sqlite_dummy.db"
export CUDA="/opt/cuda";
export CC="$CUDA/bin/gcc";
export CXX="$CUDA/bin/g++";
export RUST_LOG=info
export RUSTFLAGS="-C target-cpu=native"

cargo test --package wikidex --bin wikidex -- ingest::pipeline::processor::test::test --exact --show-output  --nocapture
