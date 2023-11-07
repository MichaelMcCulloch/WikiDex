FROM rocm/dev-ubuntu-22.04:5.7-complete

RUN --mount=type=cache,target=/var/cache/apt,sharing=locked,rw apt-get update && \
    apt-get install --no-install-recommends -y git vim build-essential python3-dev python3-venv ninja-build && \
    rm -rf /var/lib/apt/lists/*

COPY . .

ARG HSA_OVERRIDE_GFX_VERSION="${HSA_OVERRIDE_GFX_VERSION}"
RUN --mount=type=cache,target=/root/.cache/pip,rw \
    python3 -m venv /venv && \
    . /venv/bin/activate && \ 
    pip3 install --upgrade pip wheel setuptools packaging && \
    pip3 install --pre torch --index-url https://download.pytorch.org/whl/nightly/rocm5.7 && \
    pip3 install -r requirements.txt

CMD . /venv/bin/activate && python3 embeddings_service.py \
    --port "${EMBED_CONT_PORT}" \
    --model "${SBERT_MODEL_NAME}" \
    --batch "${SBERT_BATCH_SIZE}"