FROM nvidia/cuda:12.3.2-devel-ubuntu22.04 as builder

ARG TORCH_CUDA_ARCH_LIST="${TORCH_CUDA_ARCH_LIST}"
RUN \
    --mount=type=cache,target=/var/cache/apt,sharing=locked,rw apt-get update  && \
    apt-get install -y ca-certificates cmake pkg-config libssl-dev liblapack-dev libblas-dev && \
    rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src/face-machine
COPY ./Cargo.toml ./Cargo.toml
COPY ./sqlite_dummy.db ./sqlite_dummy.db
COPY ./src ./src
RUN  \
    --mount=type=cache,target=/usr/src/face-machine/target,sharing=locked,rw cargo install --path . --root ./build 

FROM nvidia/cuda:12.3.2-runtime-ubuntu22.04
ARG TORCH_CUDA_ARCH_LIST="${TORCH_CUDA_ARCH_LIST}"
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked,rw apt-get update  && \
    apt-get install -y ca-certificates pkg-config libssl-dev liblapack-dev libblas-dev libgomp1 && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/face-machine/build/bin/face-machine /usr/local/bin/face-machine

COPY ./sqlite_dummy.db ./sqlite_dummy.db

CMD face-machine 