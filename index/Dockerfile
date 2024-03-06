FROM nvidia/cuda:12.3.2-devel-ubuntu22.04 as builder

ARG TORCH_CUDA_ARCH_LIST="${TORCH_CUDA_ARCH_LIST}"
RUN \
    --mount=type=cache,target=/var/cache/apt,sharing=locked,rw apt-get update  && \
    apt-get install -y ca-certificates curl cmake pkg-config libssl-dev liblapack-dev libblas-dev && \
    rm -rf /var/lib/apt/lists/*
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
WORKDIR /usr/src/index
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src
RUN  \
    --mount=type=cache,target=/usr/src/index/target,sharing=locked,rw $HOME/.cargo/bin/cargo install --path . --root ./build 

FROM nvidia/cuda:12.3.2-runtime-ubuntu22.04
ARG TORCH_CUDA_ARCH_LIST="${TORCH_CUDA_ARCH_LIST}"
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked,rw apt-get update  && \
    apt-get install -y ca-certificates pkg-config libssl-dev liblapack-dev libblas-dev libgomp1 && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/index/build/bin/index /usr/local/bin/index


CMD index 