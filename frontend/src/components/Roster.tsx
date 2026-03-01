import { agentsByTeam, selectedAgent, type AgentState, type AgentStatus } from "../store";

const STATUS_COLORS: Record<AgentStatus, string> = {
  Idle: "#888899",
  Thinking: "#4d80ff",
  Acting: "#33e64d",
  Waiting: "#e6b31a",
  Error: "#ff3333",
  Paused: "#808080",
};

function AgentRow({ agent }: { agent: AgentState }) {
  const selected = selectedAgent.value === agent.id;
  const color = STATUS_COLORS[agent.status] ?? "#888";

  return (
    <button
      class={`roster-entry ${selected ? "selected" : ""}`}
      onClick={() => {
        selectedAgent.value = selectedAgent.value === agent.id ? null : agent.id;
        // Dispatch event for Bevy camera follow
        window.dispatchEvent(new CustomEvent("agentworld:select", { detail: agent.id }));
      }}
    >
      <span class="status-dot" style={{ backgroundColor: color }} />
      <span class="agent-name">{agent.name}</span>
      <span class="agent-role">{agent.role}</span>
    </button>
  );
}

export function Roster() {
  const teams = agentsByTeam.value;

  if (teams.size === 0) {
    return (
      <div class="panel roster">
        <div class="panel-title">AGENTS</div>
        <div class="empty-state">No agents yet</div>
      </div>
    );
  }

  return (
    <div class="panel roster">
      <div class="panel-title">AGENTS</div>
      <div class="roster-list">
        {[...teams.entries()].map(([team, members]) => (
          <div key={team} class="team-group">
            <div class="team-header">[{team.toUpperCase()}]</div>
            {members.map(agent => (
              <AgentRow key={agent.id} agent={agent} />
            ))}
          </div>
        ))}
      </div>
    </div>
  );
}
