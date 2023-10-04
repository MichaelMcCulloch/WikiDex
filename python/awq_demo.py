""" 
slight speed decrease from GPTQ, but (purportedly) lower perplexity
```trick pip into installing deps for cuda/rocm
pip3 install torch
git clone https://github.com/casper-hansen/AutoAWQ
pip install  ./AutoAWQ
pip install  torch==2.2.0.dev20230929+cu121 torchvision==0.17.0.dev20230929+cu121  --extra-index-url https://download.pytorch.org/whl/nightly/cu121
pip install  ./AutoAWQ torch==2.2.0.dev20230929+cu121 torchvision==0.17.0.dev20230929+cu121  --extra-index-url https://download.pytorch.org/whl/nightly/cu121
```
"""
import time
from awq import AutoAWQForCausalLM
from transformers import AutoTokenizer, TextStreamer

quant_path = "models/llm/TheBloke/Speechless-Llama2-Hermes-Orca-Platypus-WizardLM-13B-AWQ/"

# Load model
model = AutoAWQForCausalLM.from_quantized(quant_path, fuse_layers=True, safetensors=True)
tokenizer = AutoTokenizer.from_pretrained(quant_path, trust_remote_code=True)
streamer = TextStreamer(tokenizer, skip_special_tokens=True)

# Convert prompt to tokens
prompt_template = """\
A chat between a curious user and an artificial intelligence assistant. The assistant gives helpful, detailed, and polite answers to the user's questions.

USER: {prompt}
ASSISTANT:"""

tokens = tokenizer(
    prompt_template.format(prompt="Can you please write out the entire story of genesis, and provide a summary?"), 
    return_tensors='pt'
).input_ids.cuda()


start = time.time()
# Generate output
generation_output = model.generate(
    tokens, 
    streamer=streamer,
    max_new_tokens=512
)

time_taken = time.time() - start
tokens_per_second = streamer.i / time_taken
print("Tokens per second:", tokens_per_second)