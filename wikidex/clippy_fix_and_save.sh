#!/usr/bin/bash
export DATABASE_URL="sqlite:///home/michael/Development/Omnipedia Inc./wikidex/wikidex/sqlite_dummy.db"
export CUDA="/opt/cuda";
export CC="$CUDA/bin/gcc";
export CXX="$CUDA/bin/g++";
export RUST_LOG=info
export RUSTFLAGS="-C target-cpu=native"
cargo clippy --fix --workspace --message-format=json --all-targets --allow-dirty &
(
    x=`(cargo clippy --fix --workspace --message-format=json --all-targets --allow-dirty)`
    success=$(echo "$x" | jq -r '.success' | grep -v "null" | tail -n 1)
    # Check if the "success" field is true
    if [ "$success" = "true" ]; then
        git add .
        git commit -m "Clippy fix"
    fi
)&
wait