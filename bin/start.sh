#!/usr/bin/env bash
# Start AgentWorld services (trunk dev server + bridge)
# Usage: ./bin/start.sh [--live]  (--live enables bridge with replay)

set -euo pipefail
cd "$(dirname "$0")/.."

# Ensure cargo/trunk/bun are in PATH
source "$HOME/.cargo/env" 2>/dev/null || true
export PATH="$HOME/.bun/bin:$HOME/.cargo/bin:$PATH"

echo "Starting AgentWorld..."

# Start trunk dev server in background
trunk serve --address 0.0.0.0 --port 8080 > /tmp/trunk.log 2>&1 &
TRUNK_PID=$!
echo "  Trunk dev server: PID $TRUNK_PID (http://0.0.0.0:8080)"

if [[ "${1:-}" == "--live" ]]; then
    # Start bridge server with replay
    bun run ./bridge/server.ts --replay > /tmp/bridge.log 2>&1 &
    BRIDGE_PID=$!
    echo "  Bridge server:    PID $BRIDGE_PID (ws://0.0.0.0:9090/ws)"
    echo ""
    echo "Live mode: http://localhost:8080/?ws=ws://localhost:9090/ws"
else
    echo ""
    echo "Demo mode: http://localhost:8080"
    echo "  (use --live to connect to real agent sessions)"
fi

# Save PIDs for stop script
echo "$TRUNK_PID" > /tmp/agentworld.pids
echo "${BRIDGE_PID:-}" >> /tmp/agentworld.pids

echo ""
echo "Logs: /tmp/trunk.log, /tmp/bridge.log"
echo "Stop: ./bin/stop.sh"
