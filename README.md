# geny-harness

**Rust-core Python library** — A drop-in replacement for [geny-executor](https://github.com/CocoRoF/geny-executor), providing the same 16-stage dual-abstraction agent pipeline architecture.

```bash
pip install geny-harness
```

## geny-executor vs geny-harness: What's Different?

Both libraries provide **identical Python APIs** — the same classes, same methods, same import paths. You can switch between them by changing a single import. The difference is **where the code runs**.

### Architecture Comparison

```
geny-executor (v0.3.0)                    geny-harness (v0.3.0)
┌─────────────────────────┐               ┌─────────────────────────┐
│     Python Application  │               │     Python Application  │
├─────────────────────────┤               ├─────────────────────────┤
│  Pipeline Orchestration │  Python       │  Pipeline Orchestration │  Python
│  16 Stage Execution     │               │  16 Stage Execution     │
│  EventBus / Session     │               │  EventBus / Session     │
├─────────────────────────┤               ├─────────────────────────┤
│  PipelineState (40+fld) │  Python       │  PipelineState (40+fld) │  Rust (PyO3)
│  TokenUsage / Metrics   │  dataclass    │  TokenUsage / Metrics   │  Rust struct
│  PipelineConfig         │               │  PipelineConfig         │
│  PipelineResult         │               │  PipelineResult         │
│  PipelineEvent          │               │  PipelineEvent          │
│  Error Hierarchy        │               │  Error Hierarchy        │  Rust enum
├─────────────────────────┤               ├─────────────────────────┤
│          N/A            │               │  Full Rust Core Engine  │  148 .rs files
│                         │               │  (16 stages, 60+ strat.)│  Pure Rust
│                         │               │  reqwest HTTP client    │
│                         │               │  tokio async runtime    │
└─────────────────────────┘               └─────────────────────────┘
   176 Python files                          144 Rust + 34 Python files
   deps: anthropic, pydantic, mcp           deps: Rust (compiled), anthropic
```

### What's Actually in Rust

The **data layer** is implemented in Rust and exposed to Python via PyO3:

| Type | Python (executor) | Rust (harness) |
|------|-------------------|----------------|
| `PipelineState` | `@dataclass` (40+ fields) | Rust `struct` → `#[pyclass]` |
| `TokenUsage` | `@dataclass` + `__add__` | Rust `struct` + `Add` trait |
| `PipelineConfig` | `@dataclass` | Rust `struct` with `apply_to_state()` |
| `PipelineResult` | `@dataclass` + `from_state()` | Rust `struct` + class methods |
| `PipelineEvent` | `@dataclass` | Rust `struct` |
| `ErrorCategory` | `str, Enum` | Rust `enum` + `is_recoverable()` |
| `StageDescription` | `@dataclass` | Rust `struct` |
| Exceptions (6 types) | Python `Exception` subclasses | Rust error hierarchy → PyO3 exceptions |

Additionally, geny-harness contains a **complete Rust implementation** of the entire pipeline engine (148 `.rs` files), including all 16 stages and 60+ strategy implementations. This Rust core is designed for:

- **Future native execution** — bypassing Python entirely for maximum throughput
- **Embedding in Rust applications** — use the pipeline engine directly from Rust/C/C++
- **WebAssembly compilation** — run the pipeline in browsers or edge environments

### What's Still in Python (Same as executor)

The pipeline **orchestration layer** remains in Python for both libraries:
- `Pipeline` class (3-phase execution engine)
- `EventBus` (pub/sub with pattern matching)
- `Session` / `SessionManager`
- `PipelineBuilder` / `PipelinePresets`
- All 16 stage implementations (using `anthropic` SDK for API calls)

This means the actual pipeline execution flow is identical. The Anthropic API calls, streaming, tool execution, and agent loops work exactly the same way.

### Performance Characteristics

| Operation | executor (Python) | harness (Rust+PyO3) | Notes |
|-----------|------------------|--------------------|----|
| State creation | ~195ms/100K | ~96ms/100K | **Rust 2x faster** — 40+ field struct init |
| Field access | ~46ms/500K | ~393ms/500K | **Python faster** — PyO3 boundary cost |
| Token arithmetic | ~189ms/500K | ~234ms/500K | Similar — PyO3 overhead offsets Rust speed |
| Result.from_state | ~88ms/100K | ~90ms/100K | Identical |

**Key insight**: For the current Python-orchestrated usage pattern, there is no significant performance difference. The real performance advantage of geny-harness will emerge when:
1. The Rust pipeline engine is used natively (without Python)
2. Multiple pipelines run concurrently via tokio
3. State serialization/deserialization is done in Rust (JSON, MessagePack)

### When to Use Which

| Scenario | Recommendation |
|----------|---------------|
| Standard Python project | **geny-executor** — simpler, pure Python, easier to debug |
| Want Rust core for future native use | **geny-harness** — invest in Rust ecosystem now |
| Embedding in Rust application | **geny-harness** — use `geny-harness-core` crate directly |
| Need to modify stage internals | **geny-executor** — all Python, easy to fork/modify |
| Production with high concurrency | **geny-harness** — Rust core ready for tokio-based scaling |
| Learning/prototyping | **geny-executor** — more straightforward |

## Quick Start

```python
# Identical to geny-executor — just change the import
from geny_harness import PipelinePresets

pipeline = PipelinePresets.agent(
    api_key="sk-ant-...",
    model="claude-sonnet-4-20250514",
    system_prompt="You are a helpful assistant.",
)

result = await pipeline.run("Hello!")
print(result.text)
```

### Drop-in Replacement

```python
# Before (geny-executor)
from geny_executor import Pipeline, PipelineConfig, PipelinePresets
from geny_executor.session.manager import SessionManager

# After (geny-harness) — just change the package name
from geny_harness import Pipeline, PipelineConfig, PipelinePresets
from geny_harness.session.manager import SessionManager
```

## Project Structure

```
geny-harness/
├── crates/
│   ├── geny-harness-core/    # Pure Rust library (148 .rs files)
│   │   └── src/
│   │       ├── core/          # Pipeline, State, Config, Errors, Builder
│   │       ├── events/        # EventBus, PipelineEvent
│   │       ├── session/       # Session, Manager, Freshness
│   │       ├── tools/         # Tool, Registry, MCP
│   │       └── stages/        # 16 stages × (interface + types + artifact)
│   └── geny-harness-py/      # PyO3 bindings (cdylib)
├── python/
│   └── geny_harness/          # Python package
│       ├── core/              # Pipeline, Builder, Presets, Stage
│       ├── events/            # EventBus
│       ├── session/           # Session, Manager
│       ├── stages/            # 16 stage implementations
│       └── tools/             # Tool, Registry
└── tests/
```

## Supported Platforms

- **Python**: 3.10 — 3.14
- **OS**: Linux (x86_64), Windows (x64), macOS (arm64)
- **Rust**: 2021 edition (for building from source)

## License

MIT
