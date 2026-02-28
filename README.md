# AgentWorld

A game-native visibility layer for multi-agent systems. Think Zelda, but agents are characters, tools are weapons, artifacts are items, and messages are visible projectiles.

## The Idea

Current agent observability is stuck between terminal text and workflow diagrams. Games solved spatial awareness of autonomous actors decades ago. AgentWorld renders multi-agent systems as a 2D game world where you can **see** agents working, **watch** messages travel between them, and **interact** through a spatial interface.

**Design north star:** If you can't explain the system state to a non-technical person by pointing at the screen, the visualization has failed.

## Architecture

```
agent-world/
├── crates/
│   ├── core/     # Pure types + event-sourced state (no Bevy)
│   └── game/     # Bevy 0.18 app: rendering, plugins, ECS
├── assets/       # Sprites and visual assets
└── index.html    # WASM entry point
```

**Core crate** — Framework-agnostic types (Agent, Artifact, Tool, Room, Message) and an event-sourced store. World state is a projection of the event stream, giving you replay and time-travel for free.

**Game crate** — Bevy 0.18 plugins that render the world. Each feature area (world grid, agents, debug overlay) is a separate Plugin.

## Building

### Prerequisites

- [Rust](https://rustup.rs/) (edition 2024)
- WASM target: `rustup target add wasm32-unknown-unknown`
- [Trunk](https://trunkrs.dev/): `cargo install trunk`

### Run in browser

```bash
trunk serve        # opens http://127.0.0.1:8080
```

### Run tests

```bash
cargo test -p agent-world-core
```

### Run native (dev)

```bash
cargo run -p agent-world-game
```

## Roadmap

- [x] Core types and event store
- [x] Bevy skeleton with grid room and agent sprites
- [ ] Adapter layer (WebSocket bridge to real agent systems)
- [ ] React shell with HUD panels
- [ ] Multiple rooms and portals
- [ ] Real sprite art
- [ ] Sound design

## License

MIT
