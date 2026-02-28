#!/usr/bin/env bash
# Show AgentWorld service status
set -uo pipefail

echo "AgentWorld Status"
echo "================="

# Check trunk
TRUNK_PID=$(pgrep -f "trunk serve.*8080" 2>/dev/null | head -1)
if [[ -n "$TRUNK_PID" ]]; then
    CPU=$(ps -p "$TRUNK_PID" -o %cpu= 2>/dev/null | tr -d ' ')
    echo "  Trunk:   RUNNING (PID $TRUNK_PID, ${CPU}% CPU)"
else
    echo "  Trunk:   STOPPED"
fi

# Check bridge
BRIDGE_PID=$(pgrep -f "bridge/server.ts" 2>/dev/null | head -1)
if [[ -n "$BRIDGE_PID" ]]; then
    CPU=$(ps -p "$BRIDGE_PID" -o %cpu= 2>/dev/null | tr -d ' ')
    echo "  Bridge:  RUNNING (PID $BRIDGE_PID, ${CPU}% CPU)"
else
    echo "  Bridge:  STOPPED"
fi

# Check Playwright (should NOT be running unless actively screenshotting)
PW_PID=$(pgrep -f "ms-playwright/mcp-chrome" 2>/dev/null | head -1)
if [[ -n "$PW_PID" ]]; then
    CPU=$(ps -p "$PW_PID" -o %cpu= 2>/dev/null | tr -d ' ')
    echo "  Playwright: RUNNING (PID $PW_PID, ${CPU}% CPU) ⚠️  HIGH CPU - kill with ./bin/stop.sh"
else
    echo "  Playwright: not running (good)"
fi
