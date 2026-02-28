use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 2D position within a room.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

/// Visual configuration for an entity's sprite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteConfig {
    pub color: [u8; 4], // RGBA
    pub shape: SpriteShape,
    pub scale: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpriteShape {
    Square,
    Circle,
    Diamond,
    Triangle,
}

impl Default for SpriteConfig {
    fn default() -> Self {
        Self {
            color: [100, 149, 237, 255], // cornflower blue
            shape: SpriteShape::Square,
            scale: 1.0,
        }
    }
}

/// Agent status — maps to visual treatment in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    Idle,
    Thinking,
    Acting,
    Waiting,
    Error,
    Paused,
}

/// Provider of an agent (what system runs it).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentProvider {
    Claude,
    Gpt,
    Local,
    Custom(String),
}

/// A task an agent is working on.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskState {
    pub id: String,
    pub description: String,
    pub progress: f32, // 0.0..=1.0
    pub assigned_by: Option<String>,
}

/// An autonomous agent — the "player character" of the world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub role: String,
    pub provider: AgentProvider,
    pub status: AgentStatus,
    pub position: Position,
    pub room_id: String,
    pub sprite: SpriteConfig,
    pub equipped_tools: Vec<String>,
    pub inventory: Vec<String>,
    pub current_task: Option<TaskState>,
    pub health: f32,
    pub energy: f32,
    pub thought: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// The kind of artifact (work product).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactKind {
    Document,
    Code,
    Data,
    Image,
    Plan,
    MessageBundle,
}

/// An artifact — a visible work product that agents carry and exchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: String,
    pub name: String,
    pub kind: ArtifactKind,
    pub content_ref: String,
    pub owner: Option<String>,
    pub quality: f32,
    pub position: Position,
    pub room_id: String,
    pub sprite: SpriteConfig,
}

/// The kind of tool (capability).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolKind {
    Mcp,
    Api,
    Shell,
    Browser,
    File,
    Custom(String),
}

/// A tool — a capability agents can invoke, visualized as a weapon/spell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub id: String,
    pub name: String,
    pub description: String,
    pub kind: ToolKind,
    pub provider: String,
    pub cooldown: f32,
    pub power: f32,
    pub equipped_by: Vec<String>,
}

/// A room — a spatial container for entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: String,
    pub name: String,
    pub width: f32,
    pub height: f32,
    pub purpose: String,
    pub portals: Vec<Portal>,
}

/// A portal connecting two rooms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portal {
    pub id: String,
    pub target_room: String,
    pub position: Position,
    pub target_position: Position,
}

/// Visual style for messages traveling between entities.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MessageVisualStyle {
    Projectile,
    Bubble,
    Beam,
    Ripple,
    Scroll,
}

/// Channel a message is sent on.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageChannel {
    Direct,
    Broadcast,
    ToolCall,
    ToolResult,
    Human,
}

/// A message between entities — visible as a projectile/effect.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub from: String,
    pub to: Vec<String>,
    pub channel: MessageChannel,
    pub content: String,
    pub content_preview: String,
    pub timestamp: f64,
    pub visual_style: MessageVisualStyle,
}
