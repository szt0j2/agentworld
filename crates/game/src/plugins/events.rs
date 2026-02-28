use agent_world_core::{
    Agent, AgentProvider, AgentStatus, Artifact, ArtifactKind, Message,
    MessageChannel, MessageVisualStyle, Position, Room, SpriteConfig,
    SpriteShape, WorldEvent,
};
use bevy::prelude::*;
use std::collections::HashMap;

pub struct EventBridgePlugin;

/// Bevy resource: queue of pending WorldEvents for the agent system.
#[derive(Resource, Default)]
pub struct PendingEvents {
    pub queue: Vec<WorldEvent>,
}

/// Bevy resource: queue of pending visual events (thoughts, messages, tools, artifacts).
#[derive(Resource, Default)]
pub struct PendingVisualEvents {
    pub queue: Vec<WorldEvent>,
}

impl Plugin for EventBridgePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingEvents>()
            .init_resource::<PendingVisualEvents>()
            .add_systems(Startup, emit_demo_scenario)
            .add_systems(Update, cycle_demo_events);
    }
}

/// Emit the initial demo world: one room and three agents with tools and artifacts.
fn emit_demo_scenario(
    mut pending: ResMut<PendingEvents>,
    mut visual: ResMut<PendingVisualEvents>,
) {
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

    // Spawn initial artifacts
    visual.queue.push(WorldEvent::ArtifactCreate(Artifact {
        id: "spec-doc".into(),
        name: "Spec".into(),
        kind: ArtifactKind::Document,
        content_ref: String::new(),
        owner: Some("researcher".into()),
        quality: 0.5,
        position: Position { x: -80.0, y: 80.0 },
        room_id: "main".into(),
        sprite: SpriteConfig::default(),
    }));
    visual.queue.push(WorldEvent::ArtifactCreate(Artifact {
        id: "main-rs".into(),
        name: "main.rs".into(),
        kind: ArtifactKind::Code,
        content_ref: String::new(),
        owner: Some("coder".into()),
        quality: 0.3,
        position: Position { x: 70.0, y: -60.0 },
        room_id: "main".into(),
        sprite: SpriteConfig::default(),
    }));
}

/// Demo event cycle — a full "story" of agents working together.
fn cycle_demo_events(
    time: Res<Time>,
    mut timer: Local<f32>,
    mut step: Local<usize>,
    mut pending: ResMut<PendingEvents>,
    mut visual: ResMut<PendingVisualEvents>,
) {
    *timer += time.delta_secs();
    if *timer < 2.5 {
        return;
    }
    *timer = 0.0;

    let t = time.elapsed_secs();
    let bounds = 200.0;
    let agents = ["researcher", "coder", "reviewer"];

    // Cycle through a demo story
    match *step % 12 {
        0 => {
            // Researcher thinks
            pending.queue.push(WorldEvent::AgentStatusChange {
                agent_id: "researcher".into(),
                status: AgentStatus::Thinking,
                reason: None,
            });
            visual.queue.push(WorldEvent::AgentThink {
                agent_id: "researcher".into(),
                thought: "Analyzing requirements...".into(),
            });
        }
        1 => {
            // Researcher uses a tool (web search)
            pending.queue.push(WorldEvent::AgentStatusChange {
                agent_id: "researcher".into(),
                status: AgentStatus::Acting,
                reason: None,
            });
            visual.queue.push(WorldEvent::AgentUseTool {
                agent_id: "researcher".into(),
                tool_id: "web-search".into(),
                target: None,
            });
        }
        2 => {
            // Tool result + researcher sends findings to coder
            visual.queue.push(WorldEvent::AgentToolResult {
                agent_id: "researcher".into(),
                tool_id: "web-search".into(),
                success: true,
            });
            visual.queue.push(WorldEvent::MessageSend(Message {
                id: format!("msg-{}", *step),
                from: "researcher".into(),
                to: vec!["coder".into()],
                channel: MessageChannel::Direct,
                content: "Found the API docs, sending specs".into(),
                content_preview: "API docs ready".into(),
                timestamp: t as f64,
                visual_style: MessageVisualStyle::Projectile,
            }));
        }
        3 => {
            // Coder receives, starts thinking
            pending.queue.push(WorldEvent::AgentStatusChange {
                agent_id: "coder".into(),
                status: AgentStatus::Thinking,
                reason: None,
            });
            visual.queue.push(WorldEvent::AgentThink {
                agent_id: "coder".into(),
                thought: "Implementing the handler...".into(),
            });
            // Transfer spec doc to coder
            visual.queue.push(WorldEvent::AgentTransfer {
                from_id: "researcher".into(),
                to_id: "coder".into(),
                artifact_id: "spec-doc".into(),
            });
        }
        4 => {
            // Coder uses file tool
            pending.queue.push(WorldEvent::AgentStatusChange {
                agent_id: "coder".into(),
                status: AgentStatus::Acting,
                reason: None,
            });
            visual.queue.push(WorldEvent::AgentUseTool {
                agent_id: "coder".into(),
                tool_id: "file-write".into(),
                target: Some("main.rs".into()),
            });
        }
        5 => {
            // Coder finishes, sends code to reviewer
            visual.queue.push(WorldEvent::AgentToolResult {
                agent_id: "coder".into(),
                tool_id: "file-write".into(),
                success: true,
            });
            visual.queue.push(WorldEvent::MessageSend(Message {
                id: format!("msg-{}", *step),
                from: "coder".into(),
                to: vec!["reviewer".into()],
                channel: MessageChannel::Direct,
                content: "Code ready for review".into(),
                content_preview: "PR ready".into(),
                timestamp: t as f64,
                visual_style: MessageVisualStyle::Projectile,
            }));
            // Transfer code to reviewer
            visual.queue.push(WorldEvent::AgentTransfer {
                from_id: "coder".into(),
                to_id: "reviewer".into(),
                artifact_id: "main-rs".into(),
            });
        }
        6 => {
            // Reviewer thinks about the code
            pending.queue.push(WorldEvent::AgentStatusChange {
                agent_id: "reviewer".into(),
                status: AgentStatus::Thinking,
                reason: None,
            });
            visual.queue.push(WorldEvent::AgentThink {
                agent_id: "reviewer".into(),
                thought: "Checking for edge cases...".into(),
            });
            // Researcher goes idle
            pending.queue.push(WorldEvent::AgentStatusChange {
                agent_id: "researcher".into(),
                status: AgentStatus::Idle,
                reason: None,
            });
        }
        7 => {
            // Reviewer uses analysis tool
            visual.queue.push(WorldEvent::AgentUseTool {
                agent_id: "reviewer".into(),
                tool_id: "code-analysis".into(),
                target: Some("main.rs".into()),
            });
            pending.queue.push(WorldEvent::AgentStatusChange {
                agent_id: "reviewer".into(),
                status: AgentStatus::Acting,
                reason: None,
            });
        }
        8 => {
            // Reviewer finds an issue — sends feedback
            visual.queue.push(WorldEvent::AgentToolResult {
                agent_id: "reviewer".into(),
                tool_id: "code-analysis".into(),
                success: false,
            });
            visual.queue.push(WorldEvent::MessageSend(Message {
                id: format!("msg-{}", *step),
                from: "reviewer".into(),
                to: vec!["coder".into()],
                channel: MessageChannel::Direct,
                content: "Missing error handling on line 42".into(),
                content_preview: "Bug found L42".into(),
                timestamp: t as f64,
                visual_style: MessageVisualStyle::Projectile,
            }));
        }
        9 => {
            // Coder fixes
            pending.queue.push(WorldEvent::AgentStatusChange {
                agent_id: "coder".into(),
                status: AgentStatus::Thinking,
                reason: None,
            });
            visual.queue.push(WorldEvent::AgentThink {
                agent_id: "coder".into(),
                thought: "Fixing the error handler...".into(),
            });
            pending.queue.push(WorldEvent::AgentStatusChange {
                agent_id: "reviewer".into(),
                status: AgentStatus::Waiting,
                reason: None,
            });
        }
        10 => {
            // Broadcast: coder announces fix
            visual.queue.push(WorldEvent::AgentUseTool {
                agent_id: "coder".into(),
                tool_id: "file-write".into(),
                target: Some("main.rs".into()),
            });
            visual.queue.push(WorldEvent::AgentToolResult {
                agent_id: "coder".into(),
                tool_id: "file-write".into(),
                success: true,
            });
            visual.queue.push(WorldEvent::MessageSend(Message {
                id: format!("msg-{}", *step),
                from: "coder".into(),
                to: vec!["researcher".into(), "reviewer".into()],
                channel: MessageChannel::Broadcast,
                content: "Fix applied and tests pass".into(),
                content_preview: "Fix done!".into(),
                timestamp: t as f64,
                visual_style: MessageVisualStyle::Ripple,
            }));
        }
        11 => {
            // Everyone moves to new positions + cycle resets
            for (i, agent_id) in agents.iter().enumerate() {
                let seed = t + i as f32 * 137.5;
                let x = (seed.sin() * 1000.0).fract() * bounds * 2.0 - bounds;
                let y = (seed.cos() * 1000.0).fract() * bounds * 2.0 - bounds;

                pending.queue.push(WorldEvent::AgentMove {
                    agent_id: agent_id.to_string(),
                    to: Position { x, y },
                });
                pending.queue.push(WorldEvent::AgentStatusChange {
                    agent_id: agent_id.to_string(),
                    status: AgentStatus::Idle,
                    reason: None,
                });
            }
        }
        _ => {}
    }

    *step += 1;
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
        sprite: SpriteConfig {
            color,
            shape: SpriteShape::Square,
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
