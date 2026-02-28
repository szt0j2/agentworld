use agent_world_core::{AgentStatus, ArtifactKind};
use bevy::prelude::*;

/// Marks an entity as an agent sprite in the game world.
#[derive(Component)]
pub struct AgentSprite {
    pub agent_id: String,
    pub name: String,
    pub role: String,
    pub status: AgentStatus,
}

/// Marks an entity as an artifact sprite.
#[derive(Component)]
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

/// A thought bubble floating above an agent.
#[derive(Component)]
pub struct ThoughtBubble {
    pub lifetime: f32,
    pub max_lifetime: f32,
}

/// A message projectile traveling between agents.
#[derive(Component)]
pub struct MessageProjectile {
    pub from_pos: Vec2,
    pub to_agent_id: String,
    pub progress: f32,
    pub speed: f32,
    pub content_preview: String,
}

/// Tool use effect flash on an agent.
#[derive(Component)]
pub struct ToolEffect {
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub success: Option<bool>,
}
