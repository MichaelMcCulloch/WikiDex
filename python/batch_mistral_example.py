
import sys, os
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from exllamav2 import(
    ExLlamaV2,
    ExLlamaV2Config,
    ExLlamaV2Cache,
    ExLlamaV2Tokenizer,
)

from exllamav2.generator import (
    ExLlamaV2BaseGenerator,
    ExLlamaV2Sampler
)

import time

# Initialize model and cache

model_directory =  "models/llm/TheBloke/zephyr-7B-alpha-GPTQ/gptq-4bit-32g-actorder_True/"



prompt0 = "What is the capital of France?"
prompt1 = "Who is the president of France?"
prompt2 = "Where was the president of France born?"
prompt3 = "What is the capital of Canada?"
prompt4 = "Who is the president of Canada?"
prompt5 = "Where was the president of Canada born?"
prompts = [prompt0,prompt1,prompt2,prompt3,]

config = ExLlamaV2Config()
config.model_dir = model_directory
config.max_batch_size=(len(prompts))
config.prepare()
model = ExLlamaV2(config)
print("Loading model: " + model_directory)

# allocate 18 GB to CUDA:0 and 24 GB to CUDA:1.
# (Call `model.load()` if using a single GPU.)
model.load([18, 24])
tokenizer = ExLlamaV2Tokenizer(config)
cache = ExLlamaV2Cache(model, batch_size=config.max_batch_size)
# Initialize generator

generator = ExLlamaV2BaseGenerator(model, cache, tokenizer)

# Generate some text

settings = ExLlamaV2Sampler.Settings()
settings.temperature = 0.85
settings.top_k = 50
settings.top_p = 0.8
settings.token_repetition_penalty = 1.15
settings.disallow_tokens(tokenizer, [tokenizer.eos_token_id])


max_new_tokens = 1500

generator.warmup()
time_begin = time.time()

output = generator.generate_simple(prompts, settings, max_new_tokens, seed = 1234)

time_end = time.time()
time_total = time_end - time_begin

print(output)
print()
print(f"Response generated in {time_total:.2f} seconds, {max_new_tokens * len(prompts)} tokens, {max_new_tokens * len(prompts) / time_total:.2f} tokens/second")
