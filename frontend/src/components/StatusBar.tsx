import { connectionState, agents, type ConnState } from "../store";
import { getWsUrl } from "../ws";

const STATE_COLORS: Record<ConnState, string> = {
  disconnected: "#808080",
  connecting: "#e6b31a",
  connected: "#33e64d",
  reconnecting: "#e66a1a",
};

export function StatusBar() {
  const state = connectionState.value;
  const isLive = !!getWsUrl();
  const count = agents.value.size;

  const dotColor = isLive ? STATE_COLORS[state] : "#33e64d";
  const label = isLive ? state : "demo";

  return (
    <div class="status-bar">
      <span class="status-dot-bar" style={{ backgroundColor: dotColor }} />
      <span class="status-text">
        {label} · {count} agent{count !== 1 ? "s" : ""}
      </span>
    </div>
  );
}
