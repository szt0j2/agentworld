/**
 * AgentWorld Bridge Server
 *
 * Reads from the observability events.db (SQLite) and translates
 * Claude Code hook events into AgentWorld WorldEvents, streaming
 * them to connected WASM game clients via WebSocket.
 *
 * Usage: bun run bridge/server.ts [--db PATH] [--port PORT] [--replay]
 */

import { Database } from "bun:sqlite";

const DB_PATH = process.argv.includes("--db")
  ? process.argv[process.argv.indexOf("--db") + 1]
  : `${process.env.HOME}/ops/state/events.db`;

const PORT = process.argv.includes("--port")
  ? parseInt(process.argv[process.argv.indexOf("--port") + 1])
  : 9090;

const REPLAY = process.argv.includes("--replay");

const TEAM_FILTER = process.argv.includes("--team")
  ? process.argv[process.argv.indexOf("--team") + 1]
  : null;

// ─── Types matching agent-world-core WorldEvent variants ───

interface Position { x: number; y: number }

interface Agent {
  id: string;
  name: string;
  role: string;
  provider: string;
  status: string;
  position: Position;
  room_id: string;
  sprite: { color: number[]; shape: string; scale: number };
  equipped_tools: string[];
  inventory: string[];
  current_task: null;
  health: number;
  energy: number;
  thought: null | string;
  metadata: Record<string, string>;
}

interface Room {
  id: string;
  name: string;
  width: number;
  height: number;
  purpose: string;
  portals: Array<{
    id: string;
    target_room: string;
    position: Position;
    target_position: Position;
  }>;
}

// ─── Agent state tracking ───

const agents = new Map<string, {
  name: string;
  role: string;
  room: string;
  pos: Position;
  status: string;
  color: number[];
}>();

const rooms = new Map<string, { spawned: boolean; pos: Position }>();
const knownTeams = new Set<string>();

// Room layout constants
const ROOM_WIDTH = 500;
const ROOM_HEIGHT = 400;
const ROOM_SPACING = 700;

// Agent colors (rotate through)
const AGENT_COLORS: number[][] = [
  [100, 149, 237, 255], // cornflower blue
  [152, 251, 152, 255], // pale green
  [255, 182, 193, 255], // light pink
  [255, 218, 185, 255], // peach
  [221, 160, 221, 255], // plum
  [176, 224, 230, 255], // powder blue
  [255, 255, 224, 255], // light yellow
  [240, 128, 128, 255], // light coral
];
let colorIdx = 0;

function nextColor(): number[] {
  const c = AGENT_COLORS[colorIdx % AGENT_COLORS.length];
  colorIdx++;
  return c;
}

// Room purposes/colors by convention
const ROOM_THEMES: Record<string, { purpose: string }> = {
  "main":      { purpose: "Main workspace" },
  "research":  { purpose: "Research and analysis" },
  "coding":    { purpose: "Code development" },
  "review":    { purpose: "Code review" },
  "testing":   { purpose: "Testing and QA" },
  "deploy":    { purpose: "Deployment" },
};

// ─── Event translation ───

function prettifyName(id: string): string {
  return id
    .split(/[-_]/)
    .map(w => w.charAt(0).toUpperCase() + w.slice(1))
    .join(" ");
}

function ensureRoom(roomId: string): object[] {
  if (rooms.has(roomId)) return [];

  const idx = rooms.size;
  const pos: Position = { x: idx * ROOM_SPACING, y: 0 };
  rooms.set(roomId, { spawned: true, pos });

  const theme = ROOM_THEMES[roomId] || { purpose: roomId };

  // Build portals to existing rooms
  const portals: Room["portals"] = [];
  for (const [existingId, existing] of rooms) {
    if (existingId === roomId) continue;
    portals.push({
      id: `p-${roomId}-${existingId}`,
      target_room: prettifyName(existingId),
      position: { x: -ROOM_WIDTH / 2 + 30, y: 0 },
      target_position: { x: ROOM_WIDTH / 2 - 30, y: 0 },
    });
  }

  const room: Room = {
    id: roomId,
    name: prettifyName(roomId),
    width: ROOM_WIDTH,
    height: ROOM_HEIGHT,
    purpose: theme.purpose,
    portals,
  };

  return [{ RoomCreate: room }];
}

function agentPosition(roomId: string): Position {
  const roomData = rooms.get(roomId);
  const baseX = roomData ? roomData.pos.x : 0;
  // Scatter agents within the room
  const agentsInRoom = [...agents.values()].filter(a => a.room === roomId).length;
  const col = agentsInRoom % 3;
  const row = Math.floor(agentsInRoom / 3);
  return {
    x: baseX + 100 + col * 150,
    y: -50 + row * 150,
  };
}

function translateEvent(row: {
  id: number;
  hook_event_type: string;
  event_category: string | null;
  team_name: string | null;
  agent_name: string | null;
  agent_type: string | null;
  payload: string;
  summary: string | null;
  timestamp: number;
}): object[] {
  const events: object[] = [];
  const category = row.event_category || "";

  let payload: any;
  try {
    payload = JSON.parse(row.payload);
  } catch {
    return [];
  }

  const toolName = payload.tool_name || "";
  const toolInput = payload.tool_input || {};
  const teamName = row.team_name || "default";

  // Track teams as rooms
  if (teamName && !knownTeams.has(teamName)) {
    knownTeams.add(teamName);
    events.push(...ensureRoom(teamName));
  }

  switch (category) {
    case "agent_spawn": {
      // An agent is being created (Task tool with team_name)
      const agentName = toolInput.name || `agent-${agents.size}`;
      const agentId = `${teamName}/${agentName}`;

      if (!agents.has(agentId)) {
        events.push(...ensureRoom(teamName));
        const pos = agentPosition(teamName);
        const color = nextColor();

        const role = toolInput.subagent_type || "agent";
        agents.set(agentId, {
          name: agentName,
          role,
          room: teamName,
          pos,
          status: "Idle",
          color,
        });

        const agent: Agent = {
          id: agentId,
          name: agentName,
          role,
          provider: "Claude",
          status: "Idle",
          position: pos,
          room_id: teamName,
          sprite: { color, shape: "Square", scale: 1.0 },
          equipped_tools: [],
          inventory: [],
          current_task: null,
          health: 100,
          energy: 100,
          thought: null,
          metadata: {},
        };

        events.push({ AgentSpawn: agent });
        events.push({ RoomEnter: { agent_id: agentId, room_id: teamName } });
      }
      break;
    }

    case "agent_stop": {
      // Agent stopping (SubagentStop)
      const agentName = row.agent_name || "";
      const agentId = `${teamName}/${agentName}`;

      if (agents.has(agentId)) {
        // First set to Paused, then despawn after a visual beat
        events.push({
          AgentStatusChange: {
            agent_id: agentId,
            status: "Paused",
            reason: "Agent stopped",
          },
        });
        events.push({
          AgentDespawn: { agent_id: agentId },
        });
        agents.delete(agentId);
      }
      break;
    }

    case "tool_use": {
      // Agent using a tool
      const agentName = row.agent_name || "lead";
      const agentId = teamName ? `${teamName}/${agentName}` : agentName;

      // Auto-create agent if not seen
      if (!agents.has(agentId) && teamName) {
        events.push(...ensureRoom(teamName));
        const pos = agentPosition(teamName);
        const color = nextColor();
        const autoRole = row.agent_type || "agent";
        agents.set(agentId, { name: agentName, role: autoRole, room: teamName, pos, status: "Idle", color });

        events.push({
          AgentSpawn: {
            id: agentId,
            name: agentName,
            role: autoRole,
            provider: "Claude",
            status: "Idle",
            position: pos,
            room_id: teamName,
            sprite: { color, shape: "Square", scale: 1.0 },
            equipped_tools: [],
            inventory: [],
            current_task: null,
            health: 100,
            energy: 100,
            thought: null,
            metadata: {},
          },
        });
      }

      if (row.hook_event_type === "PreToolUse") {
        // Agent starts using tool → Acting status + tool use event
        events.push({
          AgentStatusChange: {
            agent_id: agentId,
            status: "Acting",
            reason: `Using ${toolName}`,
          },
        });

        events.push({
          AgentUseTool: {
            agent_id: agentId,
            tool_id: toolName,
            target: toolInput.description || toolInput.command?.substring(0, 50) || null,
          },
        });

        // Small position jitter — makes the agent visually "move around" while working
        const agentData = agents.get(agentId);
        if (agentData) {
          const jitterX = (Math.random() - 0.5) * 40;
          const jitterY = (Math.random() - 0.5) * 40;
          events.push({
            AgentMove: {
              agent_id: agentId,
              to: { x: agentData.pos.x + jitterX, y: agentData.pos.y + jitterY },
            },
          });
        }

        // Show thought with tool context
        const thought = toolInput.description
          || toolInput.command?.substring(0, 40)
          || toolInput.pattern
          || toolInput.file_path
          || toolName;

        events.push({
          AgentThink: {
            agent_id: agentId,
            thought: `${toolName}: ${thought}`,
          },
        });
      } else if (row.hook_event_type === "PostToolUse") {
        // Tool completed → result + back to thinking
        events.push({
          AgentToolResult: {
            agent_id: agentId,
            tool_id: toolName,
            success: true,
          },
        });

        events.push({
          AgentStatusChange: {
            agent_id: agentId,
            status: "Thinking",
            reason: null,
          },
        });

        // File operations create visible artifacts
        if (toolName === "Write" || toolName === "Edit" || toolName === "NotebookEdit") {
          const filePath = toolInput.file_path || "";
          const fileName = filePath.split("/").pop() || "file";
          const agentData = agents.get(agentId);
          const artifactPos = agentData
            ? { x: agentData.pos.x + 30, y: agentData.pos.y - 20 }
            : { x: 0, y: 0 };

          events.push({
            ArtifactCreate: {
              id: `artifact-${row.id}`,
              name: fileName,
              kind: "Code",
              content_ref: filePath,
              owner: agentId,
              quality: 0.8,
              position: artifactPos,
              room_id: teamName,
              sprite: { color: [100, 200, 100, 255], shape: "Square", scale: 0.6 },
            },
          });
        }
      }
      break;
    }

    case "message": {
      // Inter-agent message (SendMessage)
      const agentName = row.agent_name || "lead";
      const agentId = teamName ? `${teamName}/${agentName}` : agentName;
      const recipient = toolInput.recipient || "broadcast";
      const recipientId = teamName ? `${teamName}/${recipient}` : recipient;
      const content = toolInput.content || toolInput.summary || "";

      events.push({
        MessageSend: {
          id: `msg-${row.id}`,
          from: agentId,
          to: toolInput.type === "broadcast"
            ? [...agents.keys()].filter(k => k !== agentId)
            : [recipientId],
          channel: toolInput.type === "broadcast" ? "Broadcast" : "Direct",
          content: content.substring(0, 200),
          content_preview: content.substring(0, 40),
          timestamp: row.timestamp / 1000,
          visual_style: "Projectile",
        },
      });
      break;
    }

    case "task_mgmt": {
      // Task management — show as thought
      const agentName = row.agent_name || "lead";
      const agentId = teamName ? `${teamName}/${agentName}` : agentName;

      if (toolName === "TaskCreate") {
        events.push({
          AgentThink: {
            agent_id: agentId,
            thought: `New task: ${toolInput.subject || ""}`,
          },
        });
      } else if (toolName === "TaskUpdate") {
        const status = toolInput.status || "";
        if (status === "completed") {
          events.push({
            AgentThink: {
              agent_id: agentId,
              thought: `Task done ✓`,
            },
          });
        }
      }
      break;
    }

    case "team_lifecycle": {
      // Team create/delete
      if (toolName === "TeamCreate") {
        const newTeam = toolInput.team_name || teamName;
        events.push(...ensureRoom(newTeam));
      }
      break;
    }
  }

  return events;
}

// ─── WebSocket server ───

const clients = new Set<any>();
let lastEventId = 0;

function broadcastToClients(worldEvents: object[]) {
  for (const event of worldEvents) {
    const json = JSON.stringify(event);
    for (const ws of clients) {
      try {
        ws.send(json);
      } catch {
        clients.delete(ws);
      }
    }
  }
}

// ─── Polling loop ───

function pollNewEvents(db: Database) {
  const query = TEAM_FILTER
    ? "SELECT id, hook_event_type, event_category, team_name, agent_name, agent_type, payload, summary, timestamp FROM events WHERE id > ? AND team_name = ? ORDER BY id ASC LIMIT 50"
    : "SELECT id, hook_event_type, event_category, team_name, agent_name, agent_type, payload, summary, timestamp FROM events WHERE id > ? ORDER BY id ASC LIMIT 50";

  const stmt = db.prepare(query);
  const rows = (TEAM_FILTER ? stmt.all(lastEventId, TEAM_FILTER) : stmt.all(lastEventId)) as any[];

  for (const row of rows) {
    lastEventId = row.id;
    const worldEvents = translateEvent(row);
    if (worldEvents.length > 0) {
      broadcastToClients(worldEvents);
    }
  }
}

// ─── Main ───

console.log(`AgentWorld Bridge Server`);
console.log(`  DB: ${DB_PATH}`);
console.log(`  Port: ${PORT}`);
console.log(`  Replay: ${REPLAY}`);
console.log(`  Team filter: ${TEAM_FILTER || "(all)"}`);


const db = new Database(DB_PATH, { readonly: true });

// Determine starting point
if (!REPLAY) {
  // Start from now — only see new events
  const latest = db.prepare("SELECT MAX(id) as max_id FROM events").get() as any;
  lastEventId = latest?.max_id || 0;
  console.log(`  Starting from event #${lastEventId} (latest, live only)`);
} else {
  // Replay recent events
  const count = db.prepare("SELECT COUNT(*) as cnt FROM events").get() as any;
  const total = count?.cnt || 0;

  // Find events from the last hour
  const oneHourAgo = Date.now() - 3600 * 1000;
  const recentQuery = TEAM_FILTER
    ? "SELECT MIN(id) as min_id FROM events WHERE timestamp > ? AND team_name = ?"
    : "SELECT MIN(id) as min_id FROM events WHERE timestamp > ?";
  const recent = (TEAM_FILTER
    ? db.prepare(recentQuery).get(oneHourAgo, TEAM_FILTER)
    : db.prepare(recentQuery).get(oneHourAgo)) as any;

  lastEventId = (recent?.min_id || total) - 1;
  console.log(`  Replaying from event #${lastEventId} (${total} total, last hour)`);
}

// Start WebSocket server
const server = Bun.serve({
  port: PORT,
  fetch(req, server) {
    const url = new URL(req.url);

    // WebSocket upgrade
    if (url.pathname === "/ws") {
      if (server.upgrade(req)) return undefined;
      return new Response("WebSocket upgrade failed", { status: 500 });
    }

    // Health check
    if (url.pathname === "/health") {
      return Response.json({
        status: "ok",
        clients: clients.size,
        lastEventId,
        agents: agents.size,
        rooms: rooms.size,
      });
    }

    return new Response("AgentWorld Bridge - connect via ws://HOST:PORT/ws", {
      status: 200,
    });
  },
  websocket: {
    open(ws) {
      clients.add(ws);
      console.log(`Client connected (${clients.size} total)`);

      // Send current room/agent state to new client
      for (const [roomId, room] of rooms) {
        const roomEvent = ensureRoom(roomId);
        // Room already exists, send a fresh RoomCreate
        ws.send(JSON.stringify({
          RoomCreate: {
            id: roomId,
            name: prettifyName(roomId),
            width: ROOM_WIDTH,
            height: ROOM_HEIGHT,
            purpose: ROOM_THEMES[roomId]?.purpose || roomId,
            portals: [],
          },
        }));
      }

      for (const [agentId, agent] of agents) {
        ws.send(JSON.stringify({
          AgentSpawn: {
            id: agentId,
            name: agent.name,
            role: agent.role,
            provider: "Claude",
            status: agent.status,
            position: agent.pos,
            room_id: agent.room,
            sprite: { color: agent.color, shape: "Square", scale: 1.0 },
            equipped_tools: [],
            inventory: [],
            current_task: null,
            health: 100,
            energy: 100,
            thought: null,
            metadata: {},
          },
        }));
      }
    },
    close(ws) {
      clients.delete(ws);
      console.log(`Client disconnected (${clients.size} total)`);
    },
    message(ws, msg) {
      // Clients don't send messages in this version
    },
  },
});

console.log(`  WebSocket: ws://0.0.0.0:${PORT}/ws`);
console.log(`  Health: http://0.0.0.0:${PORT}/health`);
console.log(`  Polling every 500ms...`);

// Poll for new events every 500ms
setInterval(() => pollNewEvents(db), 500);
