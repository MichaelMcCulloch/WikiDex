#!/bin/bash
export DATABASE_URL="sqlite:///home/michael/Development/Omnipedia Inc./wikidex/wikidex/sqlite_dummy.db"
export CUDA="/opt/cuda";
export CC="$CUDA/bin/gcc";
export CXX="$CUDA/bin/g++";
export RUST_LOG=info
export RUSTFLAGS="-C target-cpu=native"

# cargo test --package wikidex --bin wikidex -- ingest::pipeline::processor::test::test --exact --show-output  --nocapture
cargo run  -- \
wikipedia \
--wiki-xml \
/home/michael/Documents/WIKIDUMPS/20240420/enwiki-20240420-pages-articles.xml \
--output-directory \
/home/michael/Desktop/wikisql/wikipedia_docstore.sqlite \
--ingest-limit \
"1000" \
--embed-name \
"thenlper/gte-small" \
--embed-url \
"http://192.168.1.120:9000/v1" \
--embed-endpoint \
openai \
--llm-name \
"TheBloke/Mistral-7B-Instruct-v0.2-AWQ" \
--llm-url \
"http://triton:8001" \
--llm-endpoint \
triton \
--llm-kind \
instruct \
--nebula-url \
"http://graphd:9669" \
--nebula-user \
"root" \
--nebula-pass \
"nebula"