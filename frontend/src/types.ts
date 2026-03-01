// Types mirroring agent-world-core WorldEvent variants

export interface Position { x: number; y: number }

export interface SpriteConfig {
  color: number[];
  shape: string;
  scale: number;
}

export type AgentStatus = "Idle" | "Thinking" | "Acting" | "Waiting" | "Error" | "Paused";

export interface Agent {
  id: string;
  name: string;
  role: string;
  provider: string | { Custom: string };
  status: AgentStatus;
  position: Position;
  room_id: string;
  sprite: SpriteConfig;
  equipped_tools: string[];
  inventory: string[];
  current_task: { id: string; description: string; progress: number } | null;
  health: number;
  energy: number;
  thought: string | null;
  metadata: Record<string, string>;
}

export interface Room {
  id: string;
  name: string;
  width: number;
  height: number;
  purpose: string;
  portals: { id: string; target_room: string; position: Position; target_position: Position }[];
}

export interface Message {
  id: string;
  from: string;
  to: string[];
  channel: string;
  content: string;
  content_preview: string;
  timestamp: number;
  visual_style: string;
}

// Discriminated union of all WorldEvent variants
export type WorldEvent =
  | { AgentSpawn: Agent }
  | { AgentDespawn: { agent_id: string } }
  | { AgentMove: { agent_id: string; to: Position } }
  | { AgentStatusChange: { agent_id: string; status: AgentStatus; reason: string | null } }
  | { AgentThink: { agent_id: string; thought: string } }
  | { AgentEquipTool: { agent_id: string; tool_id: string } }
  | { AgentUseTool: { agent_id: string; tool_id: string; target: string | null } }
  | { AgentToolResult: { agent_id: string; tool_id: string; success: boolean } }
  | { ArtifactCreate: unknown }
  | { AgentPickUp: { agent_id: string; artifact_id: string } }
  | { AgentDrop: { agent_id: string; artifact_id: string; position: Position } }
  | { AgentTransfer: { from_id: string; to_id: string; artifact_id: string } }
  | { ArtifactQualityChange: { artifact_id: string; quality: number } }
  | { MessageSend: Message }
  | { RoomCreate: Room }
  | { RoomEnter: { agent_id: string; room_id: string } }
  | { RoomExit: { agent_id: string; room_id: string } }
  | { HumanCommand: { target_id: string; command: string } }
  | { AgentError: { agent_id: string; error: string } };

// Helper to get the event variant key
export function eventType(event: WorldEvent): string {
  return Object.keys(event)[0];
}

// Helper to get event data
export function eventData(event: WorldEvent): unknown {
  return Object.values(event)[0];
}
