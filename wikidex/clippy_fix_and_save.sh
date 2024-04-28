#!/usr/bin/bash
export DATABASE_URL="sqlite:///home/michael/Development/Omnipedia Inc./wikidex/wikidex/sqlite_dummy.db"
export CUDA="/opt/cuda";
export CC="$CUDA/bin/gcc";
export CXX="$CUDA/bin/g++";
export RUST_LOG=info
export RUSTFLAGS="-C target-cpu=native"
cargo clippy --fix --workspace --message-format=json --all-targets --allow-dirty && git checkout local-step && git add . && git commit -m "Clippy fix"
