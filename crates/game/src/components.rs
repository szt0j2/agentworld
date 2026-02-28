use agent_world_core::{AgentStatus, ArtifactKind};
use bevy::prelude::*;

/// Marks an entity as an agent sprite in the game world.
#[derive(Component)]
pub struct AgentSprite {
    pub agent_id: String,
    pub name: String,
    pub role: String,
    pub status: AgentStatus,
    pub last_tool: Option<String>,
    pub last_thought: Option<String>,
    pub tool_count: u32,
}

/// Marks an entity as an artifact sprite.
#[derive(Component)]
#[allow(dead_code)]
pub struct ArtifactSprite {
    pub artifact_id: String,
    pub name: String,
    pub kind: ArtifactKind,
    pub owner: Option<String>,
    pub quality: f32,
}

/// Smooth movement toward a target position.
#[derive(Component)]
pub struct MovementTarget {
    pub target: Vec2,
    pub speed: f32,
}

/// The text label floating above an agent.
#[derive(Component)]
pub struct AgentLabel;

/// Status indicator ring around an agent.
#[derive(Component)]
pub struct StatusRing {
    pub base_scale: f32,
}

/// Grid cell marker for the room floor.
#[derive(Component)]
pub struct GridCell;

/// A portal connecting rooms.
#[derive(Component)]
#[allow(dead_code)]
pub struct PortalSprite {
    pub portal_id: String,
    pub target_room: String,
}

/// Health bar background (behind the fill).
#[derive(Component)]
pub struct HealthBar;

/// Energy bar background.
#[derive(Component)]
pub struct EnergyBar;

/// A thought bubble floating above an agent.
#[derive(Component)]
pub struct ThoughtBubble {
    pub agent_id: String,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

/// A message projectile traveling between agents.
#[derive(Component)]
#[allow(dead_code)]
pub struct MessageProjectile {
    pub from_pos: Vec2,
    pub to_agent_id: String,
    pub progress: f32,
    pub speed: f32,
    pub content_preview: String,
}

/// Tool use effect flash on an agent.
#[derive(Component)]
#[allow(dead_code)]
pub struct ToolEffect {
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub success: Option<bool>,
}

/// A fading trail dot left behind by a moving agent.
#[derive(Component)]
pub struct TrailDot {
    pub lifetime: f32,
    pub max_lifetime: f32,
}

/// A fading connection line between two agents (shown after messages).
#[derive(Component)]
#[allow(dead_code)]
pub struct ConnectionLine {
    pub from_agent: String,
    pub to_agent: String,
    pub lifetime: f32,
    pub max_lifetime: f32,
}
