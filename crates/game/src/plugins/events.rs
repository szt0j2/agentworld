use agent_world_core::{
    Agent, AgentProvider, AgentStatus, Position, Room, WorldEvent,
};
use bevy::prelude::*;
use std::collections::HashMap;

pub struct EventBridgePlugin;

/// Bevy resource: queue of pending WorldEvents for the game to process.
#[derive(Resource, Default)]
pub struct PendingEvents {
    pub queue: Vec<WorldEvent>,
}

impl Plugin for EventBridgePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingEvents>()
            .add_systems(Startup, emit_demo_scenario)
            .add_systems(Update, cycle_demo_events);
    }
}

/// Emit the initial demo world: one room and three agents.
fn emit_demo_scenario(mut pending: ResMut<PendingEvents>) {
    pending.queue.push(WorldEvent::RoomCreate(Room {
        id: "main".into(),
        name: "Main Hall".into(),
        width: 576.0,
        height: 576.0,
        purpose: "workspace".into(),
        portals: vec![],
    }));

    pending.queue.push(WorldEvent::AgentSpawn(make_agent(
        "researcher", "Researcher", "researcher",
        [51, 153, 255, 255],
        Position { x: -100.0, y: 80.0 },
    )));
    pending.queue.push(WorldEvent::AgentSpawn(make_agent(
        "coder", "Coder", "coder",
        [51, 230, 102, 255],
        Position { x: 50.0, y: -60.0 },
    )));
    pending.queue.push(WorldEvent::AgentSpawn(make_agent(
        "reviewer", "Reviewer", "reviewer",
        [230, 128, 51, 255],
        Position { x: 120.0, y: 100.0 },
    )));
}

/// Periodically emit movement and status change events.
fn cycle_demo_events(
    time: Res<Time>,
    mut timer: Local<f32>,
    mut pending: ResMut<PendingEvents>,
) {
    *timer += time.delta_secs();
    if *timer < 2.0 {
        return;
    }
    *timer = 0.0;

    let t = time.elapsed_secs();
    let bounds = 200.0;

    let agents = ["researcher", "coder", "reviewer"];
    let statuses = [
        AgentStatus::Idle,
        AgentStatus::Thinking,
        AgentStatus::Acting,
        AgentStatus::Waiting,
    ];

    for (i, agent_id) in agents.iter().enumerate() {
        let seed = t + i as f32 * 137.5;
        let x = (seed.sin() * 1000.0).fract() * bounds * 2.0 - bounds;
        let y = (seed.cos() * 1000.0).fract() * bounds * 2.0 - bounds;

        pending.queue.push(WorldEvent::AgentMove {
            agent_id: agent_id.to_string(),
            to: Position { x, y },
        });

        let status_idx = ((t / 3.0) as usize + i) % statuses.len();
        pending.queue.push(WorldEvent::AgentStatusChange {
            agent_id: agent_id.to_string(),
            status: statuses[status_idx],
            reason: None,
        });
    }
}

fn make_agent(
    id: &str, name: &str, role: &str,
    color: [u8; 4],
    position: Position,
) -> Agent {
    Agent {
        id: id.into(),
        name: name.into(),
        role: role.into(),
        provider: AgentProvider::Claude,
        status: AgentStatus::Idle,
        position,
        room_id: "main".into(),
        sprite: agent_world_core::SpriteConfig {
            color,
            shape: agent_world_core::SpriteShape::Square,
            scale: 1.0,
        },
        equipped_tools: vec![],
        inventory: vec![],
        current_task: None,
        health: 100.0,
        energy: 100.0,
        thought: None,
        metadata: HashMap::new(),
    }
}
