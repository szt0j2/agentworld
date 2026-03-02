import { selectedAgent, agents, artifactsByOwner, type AgentStatus } from "../store";

const STATUS_COLORS: Record<AgentStatus, string> = {
  Idle: "#8888aa",
  Thinking: "#4d80ff",
  Acting: "#33e64d",
  Waiting: "#e6b31a",
  Error: "#ff3333",
  Paused: "#808080",
};

export function Inspector() {
  const id = selectedAgent.value;
  if (!id) return null;

  const agent = agents.value.get(id);
  if (!agent) {
    return (
      <div class="panel inspector">
        <div class="inspector-header">
          <span class="inspector-name">Agent not found</span>
          <button class="inspector-close" onClick={() => { selectedAgent.value = null; }}>×</button>
        </div>
      </div>
    );
  }

  const statusColor = STATUS_COLORS[agent.status] ?? "#888";

  return (
    <div class="panel inspector">
      <div class="inspector-header">
        <span class="inspector-name">{agent.name}</span>
        <span class="inspector-role">{agent.role}</span>
        <button class="inspector-close" onClick={() => { selectedAgent.value = null; }}>×</button>
      </div>

      <div class="inspector-row">
        <span class="inspector-label">Status</span>
        <span style={{ color: statusColor }}>{agent.status}</span>
      </div>

      <div class="inspector-row">
        <span class="inspector-label">Room</span>
        <span>{agent.room_id}</span>
      </div>

      <div class="inspector-row">
        <span class="inspector-label">Health / Energy</span>
        <span>{Math.round(agent.health)} / {Math.round(agent.energy)}</span>
      </div>

      <div class="inspector-row">
        <span class="inspector-label">Tools used</span>
        <span>{agent.toolCount}{agent.lastTool ? ` (last: ${agent.lastTool})` : ""}</span>
      </div>

      {agent.equipped_tools.length > 0 && (
        <div class="inspector-row">
          <span class="inspector-label">Equipped</span>
          <span>{agent.equipped_tools.join(", ")}</span>
        </div>
      )}

      {agent.thought && (
        <div class="inspector-thought">
          <span class="inspector-label">Thinking</span>
          <div class="thought-text">{agent.thought}</div>
        </div>
      )}

      <InventorySection agentId={id} />
    </div>
  );
}

const KIND_ICONS: Record<string, string> = {
  Document: "📄",
  Code: "📝",
  Data: "📊",
  Image: "🖼",
  Plan: "📋",
  MessageBundle: "💬",
};

const KIND_COLORS: Record<string, string> = {
  Document: "#e6d999",
  Code: "#66e680",
  Data: "#66b3ff",
  Image: "#e680cc",
  Plan: "#cc99ff",
  MessageBundle: "#ffcc4d",
};

function InventorySection({ agentId }: { agentId: string }) {
  const items = artifactsByOwner.value.get(agentId);
  if (!items || items.length === 0) return null;

  return (
    <div class="inspector-inventory">
      <span class="inspector-label">Inventory ({items.length})</span>
      <div class="inventory-grid">
        {items.map(art => (
          <div class="inventory-item" key={art.id} title={`${art.name} (${art.kind}) — quality: ${Math.round(art.quality * 100)}%`}>
            <span class="item-icon">{KIND_ICONS[art.kind] ?? "📦"}</span>
            <span class="item-name" style={{ color: KIND_COLORS[art.kind] ?? "#aaa" }}>{art.name}</span>
            <div class="item-quality" style={{ width: `${art.quality * 100}%` }} />
          </div>
        ))}
      </div>
    </div>
  );
}
