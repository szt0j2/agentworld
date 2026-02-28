use agent_world_core::AgentStatus;
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
