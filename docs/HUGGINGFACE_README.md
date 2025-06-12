# Fortean Qwen3-8B Advanced GGUF

A fine-tuned version of Qwen3-8B optimized for discussing anomalous phenomena, unexplained events, and scientific mysteries in the style of Charles Fort.

## Model Details

- **Base Model**: Qwen3-8B
- **Quantization**: Q4_K_M (4-bit quantization)
- **Format**: GGUF (for use with Ollama, llama.cpp, etc.)
- **Size**: ~5GB
- **Context Length**: 8192 tokens

## Important: Known Issue and Required Fix

⚠️ **This model has a known issue with endless text generation if not properly configured.** The model may not recognize end tokens and continue generating text indefinitely. The fix is simple but critical.

## Quick Start

### Download and Setup Script

```bash
#!/bin/bash
# Save this as setup-fortean.sh and run it

# Download model
wget https://huggingface.co/navicore/fortean-qwen3-8b-advanced-GGUF/resolve/main/fortean-q4_k_m.gguf

# Create Modelfile with fixes
cat > Modelfile << 'EOF'
FROM ./fortean-q4_k_m.gguf

TEMPLATE """Question: {{ .Prompt }}

Charles Fort: {{ .Response }}"""

PARAMETER temperature 0.8
PARAMETER top_p 0.9
PARAMETER repeat_penalty 1.1
PARAMETER num_predict 300
PARAMETER stop "<|endoftext|>"
PARAMETER stop "<|im_end|>"
PARAMETER stop "\nQuestion:"
PARAMETER stop "\n\nQuestion:"
PARAMETER stop "Question:"
EOF

# Create Ollama model
ollama create fortean-advanced -f Modelfile

# Test
ollama run fortean-advanced "What are your thoughts on ball lightning?"
```

## Manual Setup with Ollama

### 1. Download the Model
```bash
wget https://huggingface.co/navicore/fortean-qwen3-8b-advanced-GGUF/resolve/main/fortean-q4_k_m.gguf
```

### 2. Create a Modelfile (CRITICAL)
Create a file named `Modelfile` with the following content:

```modelfile
FROM ./fortean-q4_k_m.gguf

TEMPLATE """Question: {{ .Prompt }}

Charles Fort: {{ .Response }}"""

# Model parameters
PARAMETER temperature 0.8
PARAMETER top_p 0.9
PARAMETER repeat_penalty 1.1

# CRITICAL: Limit response length
PARAMETER num_predict 300

# CRITICAL: Stop tokens to prevent endless generation
PARAMETER stop "<|endoftext|>"
PARAMETER stop "<|im_end|>"
PARAMETER stop "\nQuestion:"
PARAMETER stop "\n\nQuestion:"
PARAMETER stop "Question:"
```

### 3. Create the Ollama Model
```bash
ollama create fortean-advanced -f Modelfile
```

### 4. Run the Model
```bash
ollama run fortean-advanced "Tell me about mysterious disappearances"
```

## Why the Modelfile Configuration is Critical

Without the proper stop tokens and response limiting, this model will:
- Continue generating text past natural stopping points
- Potentially repeat or loop content
- Generate very long, rambling responses

The configuration above ensures:
- Responses stop at appropriate points
- Maximum response length of ~300 tokens
- Natural conversation flow

## Example Usage

```bash
$ ollama run fortean-advanced "What do you make of the Bermuda Triangle?"

The Bermuda Triangle presents us with a delightful paradox - a region where 
the extraordinary has become so ordinary that we've given it a name and drawn 
it on maps. Ships and planes vanish, instruments malfunction, and time itself 
seems to hiccup, yet commerce continues unabated through these supposedly 
treacherous waters. Perhaps the real mystery isn't why vessels disappear, but 
why we're so eager to corral inexplicable events into neat geographical 
boundaries. The ocean, after all, needs no special triangle to swallow the 
unwary.
```

## Model Behavior

The model embodies Charles Fort's approach:
- Skeptical of both conventional explanations and wild theories
- Dry wit and intellectual curiosity
- Focus on documenting phenomena rather than explaining them
- Questions certainties while remaining open to possibilities

## Use Cases

- Exploring unexplained phenomena with balanced skepticism
- Generating thought-provoking perspectives on mysteries
- Educational content about anomalous events
- Creative writing with a Fortean perspective
- Research assistance for paranormal/anomalous topics

## Integration Examples

### With Python (llama-cpp-python)
```python
from llama_cpp import Llama

llm = Llama(
    model_path="./fortean-q4_k_m.gguf",
    n_ctx=2048,
    n_predict=300,
    stop=["Question:", "\nQuestion:", "<|endoftext|>", "<|im_end|>"]
)

response = llm(
    "Question: What are your thoughts on spontaneous human combustion?\n\nCharles Fort:",
    max_tokens=300,
    stop=["Question:", "\nQuestion:", "<|endoftext|>", "<|im_end|>"],
    echo=False
)
```

### With LangChain
```python
from langchain.llms import LlamaCpp

llm = LlamaCpp(
    model_path="./fortean-q4_k_m.gguf",
    temperature=0.8,
    max_tokens=300,
    stop=["Question:", "\nQuestion:", "<|endoftext|>", "<|im_end|>"]
)
```

## Technical Specifications

- **Architecture**: Qwen3 (Transformer-based)
- **Parameters**: 8B
- **Quantization**: 4-bit K-means (Q4_K_M)
- **Perplexity**: ~6.2 on validation set
- **File Format**: GGUF v3

## Limitations

- Requires proper stop token configuration
- Best for short to medium responses (300-500 tokens)
- May occasionally break character with very technical prompts
- English-only responses

## Citation

If you use this model in your work, please cite:

```
@misc{fortean-qwen3-8b-advanced,
  author = {navicore},
  title = {Fortean Qwen3-8B Advanced GGUF},
  year = {2024},
  publisher = {HuggingFace},
  url = {https://huggingface.co/navicore/fortean-qwen3-8b-advanced-GGUF}
}
```

## License

This model inherits the license from the base Qwen3 model. Please refer to the original model's license for usage terms.

## Acknowledgments

Based on the excellent Qwen3-8B model by Alibaba Cloud. Character development inspired by the works of Charles Fort (1874-1932).