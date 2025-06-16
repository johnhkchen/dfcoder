# DFCoder Project Documentation
## Dwarf Fortress Style AI Agent Manager

### Project Overview

DFCoder is a blazingly fast terminal-based AI agent management system inspired by Dwarf Fortress aesthetics and Baldur's Gate 3 dialogue mechanics. It provides real-time monitoring, supervision, and control of multiple Claude Code agents through a rich TUI interface with Zellij integration.

### Core Features

1. **Real-time Pane Monitoring**: Sub-second polling of Claude Code agent panes
2. **BG3-Style Dialogue System**: Interactive supervision with contextual dialogue options
3. **Fortress-Style Overview**: Visual representation of agent status in themed "rooms"
4. **Instant Navigation**: Jump between agent panes with single keystrokes
5. **Performance Metrics**: Live tracking of API usage, task completion, and agent efficiency
6. **Zellij Integration**: Native plugin for terminal multiplexer workflow

### Architecture

#### System Components

```
┌─────────────────────────────────────────────────────────────┐
│                        DFCoder System                        │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │   TUI App   │  │ Zellij Plugin│  │  Daemon Service  │  │
│  │  (dfcoder)  │  │   (WASM)     │  │   (dfcoderd)     │  │
│  └──────┬──────┘  └──────┬───────┘  └────────┬─────────┘  │
│         │                 │                    │             │
│  ┌──────┴─────────────────┴───────────────────┴─────────┐  │
│  │                    Core Library                       │  │
│  │               (dfcoder-core + types)                  │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

#### Key Subsystems

1. **Pane Monitor**: High-frequency polling system for agent output
2. **Supervision System**: Context-aware dialogue generation
3. **Navigation System**: Instant pane jumping with history
4. **Cache Layer**: Performance-critical state caching
5. **Event Loop**: Responsive input handling and rendering

### Implementation Plan

#### Phase 1: Core Foundation (Week 1-2)
- [ ] Set up Rust workspace with all crates
- [ ] Implement basic TUI with ratatui
- [ ] Create pane monitoring system
- [ ] Basic agent state management

#### Phase 2: Supervision System (Week 3-4)
- [ ] Implement dialogue context detection
- [ ] Create BG3-style dialogue UI
- [ ] Add supervision action handlers
- [ ] Integrate with pane communication

#### Phase 3: Zellij Integration (Week 5-6)
- [ ] Build WASM plugin structure
- [ ] Implement IPC protocol
- [ ] Create plugin UI components
- [ ] Test pane capture/control APIs

#### Phase 4: Advanced Features (Week 7-8)
- [ ] Add metrics tracking
- [ ] Implement task queue management
- [ ] Create fortress-style visualizations
- [ ] Add configuration system

### Technical Specifications

#### Performance Requirements
- Pane polling: < 50ms per cycle
- UI render: 60 FPS target
- Memory usage: < 100MB for 10 agents
- Startup time: < 1 second

#### API Integrations
- Claude Code: Monitor and control via terminal
- GitHub: Optional PR/issue integration
- OpenAI/Anthropic: Agent communication APIs

#### Data Structures

```rust
// Core agent representation
struct Agent {
    id: String,
    name: String,
    pane_id: u32,
    status: PaneStatus,
    current_task: Option<Task>,
    metrics: AgentMetrics,
}

// Supervision dialogue
struct DialogueContext {
    agent_name: String,
    situation: String,
    options: Vec<DialogueOption>,
    timestamp: Instant,
}

// Performance tracking
struct AgentMetrics {
    tasks_completed: u32,
    success_rate: f32,
    api_calls: u32,
    total_cost: f32,
}
```

### User Interface Design

#### View Modes
1. **Overview**: Fortress-style room layout showing all agents
2. **Agents**: Detailed agent management with live status
3. **Tasks**: Queue management and assignment
4. **Code Review**: Integration with version control
5. **Metrics**: Performance dashboards
6. **Logs**: Searchable activity history

#### Key Interactions
- `1-6`: Switch between view tabs
- `j`: Jump to selected agent's pane
- `r`: Force refresh all agents
- `q`: Quit application
- Dialogue: Number keys for option selection

### Configuration

```toml
# ~/.config/dfcoder/config.toml
[general]
poll_frequency_ms = 100
max_agents = 10
theme = "monokai"

[supervision]
auto_respond_delay_ms = 5000
dialogue_timeout_s = 30
context_lines = 5

[api]
openai_key = "sk-..."
anthropic_key = "sk-..."
rate_limit_per_minute = 60

[ui]
show_fortress_view = true
enable_animations = true
vim_mode = false
```

### Development Guidelines

#### Code Style
- Use descriptive variable names in core logic
- Optimize hot paths with minimal allocations
- Implement comprehensive error handling
- Add tracing for debugging

#### Testing Strategy
- Unit tests for core logic
- Integration tests for Zellij plugin
- E2E tests for full workflows
- Performance benchmarks

#### Build Process
```bash
# Development build
cargo build --workspace

# Release build with optimizations
./scripts/build-release.sh

# Install locally
./scripts/install.sh

# Run tests
cargo test --workspace
```

### Deployment

#### Installation Methods
1. **Binary Release**: Pre-built binaries for major platforms
2. **Cargo Install**: `cargo install dfcoder`
3. **Package Managers**: AUR, Homebrew (planned)
4. **Docker**: Containerized daemon service

#### System Requirements
- OS: Linux, macOS, Windows (WSL2)
- Terminal: 256 color support
- Zellij: v0.38+ (for plugin)
- Rust: 1.70+ (for building)

### Future Enhancements

1. **Multi-Model Support**: Integrate GPT-4, Claude, local models
2. **Web Dashboard**: Browser-based monitoring
3. **Mobile App**: iOS/Android companion
4. **Cloud Sync**: Cross-device agent state
5. **Plugin System**: Custom agent types and behaviors
6. **Voice Control**: "Hey DFCoder, check on TypeScript_Expert"

### Getting Started

```bash
# Clone repository
git clone https://github.com/yourusername/dfcoder
cd dfcoder

# Build everything
cargo build --workspace

# Run TUI
cargo run --bin dfcoder

# In another terminal, start daemon
cargo run --bin dfcoderd

# Install Zellij plugin
cd crates/dfcoder-zellij-plugin
cargo build --target wasm32-wasi --release
cp target/wasm32-wasi/release/*.wasm ~/.config/zellij/plugins/
```

### License

MIT OR Apache-2.0

### Contributing

See CONTRIBUTING.md for guidelines on:
- Code style and conventions
- Testing requirements
- PR process
- Feature proposals

---

**Status**: Ready for implementation
**Version**: 1.0.0
**Last Updated**: 2025-06-16
