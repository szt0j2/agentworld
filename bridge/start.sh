#!/usr/bin/env bash
# Start the AgentWorld bridge server
# Usage: ./bridge/start.sh [--replay] [--team TEAM_NAME]

cd "$(dirname "$0")/.." || exit 1

exec bun run ./bridge/server.ts "$@"
