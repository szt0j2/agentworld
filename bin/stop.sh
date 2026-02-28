#!/usr/bin/env bash
# Stop all AgentWorld services
set -euo pipefail

echo "Stopping AgentWorld..."

# Kill from saved PIDs
if [[ -f /tmp/agentworld.pids ]]; then
    while read -r pid; do
        [[ -z "$pid" ]] && continue
        if kill -0 "$pid" 2>/dev/null; then
            kill "$pid" 2>/dev/null && echo "  Killed PID $pid"
        fi
    done < /tmp/agentworld.pids
    rm -f /tmp/agentworld.pids
fi

# Also catch any strays
pkill -f "trunk serve.*8080" 2>/dev/null && echo "  Killed trunk" || true
pkill -f "bridge/server.ts" 2>/dev/null && echo "  Killed bridge" || true

# Kill any Playwright MCP Chrome instances (major CPU hog)
pkill -f "ms-playwright/mcp-chrome" 2>/dev/null && echo "  Killed Playwright Chrome" || true

echo "Done."
