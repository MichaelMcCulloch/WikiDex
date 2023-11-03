FROM rust:1.73.0-bookworm as builder

RUN --mount=type=cache,target=/var/cache/apt,sharing=locked,rw apt-get update  && \
    apt-get install -y cmake pkg-config libssl-dev liblapack-dev libblas-dev && \
    rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src/omnipedia
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src
RUN cargo install --path . --root ./build

FROM nvidia/cuda:12.2.2-runtime-ubuntu22.04
ARG TORCH_CUDA_ARCH_LIST="${TORCH_CUDA_ARCH_LIST:-8.6}"
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked,rw apt-get update  && \
    apt-get install -y pkg-config libssl-dev liblapack-dev libblas-dev libgomp1 && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/omnipedia/build/bin/omnipedia /usr/local/bin/omnipedia
COPY ./Config.TOML /Config.TOML
CMD ["omnipedia"]