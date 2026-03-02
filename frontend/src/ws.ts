// Event bridge: receives WorldEvents and agent state from Bevy WASM
// Bevy calls window.__agentworld_event(json) for events
// and window.__agentworld_sync(json) for periodic agent roster sync.
import { connectionState, processEvent, syncAgents, syncArtifacts } from "./store";
import type { WorldEvent } from "./types";

export function getWsUrl(): string | null {
  const params = new URLSearchParams(window.location.search);
  return params.get("ws");
}

export function setup() {
  // Event callback: Bevy forwards WorldEvents here
  (window as any).__agentworld_event = (json: string) => {
    try {
      const event: WorldEvent = JSON.parse(json);
      processEvent(event);
    } catch {
      // ignore parse errors
    }
  };

  // State sync callback: Bevy sends full agent roster + connection state every 0.5s
  (window as any).__agentworld_sync = (json: string) => {
    try {
      const payload = JSON.parse(json);
      if (payload.agents) {
        syncAgents(payload.agents);
      }
      if (payload.artifacts) {
        syncArtifacts(payload.artifacts);
      }
      if (payload.connection) {
        connectionState.value = payload.connection;
      }
    } catch {
      // ignore parse errors
    }
  };

  // Initial connection state
  connectionState.value = getWsUrl() ? "connecting" : "disconnected";
}
