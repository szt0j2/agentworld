use agent_world_core::{
    Agent, AgentProvider, AgentStatus, Artifact, ArtifactKind, Message,
    MessageChannel, MessageVisualStyle, Position, Room, SpriteConfig,
    SpriteShape, WorldEvent,
};
use bevy::prelude::*;
use crate::components::AgentSprite;
use crate::plugins::hud::EventLog;
use crate::plugins::adapter::{AdapterConfig, ConnectionStatus};
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
            .init_resource::<JsSyncTimer>()
            .add_systems(Update, (cycle_demo_events, log_visual_events, forward_events_to_js, sync_agents_to_js).chain());
    }
}

/// Emit the initial demo world: three rooms, five agents, artifacts.
/// Skipped when WebSocket adapter is enabled (real data).
fn emit_demo_scenario(
    config: Res<AdapterConfig>,
    mut pending: ResMut<PendingEvents>,
    mut visual: ResMut<PendingVisualEvents>,
) {
    if config.enabled {
        return; // Skip demo when real data is coming via WebSocket
    }

    use agent_world_core::Portal;

    // Room 1: Workspace (left)
    pending.queue.push(WorldEvent::RoomCreate(Room {
        id: "workspace".into(),
        name: "Workspace".into(),
        width: 480.0,
        height: 480.0,
        purpose: "workspace".into(),
        portals: vec![Portal {
            id: "p-ws-review".into(),
            target_room: "Review".into(),
            position: Position { x: 220.0, y: 0.0 },
            target_position: Position { x: -220.0, y: 0.0 },
        }],
    }));

    // Room 2: Review (center)
    pending.queue.push(WorldEvent::RoomCreate(Room {
        id: "review".into(),
        name: "Review".into(),
        width: 480.0,
        height: 480.0,
        purpose: "review".into(),
        portals: vec![
            Portal {
                id: "p-review-ws".into(),
                target_room: "Workspace".into(),
                position: Position { x: -220.0, y: 0.0 },
                target_position: Position { x: 220.0, y: 0.0 },
            },
            Portal {
                id: "p-review-deploy".into(),
                target_room: "Deploy".into(),
                position: Position { x: 220.0, y: 0.0 },
                target_position: Position { x: -220.0, y: 0.0 },
            },
        ],
    }));

    // Room 3: Deploy (right)
    pending.queue.push(WorldEvent::RoomCreate(Room {
        id: "deploy".into(),
        name: "Deploy".into(),
        width: 480.0,
        height: 480.0,
        purpose: "deploy".into(),
        portals: vec![Portal {
            id: "p-deploy-review".into(),
            target_room: "Review".into(),
            position: Position { x: -220.0, y: 0.0 },
            target_position: Position { x: 220.0, y: 0.0 },
        }],
    }));

    // 5 Agents across rooms
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
        Position { x: 700.0 + 0.0, y: 50.0 },
    )));
    pending.queue.push(WorldEvent::AgentSpawn(make_agent(
        "tester", "Tester", "tester",
        [255, 102, 178, 255],
        Position { x: 700.0 - 80.0, y: -80.0 },
    )));
    pending.queue.push(WorldEvent::AgentSpawn(make_agent(
        "deployer", "Deployer", "deployer",
        [178, 102, 255, 255],
        Position { x: 1400.0 + 0.0, y: 0.0 },
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
        room_id: "workspace".into(),
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
        room_id: "workspace".into(),
        sprite: SpriteConfig::default(),
    }));
    visual.queue.push(WorldEvent::ArtifactCreate(Artifact {
        id: "deploy-script".into(),
        name: "deploy.sh".into(),
        kind: ArtifactKind::Code,
        content_ref: String::new(),
        owner: Some("deployer".into()),
        quality: 0.7,
        position: Position { x: 1400.0 + 20.0, y: 0.0 },
        room_id: "deploy".into(),
        sprite: SpriteConfig::default(),
    }));
}

/// Demo event cycle — a full "story" across three rooms with five agents.
fn cycle_demo_events(
    config: Res<AdapterConfig>,
    time: Res<Time>,
    mut timer: Local<f32>,
    mut step: Local<usize>,
    mut pending: ResMut<PendingEvents>,
    mut visual: ResMut<PendingVisualEvents>,
) {
    if config.enabled {
        return;
    }
    *timer += time.delta_secs();
    if *timer < 2.0 {
        return;
    }
    *timer = 0.0;

    let t = time.elapsed_secs();

    // Fixed positions — agents have "desks" in their rooms, move toward
    // collaborators when interacting, then return to their desk.
    // Workspace room at x=0, Review room at x=700, Deploy room at x=1400

    // Home positions (where agents sit when idle)
    let home = |agent: &str| -> Position {
        match agent {
            "researcher" => Position { x: -100.0, y: 80.0 },
            "coder"      => Position { x: 50.0, y: -60.0 },
            "reviewer"   => Position { x: 640.0, y: 50.0 },
            "tester"     => Position { x: 760.0, y: -80.0 },
            "deployer"   => Position { x: 1400.0, y: 0.0 },
            _ => Position { x: 0.0, y: 0.0 },
        }
    };

    // Move toward another agent (approach their position)
    let approach = |pending: &mut ResMut<PendingEvents>, who: &str, toward: &str| {
        let target = home(toward);
        let origin = home(who);
        // Move 70% of the way toward the target
        let x = origin.x + (target.x - origin.x) * 0.7;
        let y = origin.y + (target.y - origin.y) * 0.7;
        pending.queue.push(WorldEvent::AgentMove {
            agent_id: who.into(),
            to: Position { x, y },
        });
    };

    let go_home = |pending: &mut ResMut<PendingEvents>, who: &str| {
        let pos = home(who);
        pending.queue.push(WorldEvent::AgentMove {
            agent_id: who.into(),
            to: pos,
        });
    };

    match *step % 16 {
        0 => {
            // Researcher starts analyzing at their desk
            set_status(&mut pending, "researcher", AgentStatus::Thinking);
            think(&mut visual, "researcher", "Analyzing requirements...");
        }
        1 => {
            // Researcher uses web search (stays at desk)
            set_status(&mut pending, "researcher", AgentStatus::Acting);
            use_tool(&mut visual, "researcher", "web-search");
        }
        2 => {
            // Researcher moves toward Coder to hand off findings
            approach(&mut pending, "researcher", "coder");
            tool_result(&mut visual, "researcher", "web-search", true);
            send_msg(&mut visual, "researcher", &["coder"], "API docs ready", t);
            transfer(&mut visual, "researcher", "coder", "spec-doc");
        }
        3 => {
            // Researcher returns home, Coder starts working
            go_home(&mut pending, "researcher");
            set_status(&mut pending, "researcher", AgentStatus::Idle);
            set_status(&mut pending, "coder", AgentStatus::Thinking);
            think(&mut visual, "coder", "Implementing the handler...");
        }
        4 => {
            // Coder writes code at their desk
            set_status(&mut pending, "coder", AgentStatus::Acting);
            use_tool(&mut visual, "coder", "file-write");
        }
        5 => {
            // Coder warps through portal to Review room to deliver PR
            tool_result(&mut visual, "coder", "file-write", true);
            send_msg(&mut visual, "coder", &["reviewer"], "PR ready", t);
            transfer(&mut visual, "coder", "reviewer", "main-rs");
            pending.queue.push(WorldEvent::RoomEnter {
                agent_id: "coder".into(),
                room_id: "review".into(),
            });
        }
        6 => {
            // Coder returns home through portal, Reviewer + Tester start examining
            pending.queue.push(WorldEvent::RoomEnter {
                agent_id: "coder".into(),
                room_id: "workspace".into(),
            });
            set_status(&mut pending, "coder", AgentStatus::Idle);
            set_status(&mut pending, "reviewer", AgentStatus::Thinking);
            think(&mut visual, "reviewer", "Checking edge cases...");
            // Tester moves closer to Reviewer to coordinate
            approach(&mut pending, "tester", "reviewer");
            set_status(&mut pending, "tester", AgentStatus::Thinking);
            think(&mut visual, "tester", "Preparing test suite...");
        }
        7 => {
            // Reviewer + Tester both working (at their positions)
            set_status(&mut pending, "reviewer", AgentStatus::Acting);
            use_tool(&mut visual, "reviewer", "code-analysis");
            set_status(&mut pending, "tester", AgentStatus::Acting);
            use_tool(&mut visual, "tester", "test-runner");
        }
        8 => {
            // Bug found! Reviewer + Tester move toward Coder to report
            approach(&mut pending, "reviewer", "coder");
            approach(&mut pending, "tester", "coder");
            tool_result(&mut visual, "reviewer", "code-analysis", false);
            tool_result(&mut visual, "tester", "test-runner", false);
            send_msg(&mut visual, "reviewer", &["coder"], "Bug found L42", t);
            send_msg(&mut visual, "tester", &["coder"], "Test failed!", t);
        }
        9 => {
            // Reviewer + Tester return to their area and wait
            go_home(&mut pending, "reviewer");
            go_home(&mut pending, "tester");
            set_status(&mut pending, "reviewer", AgentStatus::Waiting);
            set_status(&mut pending, "tester", AgentStatus::Waiting);
            // Coder starts fixing
            set_status(&mut pending, "coder", AgentStatus::Thinking);
            think(&mut visual, "coder", "Fixing error handler...");
        }
        10 => {
            // Coder applies fix at their desk
            set_status(&mut pending, "coder", AgentStatus::Acting);
            use_tool(&mut visual, "coder", "file-write");
            tool_result(&mut visual, "coder", "file-write", true);
        }
        11 => {
            // Tester re-runs and announces to everyone
            set_status(&mut pending, "tester", AgentStatus::Acting);
            use_tool(&mut visual, "tester", "test-runner");
            tool_result(&mut visual, "tester", "test-runner", true);
            send_msg(&mut visual, "tester", &["reviewer", "coder"], "All tests pass!", t);
        }
        12 => {
            // Reviewer approves, warps through portal to Deploy room
            set_status(&mut pending, "reviewer", AgentStatus::Acting);
            think(&mut visual, "reviewer", "LGTM, approved!");
            send_msg(&mut visual, "reviewer", &["deployer"], "Deploy approved", t);
            transfer(&mut visual, "reviewer", "deployer", "main-rs");
            pending.queue.push(WorldEvent::RoomEnter {
                agent_id: "reviewer".into(),
                room_id: "deploy".into(),
            });
        }
        13 => {
            // Reviewer returns home through portal, Tester idles, Deployer starts
            pending.queue.push(WorldEvent::RoomEnter {
                agent_id: "reviewer".into(),
                room_id: "review".into(),
            });
            set_status(&mut pending, "reviewer", AgentStatus::Idle);
            go_home(&mut pending, "tester");
            set_status(&mut pending, "tester", AgentStatus::Idle);
            set_status(&mut pending, "deployer", AgentStatus::Thinking);
            think(&mut visual, "deployer", "Preparing deployment...");
        }
        14 => {
            // Deployer runs deploy and broadcasts success
            set_status(&mut pending, "deployer", AgentStatus::Acting);
            use_tool(&mut visual, "deployer", "deploy-script");
            tool_result(&mut visual, "deployer", "deploy-script", true);
            send_msg(&mut visual, "deployer", &["researcher", "coder", "reviewer", "tester"], "Deployed!", t);
        }
        15 => {
            // Everyone returns home and idles, cycle complete
            for agent_id in &["researcher", "coder", "reviewer", "tester", "deployer"] {
                go_home(&mut pending, agent_id);
                set_status(&mut pending, agent_id, AgentStatus::Idle);
            }
        }
        _ => {}
    }

    *step += 1;
}

// Helper functions for demo events
fn set_status(pending: &mut ResMut<PendingEvents>, agent: &str, status: AgentStatus) {
    pending.queue.push(WorldEvent::AgentStatusChange {
        agent_id: agent.into(),
        status,
        reason: None,
    });
}

fn think(visual: &mut ResMut<PendingVisualEvents>, agent: &str, thought: &str) {
    visual.queue.push(WorldEvent::AgentThink {
        agent_id: agent.into(),
        thought: thought.into(),
    });
}

fn use_tool(visual: &mut ResMut<PendingVisualEvents>, agent: &str, tool: &str) {
    visual.queue.push(WorldEvent::AgentUseTool {
        agent_id: agent.into(),
        tool_id: tool.into(),
        target: None,
    });
}

fn tool_result(visual: &mut ResMut<PendingVisualEvents>, agent: &str, tool: &str, success: bool) {
    visual.queue.push(WorldEvent::AgentToolResult {
        agent_id: agent.into(),
        tool_id: tool.into(),
        success,
    });
}

fn send_msg(visual: &mut ResMut<PendingVisualEvents>, from: &str, to: &[&str], preview: &str, t: f32) {
    visual.queue.push(WorldEvent::MessageSend(Message {
        id: format!("msg-{}-{}", from, t as u32),
        from: from.into(),
        to: to.iter().map(|s| s.to_string()).collect(),
        channel: if to.len() > 1 { MessageChannel::Broadcast } else { MessageChannel::Direct },
        content: preview.into(),
        content_preview: preview.into(),
        timestamp: t as f64,
        visual_style: MessageVisualStyle::Projectile,
    }));
}

fn transfer(visual: &mut ResMut<PendingVisualEvents>, from: &str, to: &str, artifact: &str) {
    visual.queue.push(WorldEvent::AgentTransfer {
        from_id: from.into(),
        to_id: to.into(),
        artifact_id: artifact.into(),
    });
}

/// Log visual events to the HUD event log.
/// Shorten agent IDs for log display (e.g., "default/lead" → "lead")
fn short_id(id: &str) -> &str {
    id.rsplit('/').next().unwrap_or(id)
}

/// Clean up tool names for display
fn clean_tool(name: &str) -> String {
    name.replace("mcp__playwright__", "pw:")
        .replace("mcp__opnsense__", "opn:")
        .replace("mcp__local__", "local:")
        .replace("mcp__winrm__", "win:")
}

fn log_visual_events(
    pending: Res<PendingEvents>,
    visual: Res<PendingVisualEvents>,
    mut log: ResMut<EventLog>,
) {
    for event in &pending.queue {
        match event {
            WorldEvent::AgentSpawn(agent) => {
                log.push(format!("+ {} joined", short_id(&agent.id)));
            }
            WorldEvent::AgentDespawn { agent_id } => {
                log.push(format!("- {} left", short_id(agent_id)));
            }
            WorldEvent::AgentStatusChange { agent_id, status, .. } => {
                let status_str = match status {
                    AgentStatus::Idle => "idle",
                    AgentStatus::Thinking => "thinking",
                    AgentStatus::Acting => "acting",
                    AgentStatus::Waiting => "waiting",
                    AgentStatus::Error => "ERROR",
                    AgentStatus::Paused => "paused",
                };
                log.push(format!("{} → {status_str}", short_id(agent_id)));
            }
            WorldEvent::AgentError { agent_id, error } => {
                let short_err = if error.len() > 25 { &error[..25] } else { error };
                log.push(format!("! {} ERR: {short_err}", short_id(agent_id)));
            }
            WorldEvent::RoomEnter { agent_id, room_id } => {
                log.push(format!("{} >> {}", short_id(agent_id), room_id));
            }
            WorldEvent::RoomExit { agent_id, room_id } => {
                log.push(format!("{} << {}", short_id(agent_id), room_id));
            }
            _ => {}
        }
    }
    for event in &visual.queue {
        match event {
            WorldEvent::AgentThink { agent_id, thought } => {
                let cleaned = clean_tool(thought);
                let short = if cleaned.len() > 30 { &cleaned[..30] } else { &cleaned };
                log.push(format!("{}: {short}", short_id(agent_id)));
            }
            WorldEvent::AgentUseTool { agent_id, tool_id, .. } => {
                log.push(format!("{} > {}", short_id(agent_id), clean_tool(tool_id)));
            }
            WorldEvent::AgentToolResult { agent_id, success, .. } => {
                let icon = if *success { "ok" } else { "FAIL" };
                log.push(format!("{} < {icon}", short_id(agent_id)));
            }
            WorldEvent::MessageSend(msg) => {
                let to = msg.to.iter().map(|t| short_id(t)).collect::<Vec<_>>().join(",");
                log.push(format!("{} msg> {to}", short_id(&msg.from)));
            }
            WorldEvent::ArtifactCreate(art) => {
                log.push(format!("+ {}", art.name));
            }
            _ => {}
        }
    }
}

/// Forward events to the React HUD shell via window.__agentworld_event(json).
fn forward_events_to_js(
    pending: Res<PendingEvents>,
    visual: Res<PendingVisualEvents>,
) {
    #[cfg(target_arch = "wasm32")]
    {
        use serde_json;
        for event in pending.queue.iter().chain(visual.queue.iter()) {
            if let Ok(json) = serde_json::to_string(event) {
                forward_one_event(&json);
            }
        }
    }

    // Suppress unused warnings on non-WASM
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (&pending, &visual);
    }
}

#[cfg(target_arch = "wasm32")]
fn forward_one_event(json: &str) {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = window, js_name = "__agentworld_event")]
        fn js_event_callback(json: &str);
    }

    // Only call if the function exists (React shell loaded)
    let window = web_sys::window().unwrap();
    let has_fn = js_sys::Reflect::get(&window, &JsValue::from_str("__agentworld_event"))
        .map(|v| v.is_function())
        .unwrap_or(false);
    if has_fn {
        js_event_callback(json);
    }
}

/// Timer for periodic agent state sync to JS.
#[derive(Resource)]
struct JsSyncTimer {
    timer: Timer,
}

impl Default for JsSyncTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.5, TimerMode::Repeating),
        }
    }
}

/// Periodically sync the full agent roster to the React HUD.
/// This ensures agents are visible even if AgentSpawn events were missed.
fn sync_agents_to_js(
    agents: Query<&AgentSprite>,
    artifact_query: Query<&crate::components::ArtifactSprite>,
    conn_status: Res<ConnectionStatus>,
    time: Res<Time>,
    mut sync_timer: ResMut<JsSyncTimer>,
) {
    sync_timer.timer.tick(time.delta());
    if !sync_timer.timer.just_finished() {
        return;
    }

    #[cfg(target_arch = "wasm32")]
    {
        let agent_list: Vec<serde_json::Value> = agents.iter().map(|a| {
            serde_json::json!({
                "id": a.agent_id,
                "name": a.name,
                "role": a.role,
                "status": format!("{:?}", a.status),
                "toolCount": a.tool_count,
                "lastTool": a.last_tool,
                "thought": a.last_thought,
            })
        }).collect();

        let artifact_list: Vec<serde_json::Value> = artifact_query.iter().map(|a| {
            serde_json::json!({
                "id": a.artifact_id,
                "name": a.name,
                "kind": format!("{:?}", a.kind),
                "owner": a.owner,
                "quality": a.quality,
            })
        }).collect();

        let conn = match *conn_status {
            ConnectionStatus::Disconnected => "disconnected",
            ConnectionStatus::Connecting => "connecting",
            ConnectionStatus::Connected => "connected",
            ConnectionStatus::Reconnecting => "reconnecting",
        };

        let payload = serde_json::json!({
            "agents": agent_list,
            "artifacts": artifact_list,
            "connection": conn,
        });

        if let Ok(json) = serde_json::to_string(&payload) {
            call_js_sync(&json);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (&agents, &artifact_query, &conn_status);
    }
}

#[cfg(target_arch = "wasm32")]
fn call_js_sync(json: &str) {
    use wasm_bindgen::prelude::*;

    let window = web_sys::window().unwrap();
    let has_fn = js_sys::Reflect::get(&window, &JsValue::from_str("__agentworld_sync"))
        .map(|v| v.is_function())
        .unwrap_or(false);
    if has_fn {
        let func = js_sys::Reflect::get(&window, &JsValue::from_str("__agentworld_sync")).unwrap();
        let func: js_sys::Function = func.into();
        let _ = func.call1(&JsValue::NULL, &JsValue::from_str(json));
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
