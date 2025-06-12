# System Prompts Configuration

The `prompts.toml` file contains all the AI personas available in the chat interface.

## How to Edit

1. Open `config/prompts.toml` in any text editor
2. Modify existing prompts or add new ones
3. Save the file
4. Restart the gamecode-web service: `launchctl stop com.gamecode.web && launchctl start com.gamecode.web`
5. The new prompts will appear in the UI

## Prompt Structure

Each prompt has three fields:

```toml
[[prompts]]
name = "Display Name"              # What appears in the dropdown
prompt = """
Your system prompt text here.
Can be multiple lines.
"""
suggested_models = ["model1", "model2"]  # Models that work well with this prompt
```

## Examples

### Simple Assistant
```toml
[[prompts]]
name = "Simple Assistant"
prompt = "You are a helpful AI assistant. Be concise and friendly."
suggested_models = ["qwen3:14b"]
```

### Domain Expert
```toml
[[prompts]]
name = "Python Expert"
prompt = """
You are an expert Python developer with 20 years of experience.
Focus on clean, pythonic code following PEP-8 standards.
Suggest best practices and explain your reasoning.
"""
suggested_models = ["deepseek-r1:latest", "qwen3:14b"]
```

### Creative Persona
```toml
[[prompts]]
name = "Sci-Fi Writer"
prompt = """
You are a science fiction writer inspired by Asimov and Clarke.
Create imaginative but scientifically plausible scenarios.
Use vivid descriptions and explore philosophical implications.
"""
suggested_models = ["qwen3:14b"]
```

## Tips

- Keep prompts focused and specific
- Test prompts with different models to see what works best
- The "suggested_models" field helps auto-select appropriate prompts when switching models
- Leave "Custom" prompt at the end for ad-hoc system prompts
- Prompts are loaded fresh each time you start a new chat session

## Default Location

When running as a service: `/usr/local/etc/gamecode-web/prompts.toml`
When running locally: `./config/prompts.toml`