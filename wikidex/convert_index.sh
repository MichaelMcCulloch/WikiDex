#!/bin/bash
export DATABASE_URL="sqlite://sqlite_dummy.db"
export RUST_LOG="info,async_openai=error"
export RUSTFLAGS="-C target-cpu=native"

cargo test --package wikidex --bin wikidex -- ingest::pipeline::index_converter::test::test --exact --show-output  --nocapture
