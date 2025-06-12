# Fortean Model Setup Guide

This guide explains how to properly set up the Fortean model for use with Ollama, including critical fixes for the endless response issue.

## Problem: Endless Response Generation

The Fortean model (based on Qwen3-8B) has a known issue where it doesn't properly recognize end tokens and continues generating text indefinitely. This results in:
- Responses that repeat or ramble
- The model continuing past natural stopping points
- Excessive token usage and slow responses

## Solution: Proper Modelfile Configuration

The fix involves configuring multiple stop tokens and response limits in the Modelfile.

### Step 1: Download the Model

```bash
# Download from HuggingFace (about 5GB)
wget https://huggingface.co/navicore/fortean-qwen3-8b-advanced-GGUF/resolve/main/fortean-q4_k_m.gguf
```

### Step 2: Create the Modelfile

Create a file named `Modelfile` with the following content:

```modelfile
FROM ./fortean-q4_k_m.gguf

TEMPLATE """Question: {{ .Prompt }}

Charles Fort: {{ .Response }}"""

# Temperature and sampling parameters
PARAMETER temperature 0.8
PARAMETER top_p 0.9
PARAMETER repeat_penalty 1.1

# CRITICAL: Response length limit
PARAMETER num_predict 300

# CRITICAL: Stop tokens to prevent endless generation
PARAMETER stop "<|endoftext|>"
PARAMETER stop "<|im_end|>"
PARAMETER stop "\nQuestion:"
PARAMETER stop "\n\nQuestion:"
PARAMETER stop "Question:"
PARAMETER stop "\n\nYou:"
PARAMETER stop "\n\nUser:"
PARAMETER stop "\n\nHuman:"
```

### Step 3: Create the Ollama Model

```bash
# Remove any existing version
ollama rm fortean-advanced 2>/dev/null

# Create the model with fixes
ollama create fortean-advanced -f Modelfile
```

### Step 4: Test the Model

```bash
# Test with a simple prompt
ollama run fortean-advanced "What are your thoughts on UFO sightings?"

# The response should:
# - Stop naturally after a complete thought
# - Not repeat or continue indefinitely
# - Be limited to ~300 tokens maximum
```

## Key Configuration Details

### Stop Tokens
- `<|endoftext|>` and `<|im_end|>` - Model's native end tokens
- `Question:` variations - Prevents continuing into new questions
- `You:/User:/Human:` - Prevents role confusion

### Response Limiting
- `num_predict 300` - Hard limit on response length
- `repeat_penalty 1.1` - Discourages repetitive text

### Template Structure
The template format helps the model understand the conversation structure:
- User input is prefixed with "Question:"
- Model response is prefixed with "Charles Fort:"
- This structure helps trigger stop tokens naturally

## Integration with GameCode Web

When using this model with the GameCode Web application:

1. Update `config/default.toml`:
```toml
[providers.ollama]
enabled = true
base_url = "http://localhost:11434"
default_model = "fortean-advanced"
timeout_seconds = 60
```

2. The model will appear in the UI's model selector dropdown

3. The server-side code already includes additional safeguards:
   - Monitors for stop patterns in the response stream
   - Cuts off responses that contain certain markers
   - See `server/src/providers/ollama.rs` lines 179-188

## Troubleshooting

### Model Still Generates Endless Responses
1. Verify all stop tokens are in the Modelfile
2. Check that `num_predict` is set
3. Try lowering `num_predict` to 200 or 150
4. Add more aggressive stop tokens like `"\n---\n"`

### Model Cuts Off Too Early
1. Increase `num_predict` to 400-500
2. Remove some of the more aggressive stop tokens
3. Adjust the template structure

### Performance Issues
1. The q4_k_m quantization is a good balance of quality/performance
2. For faster responses, try q4_0 quantization (lower quality)
3. For better quality, try q8_0 quantization (slower)

## References

- Original issue discovery: `../fortean-explanations/FORTEAN_MODEL_GUIDE.md`
- Working configuration: `../fortean-explanations/fortean-gguf-final/README.md`
- HuggingFace model: https://huggingface.co/navicore/fortean-qwen3-8b-advanced-GGUF

## Quick Setup Script

Save this as `setup-fortean.sh`:

```bash
#!/bin/bash
set -e

echo "Setting up Fortean model with fixes..."

# Download model
if [ ! -f "fortean-q4_k_m.gguf" ]; then
    echo "Downloading model from HuggingFace..."
    wget https://huggingface.co/navicore/fortean-qwen3-8b-advanced-GGUF/resolve/main/fortean-q4_k_m.gguf
else
    echo "Model file already exists, skipping download"
fi

# Create Modelfile
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
PARAMETER stop "\n\nYou:"
PARAMETER stop "\n\nUser:"
PARAMETER stop "\n\nHuman:"
EOF

# Create Ollama model
echo "Creating Ollama model..."
ollama rm fortean-advanced 2>/dev/null || true
ollama create fortean-advanced -f Modelfile

echo "Setup complete! Test with: ollama run fortean-advanced \"What are your thoughts on synchronicity?\""
```

Make it executable with `chmod +x setup-fortean.sh` and run it whenever you need to set up the model.