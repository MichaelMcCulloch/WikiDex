#!/bin/bash
export DATABASE_URL="sqlite:///home/michael/Development/Omnipedia Inc./wikidex/wikidex/sqlite_dummy.db"
export CUDA="/opt/cuda";
export CC="$CUDA/bin/gcc";
export CXX="$CUDA/bin/g++";
export RUST_LOG="info,async_openai=error"
export RUSTFLAGS="-C target-cpu=native"

cargo run --release -- \
wikipedia \
--wiki-xml \
"/home/michael/Development/Scratch Space/wikisql/enwiki-20240420-pages-articles.xml" \
--output-directory \
"/home/michael/Development/Scratch Space/wikisql/" \
--ingest-limit \
"0" \
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