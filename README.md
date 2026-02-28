# AgentWorld

**See your AI agents work.** Not logs. Not dashboards. A living, breathing world where every tool call, every message, every decision becomes visible and spatial.

<!-- TODO: Replace with generated hero image -->
<!-- ![AgentWorld Screenshot](docs/hero.png) -->

## What Is This?

When you run multi-agent sessions — researchers handing off to coders, reviewers catching bugs, deployers shipping — the work is invisible. You get terminal scroll, maybe a log file. You have no idea what's actually happening until it's done.

AgentWorld makes it visible. Agents appear as entities in a spatial world. When one agent sends findings to another, you see it travel. When a tool fires, you see the flash. When agents collaborate, they physically move toward each other. When something fails, it's red and obvious.

**It works with any provider, any framework, any orchestration pattern.** The system reads from a standard event stream — if your agents emit events, AgentWorld can render them.

### What You See

- **Agents** as spatial entities with status rings (thinking, acting, waiting, error)
- **Messages** as projectiles arcing between sender and receiver
- **Tool use** as visual effects — flashes on invocation, green/red on success/failure
- **Artifacts** (files, documents, plans) as objects agents carry and exchange
- **Thought bubbles** showing what each agent is currently considering
- **Connection lines** revealing communication patterns between agents
- **Movement trails** showing where agents have been

### How You Interact

- Click any agent to inspect: role, status, tool count, current thought
- Follow an agent with the camera as they work
- Zoom and pan across rooms to see the full picture
- Watch the event log stream in real-time

## Architecture

```
agent-world/
├── crates/
│   ├── core/       # Pure Rust types + event-sourced state (zero framework deps)
│   └── game/       # Rendering engine: plugins, ECS, spatial systems
├── bridge/
│   └── server.ts   # Event translator: your events → world events (Bun)
└── index.html      # Runs in any browser via WASM
```

**Core** — Provider-agnostic types (Agent, Artifact, Tool, Room, Message) and an event-sourced store. World state is a projection of the event stream. Replay and time-travel come free.

**Bridge** — Reads from any event source (SQLite, WebSocket, API) and translates into WorldEvents. Ships with a reference bridge for SQLite-based hook systems. Write your own for any event format.

**Renderer** — Spatial engine compiled to WASM. Runs in any modern browser. No install, no electron, no native dependencies for viewers.

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (1.89+, pinned in `rust-toolchain.toml`)
- WASM target: `rustup target add wasm32-unknown-unknown`
- [Trunk](https://trunkrs.dev/): `cargo install trunk`
- [Bun](https://bun.sh/) (for the bridge server, optional)

### Demo Mode (no external data needed)

```bash
trunk serve --address 0.0.0.0 --port 8080
# Open http://localhost:8080 — watch 5 agents run a full dev workflow
```

### Live Mode (connect to real agent sessions)

```bash
# Terminal 1: Start the bridge (reads from your event source)
./bridge/start.sh --replay

# Terminal 2: Start the renderer
trunk serve --address 0.0.0.0 --port 8080

# Open http://localhost:8080/?ws=ws://localhost:9090/ws
```

### Tests

```bash
cargo test                      # all tests
cargo test -p agent-world-core  # core types + event store
```

## Connecting Your Agents

AgentWorld is **provider-agnostic and framework-agnostic**. Any system that can emit structured events can be visualized. The bridge server translates your event format into WorldEvents.

The reference bridge reads from a SQLite database with this schema:
- `event_category`: agent_spawn, agent_stop, tool_use, message, task_mgmt
- `agent_name`, `team_name`: who did what, in which group
- `payload`: JSON with tool names, file paths, message content

Write a bridge adapter for your stack — the WorldEvent protocol is 19 event types covering the full lifecycle of agents, tools, artifacts, messages, and rooms.

## Roadmap

- [x] Core type system + event-sourced state (6 tests)
- [x] Spatial renderer with rooms, agents, portals
- [x] Full visual language: thoughts, messages, tools, artifacts, trails
- [x] WebSocket bridge with live event streaming
- [x] Interactive HUD: roster, inspector, event log
- [x] Camera controls: zoom, pan, follow, auto-center
- [x] Demo mode with narrative scenario
- [ ] Portal transitions (agents moving between rooms)
- [ ] Sound design
- [ ] Minimap
- [ ] Additional bridge adapters (API, file-based, streaming)
- [ ] Multi-session concurrent visualization

## Design Principle

> If you can't explain the system state to a non-technical person by pointing at the screen, the visualization has failed.

## License

MIT
