import { render } from "preact";
import { useEffect } from "preact/hooks";
import { Roster } from "./components/Roster";
import { EventLog } from "./components/EventLog";
import { Inspector } from "./components/Inspector";
import { StatusBar } from "./components/StatusBar";
import { setup } from "./ws";

function App() {
  useEffect(() => {
    setup();
  }, []);

  return (
    <>
      <div class="hud-left">
        <StatusBar />
        <Roster />
      </div>
      <div class="hud-right">
        <EventLog />
      </div>
      <div class="hud-bottom">
        <Inspector />
      </div>
    </>
  );
}

// Mount into the shell container
const root = document.getElementById("hud-root");
if (root) {
  render(<App />, root);
}
