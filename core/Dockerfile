FROM rust:1.75.0-bookworm as builder

RUN \
    --mount=type=cache,target=/var/cache/apt,sharing=locked,rw apt-get update  && \
    apt-get install -y ca-certificates cmake pkg-config libssl-dev liblapack-dev libblas-dev && \
    rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src/omnipedia
COPY ./Cargo.toml ./Cargo.toml
COPY ./sqlite_dummy.db ./sqlite_dummy.db
COPY ./src ./src
ARG DATABASE_URL="sqlite://sqlite_dummy.db"
RUN  \
    --mount=type=cache,target=/usr/src/omnipedia/target,sharing=locked,rw cargo install --path . --root ./build 

FROM ubuntu:22.04
ARG TORCH_CUDA_ARCH_LIST="${TORCH_CUDA_ARCH_LIST}"
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked,rw apt-get update  && \
    apt-get install -y ca-certificates pkg-config libssl-dev liblapack-dev libblas-dev libgomp1 && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/omnipedia/build/bin/omnipedia /usr/local/bin/omnipedia

COPY ./sqlite_dummy.db ./sqlite_dummy.db

CMD omnipedia \
    server \
    --docstore "${DOCSTORE_PATH}" \
    --llm-url "https://vllm.semanticallyinvalid.net/v1/" \
    --language-model-name "${LLM_MODEL_NAME}" \
    --language-model-kind "${MODEL_KIND}" \
    --embed-url "https://infinity.semanticallyinvalid.net/v1" \
    --embed-model-name "${SBERT_MODEL_NAME}" \
    --index-url "https://face.semanticallyinvalid.net/"\
    --host 0.0.0.0 \
    --port "${OMNIPEDIA_CONT_PORT}" \
    --system-prompt-path "${SYSTEM_PROMPT_PATH}" \
    --api-key "$API_SECRET_KEY"