//! Procedural sound effects via Web Audio API.
//!
//! All sounds are generated in-browser using oscillators + gain envelopes.
//! No audio files needed. Chrome autoplay policy is handled by creating
//! the AudioContext on the first user interaction.

use bevy::prelude::*;
use crate::plugins::events::{PendingEvents, PendingVisualEvents};
use agent_world_core::{AgentStatus, WorldEvent};

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SoundState>()
            .add_systems(Update, (
                init_audio_on_interaction,
                play_event_sounds,
            ).chain());
    }
}

/// Tracks audio context state. Uses NonSend because web_sys types are !Send.
#[derive(Resource, Default)]
struct SoundState {
    /// Set to true once the user has interacted (click/key) and AudioContext is created.
    initialized: bool,
    /// Mute toggle.
    muted: bool,
}

/// Initialize AudioContext on first user interaction (Chrome autoplay policy).
fn init_audio_on_interaction(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut state: ResMut<SoundState>,
) {
    if state.initialized {
        // Toggle mute with M key
        if keys.just_pressed(KeyCode::KeyM) {
            state.muted = !state.muted;
        }
        return;
    }

    // Any key or mouse press initializes audio
    if keys.get_just_pressed().len() > 0 || mouse.get_just_pressed().len() > 0 {
        #[cfg(target_arch = "wasm32")]
        {
            if create_audio_context() {
                state.initialized = true;
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            state.initialized = true;
        }
    }
}

/// Play sounds for world events.
fn play_event_sounds(
    state: Res<SoundState>,
    pending: Res<PendingEvents>,
    visual: Res<PendingVisualEvents>,
) {
    if !state.initialized || state.muted {
        return;
    }

    for event in &pending.queue {
        match event {
            WorldEvent::AgentSpawn(_) => play_sound(SoundType::Spawn),
            WorldEvent::AgentDespawn { .. } => play_sound(SoundType::Despawn),
            WorldEvent::RoomEnter { .. } => play_sound(SoundType::Portal),
            WorldEvent::AgentStatusChange { status, .. } => {
                if *status == AgentStatus::Error {
                    play_sound(SoundType::Error);
                }
            }
            _ => {}
        }
    }

    for event in &visual.queue {
        match event {
            WorldEvent::AgentUseTool { .. } => play_sound(SoundType::ToolUse),
            WorldEvent::AgentToolResult { success, .. } => {
                if *success {
                    play_sound(SoundType::ToolOk);
                } else {
                    play_sound(SoundType::ToolFail);
                }
            }
            WorldEvent::MessageSend(_) => play_sound(SoundType::Message),
            _ => {}
        }
    }
}

#[derive(Clone, Copy)]
enum SoundType {
    Spawn,
    Despawn,
    Portal,
    ToolUse,
    ToolOk,
    ToolFail,
    Message,
    Error,
}

fn play_sound(sound: SoundType) {
    #[cfg(target_arch = "wasm32")]
    play_web_audio(sound);
    #[cfg(not(target_arch = "wasm32"))]
    let _ = sound;
}

// ── Web Audio API implementation (WASM only) ──

#[cfg(target_arch = "wasm32")]
mod wasm_audio {
    use super::SoundType;
    use web_sys::{AudioContext, OscillatorType};

    thread_local! {
        static AUDIO_CTX: std::cell::RefCell<Option<AudioContext>> = std::cell::RefCell::new(None);
    }

    pub fn create_audio_context() -> bool {
        AUDIO_CTX.with(|ctx| {
            if ctx.borrow().is_some() {
                return true;
            }
            match AudioContext::new() {
                Ok(ac) => {
                    *ctx.borrow_mut() = Some(ac);
                    true
                }
                Err(_) => false,
            }
        })
    }

    pub fn play_web_audio(sound: SoundType) {
        AUDIO_CTX.with(|ctx| {
            let binding = ctx.borrow();
            let Some(ac) = binding.as_ref() else { return };

            // Don't play if context is suspended
            if ac.state() == web_sys::AudioContextState::Suspended {
                let _ = ac.resume();
                return;
            }

            let now = ac.current_time();

            match sound {
                SoundType::Spawn => {
                    // Rising chime: sine 440→880 Hz over 0.2s
                    tone(ac, now, 440.0, 880.0, 0.2, 0.08, OscillatorType::Sine);
                }
                SoundType::Despawn => {
                    // Falling tone: sine 600→200 Hz over 0.3s
                    tone(ac, now, 600.0, 200.0, 0.3, 0.06, OscillatorType::Sine);
                }
                SoundType::Portal => {
                    // Whoosh: noise-like sweep 200→1200→200 Hz
                    tone(ac, now, 200.0, 1200.0, 0.15, 0.05, OscillatorType::Sawtooth);
                    tone(ac, now + 0.15, 1200.0, 200.0, 0.15, 0.04, OscillatorType::Sawtooth);
                }
                SoundType::ToolUse => {
                    // Short click: square 800 Hz for 0.05s
                    tone(ac, now, 800.0, 800.0, 0.05, 0.04, OscillatorType::Square);
                }
                SoundType::ToolOk => {
                    // Success ding: two ascending tones
                    tone(ac, now, 523.0, 523.0, 0.08, 0.06, OscillatorType::Sine);
                    tone(ac, now + 0.1, 659.0, 659.0, 0.08, 0.06, OscillatorType::Sine);
                }
                SoundType::ToolFail => {
                    // Fail buzz: low square 150 Hz
                    tone(ac, now, 150.0, 130.0, 0.15, 0.06, OscillatorType::Square);
                }
                SoundType::Message => {
                    // Soft ping: triangle 660 Hz
                    tone(ac, now, 660.0, 660.0, 0.1, 0.05, OscillatorType::Triangle);
                }
                SoundType::Error => {
                    // Alarm: alternating tones
                    tone(ac, now, 400.0, 400.0, 0.1, 0.08, OscillatorType::Square);
                    tone(ac, now + 0.12, 300.0, 300.0, 0.1, 0.08, OscillatorType::Square);
                }
            }
        });
    }

    /// Play a single oscillator tone with frequency sweep and gain envelope.
    fn tone(
        ac: &AudioContext,
        start: f64,
        freq_start: f64,
        freq_end: f64,
        duration: f64,
        volume: f64,
        wave: OscillatorType,
    ) {
        let Ok(osc) = ac.create_oscillator() else { return };
        let Ok(gain) = ac.create_gain() else { return };

        osc.set_type(wave);
        let _ = osc.frequency().set_value_at_time(freq_start as f32, start);
        let _ = osc.frequency().linear_ramp_to_value_at_time(freq_end as f32, start + duration);

        // Gain envelope: quick attack, sustain, quick release
        let _ = gain.gain().set_value_at_time(0.0, start);
        let _ = gain.gain().linear_ramp_to_value_at_time(volume as f32, start + 0.01);
        let _ = gain.gain().set_value_at_time(volume as f32, start + duration - 0.02);
        let _ = gain.gain().linear_ramp_to_value_at_time(0.0, start + duration);

        let _ = osc.connect_with_audio_node(&gain);
        let _ = gain.connect_with_audio_node(&ac.destination());

        let _ = osc.start_with_when(start);
        let _ = osc.stop_with_when(start + duration + 0.01);
    }
}

#[cfg(target_arch = "wasm32")]
use wasm_audio::{create_audio_context, play_web_audio};
