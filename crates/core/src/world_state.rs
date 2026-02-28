use crate::events::{EventStore, WorldEvent};
use crate::types::*;
use std::collections::HashMap;

/// WorldState is a projection built by replaying events.
/// It provides the current state of every entity in the world.
#[derive(Debug, Clone, Default)]
pub struct WorldState {
    pub agents: HashMap<String, Agent>,
    pub artifacts: HashMap<String, Artifact>,
    pub tools: HashMap<String, Tool>,
    pub rooms: HashMap<String, Room>,
    pub messages: Vec<Message>,
}

impl WorldState {
    /// Build a complete WorldState by replaying all events in the store.
    pub fn from_events(store: &EventStore) -> Self {
        let mut state = Self::default();
        for entry in store.all_events() {
            state.apply(&entry.event);
        }
        state
    }

    /// Apply a single event to update the state.
    pub fn apply(&mut self, event: &WorldEvent) {
        match event {
            WorldEvent::AgentSpawn(agent) => {
                self.agents.insert(agent.id.clone(), agent.clone());
            }
            WorldEvent::AgentDespawn { agent_id } => {
                self.agents.remove(agent_id);
            }
            WorldEvent::AgentMove { agent_id, to } => {
                if let Some(agent) = self.agents.get_mut(agent_id) {
                    agent.position = *to;
                }
            }
            WorldEvent::AgentStatusChange {
                agent_id, status, ..
            } => {
                if let Some(agent) = self.agents.get_mut(agent_id) {
                    agent.status = *status;
                }
            }
            WorldEvent::AgentThink { agent_id, thought } => {
                if let Some(agent) = self.agents.get_mut(agent_id) {
                    agent.thought = Some(thought.clone());
                }
            }
            WorldEvent::AgentEquipTool {
                agent_id, tool_id, ..
            } => {
                if let Some(agent) = self.agents.get_mut(agent_id) {
                    if !agent.equipped_tools.contains(tool_id) {
                        agent.equipped_tools.push(tool_id.clone());
                    }
                }
            }
            WorldEvent::AgentUseTool { .. } | WorldEvent::AgentToolResult { .. } => {
                // These are transient events — no state mutation needed.
                // The game layer handles visual effects for these.
            }
            WorldEvent::ArtifactCreate(artifact) => {
                self.artifacts.insert(artifact.id.clone(), artifact.clone());
            }
            WorldEvent::AgentPickUp {
                agent_id,
                artifact_id,
            } => {
                if let Some(agent) = self.agents.get_mut(agent_id) {
                    if !agent.inventory.contains(artifact_id) {
                        agent.inventory.push(artifact_id.clone());
                    }
                }
                if let Some(artifact) = self.artifacts.get_mut(artifact_id) {
                    artifact.owner = Some(agent_id.clone());
                }
            }
            WorldEvent::AgentDrop {
                agent_id,
                artifact_id,
                position,
            } => {
                if let Some(agent) = self.agents.get_mut(agent_id) {
                    agent.inventory.retain(|id| id != artifact_id);
                }
                if let Some(artifact) = self.artifacts.get_mut(artifact_id) {
                    artifact.owner = None;
                    artifact.position = *position;
                }
            }
            WorldEvent::AgentTransfer {
                from_id,
                to_id,
                artifact_id,
            } => {
                if let Some(from) = self.agents.get_mut(from_id) {
                    from.inventory.retain(|id| id != artifact_id);
                }
                if let Some(to) = self.agents.get_mut(to_id) {
                    if !to.inventory.contains(artifact_id) {
                        to.inventory.push(artifact_id.clone());
                    }
                }
                if let Some(artifact) = self.artifacts.get_mut(artifact_id) {
                    artifact.owner = Some(to_id.clone());
                }
            }
            WorldEvent::ArtifactQualityChange {
                artifact_id,
                quality,
            } => {
                if let Some(artifact) = self.artifacts.get_mut(artifact_id) {
                    artifact.quality = *quality;
                }
            }
            WorldEvent::MessageSend(message) => {
                self.messages.push(message.clone());
            }
            WorldEvent::RoomCreate(room) => {
                self.rooms.insert(room.id.clone(), room.clone());
            }
            WorldEvent::RoomEnter {
                agent_id, room_id, ..
            } => {
                if let Some(agent) = self.agents.get_mut(agent_id) {
                    agent.room_id = room_id.clone();
                }
            }
            WorldEvent::RoomExit { .. } => {
                // Room exit is informational — RoomEnter sets the new room.
            }
            WorldEvent::HumanCommand { .. } => {}
            WorldEvent::AgentError { agent_id, .. } => {
                if let Some(agent) = self.agents.get_mut(agent_id) {
                    agent.status = AgentStatus::Error;
                }
            }
        }
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
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn project_state_from_events() {
        let mut store = EventStore::new();
        store.emit(WorldEvent::RoomCreate(Room {
            id: "main".to_string(),
            name: "Main Hall".to_string(),
            width: 800.0,
            height: 600.0,
            purpose: "workspace".to_string(),
            portals: vec![],
        }));
        store.emit(WorldEvent::AgentSpawn(test_agent("a1", "Alice")));
        store.emit(WorldEvent::AgentSpawn(test_agent("a2", "Bob")));
        store.emit(WorldEvent::AgentMove {
            agent_id: "a1".to_string(),
            to: Position { x: 100.0, y: 50.0 },
        });
        store.emit(WorldEvent::AgentStatusChange {
            agent_id: "a2".to_string(),
            status: AgentStatus::Thinking,
            reason: None,
        });

        let state = WorldState::from_events(&store);
        assert_eq!(state.agents.len(), 2);
        assert_eq!(state.rooms.len(), 1);

        let alice = &state.agents["a1"];
        assert_eq!(alice.position.x, 100.0);
        assert_eq!(alice.position.y, 50.0);

        let bob = &state.agents["a2"];
        assert_eq!(bob.status, AgentStatus::Thinking);
    }

    #[test]
    fn artifact_pickup_and_transfer() {
        let mut store = EventStore::new();
        store.emit(WorldEvent::AgentSpawn(test_agent("a1", "Alice")));
        store.emit(WorldEvent::AgentSpawn(test_agent("a2", "Bob")));
        store.emit(WorldEvent::ArtifactCreate(Artifact {
            id: "doc1".to_string(),
            name: "Report".to_string(),
            kind: ArtifactKind::Document,
            content_ref: String::new(),
            owner: None,
            quality: 0.5,
            position: Position { x: 50.0, y: 50.0 },
            room_id: "main".to_string(),
            sprite: SpriteConfig::default(),
        }));
        store.emit(WorldEvent::AgentPickUp {
            agent_id: "a1".to_string(),
            artifact_id: "doc1".to_string(),
        });
        store.emit(WorldEvent::AgentTransfer {
            from_id: "a1".to_string(),
            to_id: "a2".to_string(),
            artifact_id: "doc1".to_string(),
        });

        let state = WorldState::from_events(&store);
        assert!(state.agents["a1"].inventory.is_empty());
        assert_eq!(state.agents["a2"].inventory, vec!["doc1"]);
        assert_eq!(state.artifacts["doc1"].owner.as_deref(), Some("a2"));
    }

    #[test]
    fn agent_despawn() {
        let mut store = EventStore::new();
        store.emit(WorldEvent::AgentSpawn(test_agent("a1", "Alice")));
        assert_eq!(WorldState::from_events(&store).agents.len(), 1);

        store.emit(WorldEvent::AgentDespawn {
            agent_id: "a1".to_string(),
        });
        assert_eq!(WorldState::from_events(&store).agents.len(), 0);
    }
}
