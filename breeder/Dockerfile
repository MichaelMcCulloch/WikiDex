FROM rust:1.75.0-bookworm as builder

RUN \
    --mount=type=cache,target=/var/cache/apt,sharing=locked,rw apt-get update  && \
    apt-get install -y ca-certificates cmake pkg-config libssl-dev liblapack-dev libblas-dev && \
    rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src/wikidex
COPY ./Cargo.toml ./Cargo.toml
COPY ./sqlite_dummy.db ./sqlite_dummy.db
COPY ./src ./src
ARG DATABASE_URL="sqlite://sqlite_dummy.db"
RUN  \
    --mount=type=cache,target=/usr/src/wikidex/target,sharing=locked,rw cargo install --no-default-features --features server --path . --root ./build 

FROM ubuntu:22.04
ARG TORCH_CUDA_ARCH_LIST="${TORCH_CUDA_ARCH_LIST}"
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked,rw apt-get update  && \
    apt-get install -y ca-certificates pkg-config libssl-dev liblapack-dev libblas-dev libgomp1 && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/wikidex/build/bin/wikidex /usr/local/bin/wikidex

COPY ./sqlite_dummy.db ./sqlite_dummy.db

CMD wikidex \
    server \
    --docstore "${DOCSTORE_URL}" \
    --llm-url "${VLLM_URL}" \
    --language-model-name "${LLM_MODEL_NAME}" \
    --language-model-kind "${MODEL_KIND}" \
    --embed-url "${EMBED_URL}" \
    --embed-model-name "${SBERT_MODEL_NAME}" \
    --index-url "${INDEX_URL}"\
    --host 0.0.0.0 \
    --port "${WIKIDEX_CONT_PORT}" \
    --system-prompt-path "${SYSTEM_PROMPT_PATH}" \
    --api-key "$API_SECRET_KEY"