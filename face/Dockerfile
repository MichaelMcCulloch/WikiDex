FROM nvidia/cuda:12.3.2-devel-ubuntu22.04 as builder



ARG TORCH_CUDA_ARCH_LIST="${TORCH_CUDA_ARCH_LIST}"

RUN \
    --mount=type=cache,target=/var/cache/apt,sharing=locked,rw apt-get update && \
    apt-get install -y curl wget gcc g++ ca-certificates cmake software-properties-common pkg-config libssl-dev liblapack-dev libblas-dev && \
    rm -rf /var/lib/apt/lists/*

RUN \
    --mount=type=cache,target=/var/cache/apt,sharing=locked,rw apt-get update && \
    apt remove -y --purge --auto-remove cmake

RUN \
    wget -O - https://apt.kitware.com/keys/kitware-archive-latest.asc 2>/dev/null | gpg --dearmor - | tee /etc/apt/trusted.gpg.d/kitware.gpg >/dev/null && \
    add-apt-repository "deb https://apt.kitware.com/ubuntu/ $(lsb_release -cs) main" && \
    apt update && \
    apt install -y kitware-archive-keyring && \
    rm /etc/apt/trusted.gpg.d/kitware.gpg && \
    apt update && \
    apt install -y cmake 

RUN \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup update

WORKDIR /usr/src/face
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src
ENV CUDA_PATH=/usr/local/cuda
RUN  \
    --mount=type=cache,target=/usr/src/face/target,sharing=locked,rw cargo install --path . --root ./build 

FROM nvidia/cuda:12.3.2-runtime-ubuntu22.04
ARG TORCH_CUDA_ARCH_LIST="${TORCH_CUDA_ARCH_LIST}"
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked,rw apt-get update  && \
    apt-get install -y ca-certificates pkg-config libssl-dev liblapack-dev libblas-dev libgomp1 && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/face/build/bin/face /usr/local/bin/face

