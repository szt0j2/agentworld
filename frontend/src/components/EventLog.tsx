import { useRef, useEffect } from "preact/hooks";
import { eventLog } from "../store";

const TYPE_COLORS: Record<string, string> = {
  spawn: "#5ae65a",
  despawn: "#e65a5a",
  status: "#5a8ae6",
  tool: "#b8b855",
  "tool-ok": "#5ae65a",
  "tool-fail": "#e65a5a",
  message: "#c09de6",
  portal: "#e6a85a",
  room: "#5ab8e6",
  error: "#ff4444",
};

export function EventLog() {
  const listRef = useRef<HTMLDivElement>(null);
  const entries = eventLog.value;

  // Auto-scroll to bottom when new events arrive
  useEffect(() => {
    const el = listRef.current;
    if (el) {
      el.scrollTop = el.scrollHeight;
    }
  }, [entries.length]);

  return (
    <div class="panel event-log">
      <div class="panel-title">EVENT LOG</div>
      <div class="event-list" ref={listRef}>
        {entries.length === 0 ? (
          <div class="empty-state">Waiting for events...</div>
        ) : (
          entries.map(entry => (
            <div key={entry.id} class="event-entry" style={{ color: TYPE_COLORS[entry.type] ?? "#aab" }}>
              <span class="event-time">{formatTime(entry.time)}</span>
              <span class="event-text">{entry.text}</span>
            </div>
          ))
        )}
      </div>
    </div>
  );
}

function formatTime(ms: number): string {
  const d = new Date(ms);
  return `${d.getHours().toString().padStart(2, "0")}:${d.getMinutes().toString().padStart(2, "0")}:${d.getSeconds().toString().padStart(2, "0")}`;
}
