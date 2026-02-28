use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Every state change in AgentWorld flows through a WorldEvent.
/// The world state is a projection of the event stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorldEvent {
    // Agent lifecycle
    AgentSpawn(Agent),
    AgentDespawn { agent_id: String },
    AgentMove { agent_id: String, to: Position },
    AgentStatusChange { agent_id: String, status: AgentStatus, reason: Option<String> },
    AgentThink { agent_id: String, thought: String },
    AgentEquipTool { agent_id: String, tool_id: String },

    // Tool use
    AgentUseTool { agent_id: String, tool_id: String, target: Option<String> },
    AgentToolResult { agent_id: String, tool_id: String, success: bool },

    // Artifact interactions
    ArtifactCreate(Artifact),
    AgentPickUp { agent_id: String, artifact_id: String },
    AgentDrop { agent_id: String, artifact_id: String, position: Position },
    AgentTransfer { from_id: String, to_id: String, artifact_id: String },
    ArtifactQualityChange { artifact_id: String, quality: f32 },

    // Messages
    MessageSend(Message),

    // Rooms
    RoomCreate(Room),
    RoomEnter { agent_id: String, room_id: String },
    RoomExit { agent_id: String, room_id: String },

    // Human interaction
    HumanCommand { target_id: String, command: String },

    // Errors
    AgentError { agent_id: String, error: String },
}

/// A timestamped event entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEntry {
    pub index: usize,
    pub timestamp: f64,
    pub event: WorldEvent,
}

type Listener = Box<dyn Fn(&EventEntry) + Send>;

/// Event-sourced store. All world state is derived from replaying events.
pub struct EventStore {
    events: Vec<EventEntry>,
    listeners: HashMap<String, Vec<Listener>>,
    wildcard_listeners: Vec<Listener>,
}

impl EventStore {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            listeners: HashMap::new(),
            wildcard_listeners: Vec::new(),
        }
    }

    /// Emit an event, appending it to the log and notifying subscribers.
    pub fn emit(&mut self, event: WorldEvent) -> usize {
        let index = self.events.len();
        let entry = EventEntry {
            index,
            timestamp: Self::now(),
            event,
        };

        // Notify type-specific listeners
        let key = entry.event.event_type();
        if let Some(listeners) = self.listeners.get(&key) {
            for listener in listeners {
                listener(&entry);
            }
        }

        // Notify wildcard listeners
        for listener in &self.wildcard_listeners {
            listener(&entry);
        }

        self.events.push(entry);
        index
    }

    /// Subscribe to a specific event type by name (e.g. "AgentSpawn").
    pub fn subscribe(&mut self, event_type: &str, handler: impl Fn(&EventEntry) + Send + 'static) {
        self.listeners
            .entry(event_type.to_string())
            .or_default()
            .push(Box::new(handler));
    }

    /// Subscribe to all events.
    pub fn subscribe_all(&mut self, handler: impl Fn(&EventEntry) + Send + 'static) {
        self.wildcard_listeners.push(Box::new(handler));
    }

    /// Replay events in a range.
    pub fn replay(&self, from: usize, to: Option<usize>) -> &[EventEntry] {
        let end = to.unwrap_or(self.events.len()).min(self.events.len());
        let start = from.min(end);
        &self.events[start..end]
    }

    /// Get all events.
    pub fn all_events(&self) -> &[EventEntry] {
        &self.events
    }

    /// Number of events stored.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Serialize all events to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.events)
    }

    /// Deserialize events from JSON (for loading saved state).
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let events: Vec<EventEntry> = serde_json::from_str(json)?;
        Ok(Self {
            events,
            listeners: HashMap::new(),
            wildcard_listeners: Vec::new(),
        })
    }

    fn now() -> f64 {
        #[cfg(target_arch = "wasm32")]
        {
            // In WASM, we'd use js_sys::Date::now() but for the core crate
            // we keep it simple — callers can override timestamps.
            0.0
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0)
        }
    }
}

impl Default for EventStore {
    fn default() -> Self {
        Self::new()
    }
}

impl WorldEvent {
    /// Return a string tag for the event type (used for listener dispatch).
    pub fn event_type(&self) -> String {
        match self {
            WorldEvent::AgentSpawn(_) => "AgentSpawn",
            WorldEvent::AgentDespawn { .. } => "AgentDespawn",
            WorldEvent::AgentMove { .. } => "AgentMove",
            WorldEvent::AgentStatusChange { .. } => "AgentStatusChange",
            WorldEvent::AgentThink { .. } => "AgentThink",
            WorldEvent::AgentEquipTool { .. } => "AgentEquipTool",
            WorldEvent::AgentUseTool { .. } => "AgentUseTool",
            WorldEvent::AgentToolResult { .. } => "AgentToolResult",
            WorldEvent::ArtifactCreate(_) => "ArtifactCreate",
            WorldEvent::AgentPickUp { .. } => "AgentPickUp",
            WorldEvent::AgentDrop { .. } => "AgentDrop",
            WorldEvent::AgentTransfer { .. } => "AgentTransfer",
            WorldEvent::ArtifactQualityChange { .. } => "ArtifactQualityChange",
            WorldEvent::MessageSend(_) => "MessageSend",
            WorldEvent::RoomCreate(_) => "RoomCreate",
            WorldEvent::RoomEnter { .. } => "RoomEnter",
            WorldEvent::RoomExit { .. } => "RoomExit",
            WorldEvent::HumanCommand { .. } => "HumanCommand",
            WorldEvent::AgentError { .. } => "AgentError",
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_agent(id: &str, name: &str) -> Agent {
        Agent {
            id: id.to_string(),
            name: name.to_string(),
            role: "tester".to_string(),
            provider: AgentProvider::Claude,
            status: AgentStatus::Idle,
            position: Position { x: 0.0, y: 0.0 },
            room_id: "main".to_string(),
            sprite: SpriteConfig::default(),
            equipped_tools: vec![],
            inventory: vec![],
            current_task: None,
            health: 100.0,
            energy: 100.0,
            thought: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn emit_and_replay() {
        let mut store = EventStore::new();
        store.emit(WorldEvent::AgentSpawn(test_agent("a1", "Alice")));
        store.emit(WorldEvent::AgentMove {
            agent_id: "a1".to_string(),
            to: Position { x: 5.0, y: 3.0 },
        });
        store.emit(WorldEvent::AgentStatusChange {
            agent_id: "a1".to_string(),
            status: AgentStatus::Thinking,
            reason: Some("processing".to_string()),
        });

        assert_eq!(store.len(), 3);
        assert_eq!(store.replay(0, None).len(), 3);
        assert_eq!(store.replay(1, Some(3)).len(), 2);
    }

    #[test]
    fn serialization_roundtrip() {
        let mut store = EventStore::new();
        store.emit(WorldEvent::AgentSpawn(test_agent("a1", "Alice")));
        store.emit(WorldEvent::AgentMove {
            agent_id: "a1".to_string(),
            to: Position { x: 10.0, y: 20.0 },
        });

        let json = store.to_json().unwrap();
        let restored = EventStore::from_json(&json).unwrap();
        assert_eq!(restored.len(), 2);
    }

    #[test]
    fn event_type_tags() {
        let e = WorldEvent::AgentSpawn(test_agent("a1", "Alice"));
        assert_eq!(e.event_type(), "AgentSpawn");

        let e = WorldEvent::AgentMove {
            agent_id: "a1".to_string(),
            to: Position { x: 0.0, y: 0.0 },
        };
        assert_eq!(e.event_type(), "AgentMove");
    }
}
