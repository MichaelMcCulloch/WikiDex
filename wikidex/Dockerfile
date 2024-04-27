FROM rust:1.77.2-bookworm as builder

RUN \
    --mount=type=cache,target=/var/cache/apt,sharing=locked,rw apt-get update  && \
    apt-get install -y ca-certificates cmake pkg-config libssl-dev liblapack-dev libblas-dev protobuf-compiler && \
    rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src/wikidex
COPY ./sqlite_dummy.db ./sqlite_dummy.db
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src

ARG DATABASE_URL="postgres://postgres:postgres@192.168.1.120:5433/postgres"

RUN  \
    --mount=type=cache,target=/usr/src/wikidex/target,sharing=locked,rw cargo install --no-default-features --features postgres,server --path . --root ./build 

FROM ubuntu:22.04
COPY ./sqlite_dummy.db ./sqlite_dummy.db
ARG TORCH_CUDA_ARCH_LIST="${TORCH_CUDA_ARCH_LIST}"
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked,rw apt-get update  && \
    apt-get install -y ca-certificates pkg-config libssl-dev liblapack-dev libblas-dev libgomp1 && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/wikidex/build/bin/wikidex /usr/local/bin/wikidex

