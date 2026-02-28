use agent_world_core::WorldEvent;
use bevy::prelude::*;
use crate::plugins::events::{PendingEvents, PendingVisualEvents};
use crate::plugins::hud::EventLog;

pub struct AdapterPlugin;

/// Configuration for the WebSocket connection.
#[derive(Resource)]
pub struct AdapterConfig {
    pub url: String,
    pub enabled: bool,
}

impl Default for AdapterConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            enabled: false,
        }
    }
}

/// Internal state for the WebSocket connection.
/// Uses NonSend because web_sys::WebSocket and Rc<RefCell> are not Send+Sync.
/// This is fine — WASM is single-threaded.
struct WsState {
    #[cfg(target_arch = "wasm32")]
    incoming: std::rc::Rc<std::cell::RefCell<Vec<String>>>,
    #[cfg(target_arch = "wasm32")]
    ws: Option<web_sys::WebSocket>,
    connected: bool,
}

impl Default for WsState {
    fn default() -> Self {
        Self {
            #[cfg(target_arch = "wasm32")]
            incoming: std::rc::Rc::new(std::cell::RefCell::new(Vec::new())),
            #[cfg(target_arch = "wasm32")]
            ws: None,
            connected: false,
        }
    }
}

impl Plugin for AdapterPlugin {
    fn build(&self, app: &mut App) {
        let config = get_config_from_url();

        app.insert_resource(config)
            .insert_non_send_resource(WsState::default())
            .add_systems(Update, (
                try_connect,
                poll_messages,
            ).chain());
    }
}

/// Read ?ws=... from the browser URL to auto-configure the adapter.
fn get_config_from_url() -> AdapterConfig {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(search) = window.location().search() {
                if let Some(ws_param) = search
                    .trim_start_matches('?')
                    .split('&')
                    .find_map(|pair| {
                        let mut parts = pair.splitn(2, '=');
                        let key = parts.next()?;
                        let val = parts.next()?;
                        if key == "ws" { Some(val.to_string()) } else { None }
                    })
                {
                    return AdapterConfig {
                        url: ws_param,
                        enabled: true,
                    };
                }
            }
        }
    }
    AdapterConfig::default()
}

/// Try to establish WebSocket connection if configured and not connected.
fn try_connect(
    config: Res<AdapterConfig>,
    mut state: NonSendMut<WsState>,
    mut log: ResMut<EventLog>,
) {
    if !config.enabled || state.connected || config.url.is_empty() {
        return;
    }

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::prelude::*;
        use web_sys::MessageEvent;

        match web_sys::WebSocket::new(&config.url) {
            Ok(ws) => {
                ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

                let incoming = state.incoming.clone();

                // onmessage callback — push text into shared buffer
                let onmessage = Closure::wrap(Box::new(move |e: MessageEvent| {
                    if let Ok(text) = e.data().dyn_into::<js_sys::JsString>() {
                        let s: String = text.into();
                        incoming.borrow_mut().push(s);
                    }
                }) as Box<dyn FnMut(MessageEvent)>);
                ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
                onmessage.forget();

                // onopen
                let onopen = Closure::wrap(Box::new(move |_: JsValue| {
                    web_sys::console::log_1(&"AgentWorld: WebSocket connected".into());
                }) as Box<dyn FnMut(JsValue)>);
                ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
                onopen.forget();

                // onerror
                let onerror = Closure::wrap(Box::new(move |_: JsValue| {
                    web_sys::console::log_1(&"AgentWorld: WebSocket error".into());
                }) as Box<dyn FnMut(JsValue)>);
                ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
                onerror.forget();

                state.ws = Some(ws);
                state.connected = true;
                log.push("WS: connected".into());
            }
            Err(_) => {
                log.push("WS: connection failed".into());
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        log.push("WS: not available (native)".into());
        state.connected = false; // suppress unused warning
    }
}

/// Poll incoming WebSocket messages and dispatch as WorldEvents.
fn poll_messages(
    mut state: NonSendMut<WsState>,
    mut pending: ResMut<PendingEvents>,
    mut visual: ResMut<PendingVisualEvents>,
    mut log: ResMut<EventLog>,
) {
    if !state.connected {
        return;
    }

    #[cfg(target_arch = "wasm32")]
    {
        // Check if connection is still open
        if let Some(ref ws) = state.ws {
            if ws.ready_state() == web_sys::WebSocket::CLOSED {
                state.connected = false;
                log.push("WS: disconnected".into());
                return;
            }
        }

        let messages: Vec<String> = state.incoming.borrow_mut().drain(..).collect();

        for msg in messages {
            match serde_json::from_str::<WorldEvent>(&msg) {
                Ok(event) => {
                    match &event {
                        WorldEvent::AgentSpawn(_)
                        | WorldEvent::AgentDespawn { .. }
                        | WorldEvent::AgentMove { .. }
                        | WorldEvent::AgentStatusChange { .. }
                        | WorldEvent::RoomCreate(_)
                        | WorldEvent::RoomEnter { .. }
                        | WorldEvent::RoomExit { .. } => {
                            pending.queue.push(event);
                        }
                        WorldEvent::AgentThink { .. }
                        | WorldEvent::AgentUseTool { .. }
                        | WorldEvent::AgentToolResult { .. }
                        | WorldEvent::MessageSend(_)
                        | WorldEvent::ArtifactCreate(_)
                        | WorldEvent::AgentPickUp { .. }
                        | WorldEvent::AgentDrop { .. }
                        | WorldEvent::AgentTransfer { .. }
                        | WorldEvent::ArtifactQualityChange { .. } => {
                            visual.queue.push(event);
                        }
                        _ => {
                            pending.queue.push(event);
                        }
                    }
                }
                Err(e) => {
                    log.push(format!("WS: parse error: {}", e));
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (&mut pending, &mut visual, &mut log);
    }
}
