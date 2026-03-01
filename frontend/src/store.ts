// Reactive state store using Preact signals
import { signal, computed } from "@preact/signals";
import type { Agent, AgentStatus, WorldEvent, Message } from "./types";

// Agent state
export const agents = signal<Map<string, AgentState>>(new Map());

export interface AgentState {
  id: string;
  name: string;
  role: string;
  status: AgentStatus;
  room_id: string;
  color: number[];
  health: number;
  energy: number;
  thought: string | null;
  toolCount: number;
  lastTool: string | null;
  equipped_tools: string[];
}

// Event log
export interface EventEntry {
  id: number;
  time: number;
  text: string;
  type: string; // event category for styling
}

let eventCounter = 0;
export const eventLog = signal<EventEntry[]>([]);
const MAX_EVENTS = 200;

// Inspector
export const selectedAgent = signal<string | null>(null);

// Connection
export type ConnState = "disconnected" | "connecting" | "connected" | "reconnecting";
export const connectionState = signal<ConnState>("disconnected");

// Derived: agents grouped by team
export const agentsByTeam = computed(() => {
  const map = new Map<string, AgentState[]>();
  for (const agent of agents.value.values()) {
    const team = agent.id.includes("/") ? agent.id.split("/")[0] : "agents";
    if (!map.has(team)) map.set(team, []);
    map.get(team)!.push(agent);
  }
  return map;
});

// Push an event log entry
function pushEvent(text: string, type: string) {
  const entries = [...eventLog.value, { id: eventCounter++, time: Date.now(), text, type }];
  if (entries.length > MAX_EVENTS) entries.splice(0, entries.length - MAX_EVENTS);
  eventLog.value = entries;
}

// Short agent name: "team/agent-name" → "agent-name"
function shortName(id: string): string {
  return id.includes("/") ? id.split("/").slice(1).join("/") : id;
}

// Sync agent roster from Bevy's periodic state dump.
// This ensures agents appear even if spawn events were missed.
export function syncAgents(agentList: Array<{
  id: string; name: string; role: string; status: AgentStatus;
  toolCount: number; lastTool: string | null; thought: string | null;
}>) {
  const next = new Map<string, AgentState>();
  for (const a of agentList) {
    const existing = agents.value.get(a.id);
    next.set(a.id, {
      id: a.id,
      name: a.name,
      role: a.role,
      status: a.status,
      room_id: existing?.room_id ?? "",
      color: existing?.color ?? [100, 149, 237, 255],
      health: existing?.health ?? 100,
      energy: existing?.energy ?? 100,
      thought: a.thought,
      toolCount: a.toolCount,
      lastTool: a.lastTool,
      equipped_tools: existing?.equipped_tools ?? [],
    });
  }
  agents.value = next;
}

// Process a WorldEvent
export function processEvent(event: WorldEvent) {
  const key = Object.keys(event)[0] as string;
  const data = (event as Record<string, unknown>)[key];

  switch (key) {
    case "AgentSpawn": {
      const a = data as Agent;
      const next = new Map(agents.value);
      next.set(a.id, {
        id: a.id,
        name: a.name,
        role: a.role,
        status: a.status,
        room_id: a.room_id,
        color: a.sprite.color,
        health: a.health,
        energy: a.energy,
        thought: a.thought,
        toolCount: 0,
        lastTool: null,
        equipped_tools: a.equipped_tools,
      });
      agents.value = next;
      pushEvent(`+ ${a.name} joined (${a.role})`, "spawn");
      break;
    }
    case "AgentDespawn": {
      const { agent_id } = data as { agent_id: string };
      const next = new Map(agents.value);
      const name = next.get(agent_id)?.name ?? shortName(agent_id);
      next.delete(agent_id);
      agents.value = next;
      if (selectedAgent.value === agent_id) selectedAgent.value = null;
      pushEvent(`- ${name} left`, "despawn");
      break;
    }
    case "AgentStatusChange": {
      const { agent_id, status, reason } = data as { agent_id: string; status: AgentStatus; reason: string | null };
      const next = new Map(agents.value);
      const agent = next.get(agent_id);
      if (agent) {
        next.set(agent_id, { ...agent, status });
        agents.value = next;
      }
      const name = agent?.name ?? shortName(agent_id);
      const extra = reason ? ` (${reason})` : "";
      pushEvent(`${name} → ${status}${extra}`, "status");
      break;
    }
    case "AgentThink": {
      const { agent_id, thought } = data as { agent_id: string; thought: string };
      const next = new Map(agents.value);
      const agent = next.get(agent_id);
      if (agent) {
        next.set(agent_id, { ...agent, thought });
        agents.value = next;
      }
      break;
    }
    case "AgentUseTool": {
      const { agent_id, tool_id } = data as { agent_id: string; tool_id: string };
      const next = new Map(agents.value);
      const agent = next.get(agent_id);
      if (agent) {
        next.set(agent_id, { ...agent, lastTool: tool_id, toolCount: agent.toolCount + 1 });
        agents.value = next;
      }
      const name = agent?.name ?? shortName(agent_id);
      pushEvent(`${name} > ${tool_id}`, "tool");
      break;
    }
    case "AgentToolResult": {
      const { agent_id, tool_id, success } = data as { agent_id: string; tool_id: string; success: boolean };
      const name = agents.value.get(agent_id)?.name ?? shortName(agent_id);
      pushEvent(`${name} < ${tool_id} ${success ? "✓" : "✗"}`, success ? "tool-ok" : "tool-fail");
      break;
    }
    case "MessageSend": {
      const msg = data as Message;
      const from = agents.value.get(msg.from)?.name ?? shortName(msg.from);
      const to = msg.to.map(id => agents.value.get(id)?.name ?? shortName(id)).join(", ");
      const preview = msg.content_preview.length > 50 ? msg.content_preview.slice(0, 47) + "..." : msg.content_preview;
      pushEvent(`${from} → ${to}: ${preview}`, "message");
      break;
    }
    case "RoomCreate": {
      const room = data as { id: string; name: string; purpose: string };
      pushEvent(`Room: ${room.name} (${room.purpose})`, "room");
      break;
    }
    case "RoomEnter": {
      const { agent_id, room_id } = data as { agent_id: string; room_id: string };
      const next = new Map(agents.value);
      const agent = next.get(agent_id);
      if (agent) {
        next.set(agent_id, { ...agent, room_id });
        agents.value = next;
      }
      const name = agent?.name ?? shortName(agent_id);
      pushEvent(`${name} >> ${room_id}`, "portal");
      break;
    }
    case "RoomExit": {
      const { agent_id, room_id } = data as { agent_id: string; room_id: string };
      const name = agents.value.get(agent_id)?.name ?? shortName(agent_id);
      pushEvent(`${name} << ${room_id}`, "portal");
      break;
    }
    case "AgentError": {
      const { agent_id, error } = data as { agent_id: string; error: string };
      const name = agents.value.get(agent_id)?.name ?? shortName(agent_id);
      pushEvent(`⚠ ${name}: ${error}`, "error");
      break;
    }
    case "AgentEquipTool": {
      const { agent_id, tool_id } = data as { agent_id: string; tool_id: string };
      const next = new Map(agents.value);
      const agent = next.get(agent_id);
      if (agent && !agent.equipped_tools.includes(tool_id)) {
        next.set(agent_id, { ...agent, equipped_tools: [...agent.equipped_tools, tool_id] });
        agents.value = next;
      }
      break;
    }
    // AgentMove handled by Bevy, we don't need world positions in React
  }
}
