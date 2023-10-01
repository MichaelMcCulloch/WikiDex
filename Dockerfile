FROM nvidia/cuda:12.2.0-devel-ubuntu22.04 as builder
ARG TORCH_CUDA_ARCH_LIST="${TORCH_CUDA_ARCH_LIST:-8.6}"
# RUN --mount=type=cache,target=/root/.cache/pip,rw \
#     rm -rf /root/.cache/pip/*

RUN --mount=type=cache,target=/var/cache/apt,sharing=locked,rw apt-get update && \
    apt-get install --no-install-recommends -y git vim build-essential python3-dev python3-venv ninja-build && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /build
ARG TORCH_CUDA_ARCH_LIST="${TORCH_CUDA_ARCH_LIST:-8.6}"
RUN --mount=type=cache,target=/root/.cache/pip,rw \
    python3 -m venv /build/venv && \
    . /build/venv/bin/activate && \
    pip3 install --upgrade pip wheel setuptools packaging && \
    pip3 install torch==2.2.0.dev20230929+cu121 --extra-index-url https://download.pytorch.org/whl/nightly/cu121 && \
    MAX_JOBS=6  pip3 install  torch==2.2.0.dev20230929+cu121  --extra-index-url https://download.pytorch.org/whl/nightly/cu121 flash-attn --no-build-isolation && \
    pip3 install flask flask_cors exllamav2



WORKDIR /app
copy ./llama.py /app/llama.py
# CMD ls -l /usr/local/cuda
EXPOSE 5000:5000
cmd . /build/venv/bin/activate && export CUDA_HOME=/usr/local/cuda && python3 ./llama.py -m /app/models/llm/TheBloke/Speechless-Llama2-Hermes-Orca-Platypus-WizardLM-13B-GPTQ/gptq-4bit-32g-actorder_True/ -mode llama