import sys
import os
import torch
import argparse
import time
import json
from flask import Flask, jsonify, request

from exllamav2 import(
    ExLlamaV2,
    ExLlamaV2Config,
    ExLlamaV2Cache,
    ExLlamaV2Tokenizer,
    model_init,
)

from exllamav2.generator import (
    ExLlamaV2SpeculativeGenerator,
    ExLlamaV2StreamingGenerator,
    ExLlamaV2Sampler
)

app = Flask(__name__)

class ChatAssistant:

    def __init__(self, args):
        model_init.check_args(args)
        model_init.print_options(args)

        self.max_seq_len = 4096

        draft_config = ExLlamaV2Config()
        draft_config.model_dir = "models/llm/turboderp/Llama2-7B-exl2"
        draft_config.prepare()
        draft_config.max_seq_len = self.max_seq_len
        draft_config.rope_scale = args.rope_scale
        draft_config.rope_alpha = args.rope_alpha
        draft_config.no_flash_attn = args.no_flash_attn


        model_config = ExLlamaV2Config()
        model_config.model_dir = args.model_dir
        model_config.prepare()
        model_config.max_seq_len = self.max_seq_len
        model_config.rope_scale = args.rope_scale
        model_config.rope_alpha = args.rope_alpha
        model_config.no_flash_attn = args.no_flash_attn

        # Initialize model and tokenizer
        draft = ExLlamaV2(draft_config)
        draft.load([24, 0])
        model = ExLlamaV2(model_config)
        model.load([16, 24])

        # Create cache
        model_cache = ExLlamaV2Cache(model)
        draft_cache = ExLlamaV2Cache(draft)

        self.tokenizer = ExLlamaV2Tokenizer(model_config)
        self.generator = ExLlamaV2SpeculativeGenerator(model, model_cache, draft, draft_cache, self.tokenizer)

        # Set up generator settings
        self.settings = ExLlamaV2Sampler.Settings()
        self.settings.temperature = args.temperature
        self.settings.top_k = args.top_k
        self.settings.top_p = args.top_p
        self.settings.typical = args.typical
        self.settings.token_repetition_penalty = args.repetition_penalty

        # Set up generator
        # if args.mode in {"llama", "codellama"}:
        #     self.generator.set_stop_conditions([self.tokenizer.eos_token_id])
        # elif args.mode == "raw":
        #     self.generator.set_stop_conditions([args.username + ":", args.username[0:1] + ":", args.username.upper() + ":", args.username.lower() + ":", self.tokenizer.eos_token_id])

        # Set up prompt templates
        self.username = args.username
        self.botname = args.botname
        self.mode = args.mode
        self.max_response_tokens = args.max_response_tokens
        self.min_space_in_context = args.response_chunk

        if self.mode == "llama" or self.mode == "codellama":
            self.first_prompt = """[INST] <<SYS>>\n<|system_prompt|>\n<</SYS>>\n\n<|user_prompt|> [/INST]"""
            self.subs_prompt = """[INST] <|user_prompt|> [/INST]"""
        elif self.mode == "raw":
            self.first_prompt = f"""<|system_prompt|>\n{self.username}: <|user_prompt|>\n{self.botname}:"""
            self.subs_prompt = f"""{self.username}: <|user_prompt|>\n{self.botname}:"""

    def format_prompt(self,system, user_prompt, first):
        if first:
            return self.first_prompt.replace("<|system_prompt|>", system_prompt).replace("<|user_prompt|>", user_prompt)
        else:
            return self.subs_prompt.replace("<|user_prompt|>", user_prompt)

    def encode_prompt(self, text):
        if self.mode in {"llama", "codellama"}:
            return self.tokenizer.encode(text, add_bos=True)
        elif self.mode == "raw":
            return self.tokenizer.encode(text)

    def get_tokenized_context(self, json, max_len):

        while True:
            context = torch.empty((1, 0), dtype=torch.long)
            prompt = ""

            for i, msg in enumerate(json['conversation']):
                if i == 0:
                    prompt = self.first_prompt.replace("<|system_prompt|>", json['system']).replace("<|user_prompt|>", msg["message"])
                elif msg["role"] == "user":
                    prompt = self.subs_prompt.replace("<|user_prompt|>", msg['message'])
                else:  # role == "assistant"
                    prompt = msg["message"]

                prompt_ids = self.encode_prompt(prompt)
                context = torch.cat([context, prompt_ids], dim=-1)

            if context.shape[-1] < max_len:
                print("Input context length: " + str(len(context[0])))
                return context

            # If the context is too long, remove the first Q/A pair and try again.
            conversation = conversation[2:]
            
    def get_context(self, json, max_len):

        while True:
            context_text = ""
            context = torch.empty((1, 0), dtype=torch.long)
            prompt = ""

            for i, msg in enumerate(json['conversation']):
                if i == 0:
                    prompt = self.first_prompt.replace("<|system_prompt|>", json['system']).replace("<|user_prompt|>", msg["message"])
                elif msg["role"] == "user":
                    prompt = self.subs_prompt.replace("<|user_prompt|>", msg['message'])
                else:  # role == "assistant"
                    prompt = msg["message"]

                prompt_ids = self.encode_prompt(prompt)
                context_text += prompt
                context = torch.cat([context, prompt_ids], dim=-1)

            if context.shape[-1] < max_len:
                print("Input context length: " + str(len(context[0])))
                return context_text

            # If the context is too long, remove the first Q/A pair and try again.
            conversation = conversation[2:]
    def generate_response(self, conversation):
        max_new_tokens = 1000
        active_context = self.get_context( conversation, self.max_seq_len - self.min_space_in_context)
        start = time.time()
        output = self.generator.generate_simple(active_context, self.settings, max_new_tokens, seed = 31337)
        time_taken = time.time() - start
        tokens_per_second = 1000 / time_taken
        print(f"Response generated in {time_taken:.2f} seconds, {max_new_tokens} tokens, {max_new_tokens / time_taken:.2f} tokens/second")
        print()
        print("Prediction attempts:", self.generator.attempts)
        print("Prediction hits:", self.generator.hits)
        return {"role": "assistant", "message": output}



    def generate_streaming_response(self, conversation):

        active_context = self.get_tokenized_context( conversation, self.max_seq_len - self.min_space_in_context)
        print("Input context length: " + str(len(active_context[0])))
        start = time.time()
        self.generator.begin_stream(active_context, self.settings)

        # Stream response
        response_tokens = 0
        response_text = ""

        while True:

            # Get response stream
            chunk, eos, tokens = self.generator.stream()
            if len(response_text) == 0:
                chunk = chunk.lstrip()
            response_text += chunk

            # If model has run out of space, rebuild the context and restart stream
            if self.generator.full():
                active_context = self.get_tokenized_context(conversation, self.max_seq_len - self.min_space_in_context)
                self.generator.begin_stream(active_context, self.settings)

            response_tokens += 1
            if response_tokens == self.max_response_tokens:
                if self.tokenizer.eos_token_id in self.generator.stop_tokens:
                    tokens = torch.cat([tokens, self.tokenizer.single_token(self.tokenizer.eos_token_id)], dim=-1)

                break

            # EOS signal returned
            if eos:
                break

        time_taken = time.time() - start
        tokens_per_second = response_tokens / time_taken
        print("Tokens per second:", tokens_per_second)
        return {"role": "assistant", "message": response_text}

@app.route('/conversation', methods=['POST'])
def conversation():
    json_data = request.json
    response = assistant.generate_response(json_data)
    json_data['conversation'].append(response)
    return jsonify(json_data)

if __name__ == '__main__':

    parser = argparse.ArgumentParser(description="Simple Llama2 chat example for ExLlamaV2")
    parser.add_argument("-mode", "--mode", choices=["llama", "raw", "codellama"], help="Chat mode. Use llama for Llama 1/2 chat finetunes.")
    parser.add_argument("-un", "--username", type=str, default="User", help="Username when using raw chat mode")
    parser.add_argument("-bn", "--botname", type=str, default="Chatbort", help="Bot name when using raw chat mode")

    parser.add_argument("-temp", "--temperature", type=float, default=0.95, help="Sampler temperature, default = 0.95 (1 to disable)")
    parser.add_argument("-topk", "--top_k", type=int, default=50, help="Sampler top-K, default = 50 (0 to disable)")
    parser.add_argument("-topp", "--top_p", type=float, default=0.8, help="Sampler top-P, default = 0.8 (0 to disable)")
    parser.add_argument("-typical", "--typical", type=float, default=0.0, help="Sampler typical threshold, default = 0.0 (0 to disable)")
    parser.add_argument("-repp", "--repetition_penalty", type=float, default=1.1, help="Sampler repetition penalty, default = 1.1 (1 to disable)")
    parser.add_argument("-maxr", "--max_response_tokens", type=int, default=1000, help="Max tokens per response, default = 1000")
    parser.add_argument("-resc", "--response_chunk", type=int, default=250, help="Space to reserve in context for reply, default = 250")
    parser.add_argument("-port", "--port", type=int, default=5050, help="Port on which to expose the service")

    model_init.add_args(parser)
    args = parser.parse_args()

    assistant = ChatAssistant(args)
    app.run(host='0.0.0.0', port=args.port, debug=False)