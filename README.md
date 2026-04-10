# geny-harness

Rust-powered agent pipeline library — drop-in replacement for [geny-executor](https://github.com/CocoRoF/geny-executor).

16-stage dual-abstraction architecture built on the Anthropic API, implemented in Rust with PyO3 bindings.

## Installation

```bash
pip install geny-harness
```

## Quick Start

```python
from geny_harness import PipelineConfig, PipelineState, PipelineResult, TokenUsage

config = PipelineConfig(name="my-agent", api_key="sk-...")
state = PipelineState()
config.apply_to_state(state)
```

## Architecture

Same 16-stage pipeline as geny-executor:

| Phase | Stages | Description |
|-------|--------|-------------|
| **A: Input** | S01 | Input validation & normalization |
| **B: Agent Loop** | S02-S13 | Context, System, Guard, Cache, API, Token, Think, Parse, Tool, Agent, Evaluate, Loop |
| **C: Finalize** | S14-S16 | Emit, Memory, Yield |

## License

MIT
