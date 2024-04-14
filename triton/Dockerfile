FROM nvcr.io/nvidia/tritonserver:24.02-trtllm-python-py3

RUN --mount=type=cache,target=/root/.cache/pip \
    pip install sentencepiece protobuf