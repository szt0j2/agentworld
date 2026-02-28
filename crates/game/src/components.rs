use bevy::prelude::*;

/// Marks an entity as an agent sprite in the game world.
#[derive(Component)]
pub struct AgentSprite {
    pub agent_id: String,
    pub name: String,
    pub role: String,
}

/// Marks an entity as an artifact sprite.
#[derive(Component)]
pub struct ArtifactSprite {
    pub artifact_id: String,
}

/// Simple movement target for agent wandering.
#[derive(Component)]
pub struct MovementTarget {
    pub target: Vec2,
    pub speed: f32,
}

/// The text label floating above an agent.
#[derive(Component)]
pub struct AgentLabel;

/// Grid cell marker for the room floor.
#[derive(Component)]
pub struct GridCell;
