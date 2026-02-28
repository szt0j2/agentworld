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

/// Connection status visible to the HUD.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
}

/// Internal state for the WebSocket connection.
/// Uses NonSend because web_sys::WebSocket and Rc<RefCell> are not Send+Sync.
struct WsState {
    #[cfg(target_arch = "wasm32")]
    incoming: std::rc::Rc<std::cell::RefCell<Vec<String>>>,
    #[cfg(target_arch = "wasm32")]
    ws: Option<web_sys::WebSocket>,
    connected: bool,
    reconnect_cooldown: f32,
    reconnect_attempts: u32,
}

impl Default for WsState {
    fn default() -> Self {
        Self {
            #[cfg(target_arch = "wasm32")]
            incoming: std::rc::Rc::new(std::cell::RefCell::new(Vec::new())),
            #[cfg(target_arch = "wasm32")]
            ws: None,
            connected: false,
            reconnect_cooldown: 0.0,
            reconnect_attempts: 0,
        }
    }
}

impl Plugin for AdapterPlugin {
    fn build(&self, app: &mut App) {
        let config = get_config_from_url();

        app.insert_resource(config)
            .init_resource::<ConnectionStatus>()
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
/// Includes reconnection with exponential backoff.
fn try_connect(
    config: Res<AdapterConfig>,
    time: Res<Time>,
    mut state: NonSendMut<WsState>,
    mut status: ResMut<ConnectionStatus>,
    mut log: ResMut<EventLog>,
) {
    if !config.enabled || state.connected || config.url.is_empty() {
        return;
    }

    // Reconnect cooldown
    if state.reconnect_cooldown > 0.0 {
        state.reconnect_cooldown -= time.delta_secs();
        *status = ConnectionStatus::Reconnecting;
        return;
    }

    *status = ConnectionStatus::Connecting;

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::prelude::*;
        use web_sys::MessageEvent;

        match web_sys::WebSocket::new(&config.url) {
            Ok(ws) => {
                ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

                let incoming = state.incoming.clone();

                let onmessage = Closure::wrap(Box::new(move |e: MessageEvent| {
                    if let Ok(text) = e.data().dyn_into::<js_sys::JsString>() {
                        let s: String = text.into();
                        incoming.borrow_mut().push(s);
                    }
                }) as Box<dyn FnMut(MessageEvent)>);
                ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
                onmessage.forget();

                let onopen = Closure::wrap(Box::new(move |_: JsValue| {
                    web_sys::console::log_1(&"AgentWorld: WebSocket connected".into());
                }) as Box<dyn FnMut(JsValue)>);
                ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
                onopen.forget();

                let onerror = Closure::wrap(Box::new(move |_: JsValue| {
                    web_sys::console::log_1(&"AgentWorld: WebSocket error".into());
                }) as Box<dyn FnMut(JsValue)>);
                ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
                onerror.forget();

                state.ws = Some(ws);
                state.connected = true;
                state.reconnect_attempts = 0;
                *status = ConnectionStatus::Connected;
                log.push("WS: connected".into());
            }
            Err(_) => {
                // Exponential backoff: 2s, 4s, 8s, 16s, max 30s
                state.reconnect_attempts += 1;
                state.reconnect_cooldown = (2.0_f32 * 2.0_f32.powi(state.reconnect_attempts as i32 - 1)).min(30.0);
                *status = ConnectionStatus::Reconnecting;
                log.push(format!("WS: failed, retry in {:.0}s", state.reconnect_cooldown));
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        log.push("WS: not available (native)".into());
        state.connected = false;
    }
}

/// Poll incoming WebSocket messages and dispatch as WorldEvents.
fn poll_messages(
    mut state: NonSendMut<WsState>,
    mut status: ResMut<ConnectionStatus>,
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
                state.ws = None;
                // Will auto-reconnect on next try_connect tick
                state.reconnect_cooldown = 2.0;
                state.reconnect_attempts = 1;
                *status = ConnectionStatus::Reconnecting;
                log.push("WS: disconnected, reconnecting...".into());
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
        let _ = (&mut pending, &mut visual, &mut log, &mut status);
    }
}
