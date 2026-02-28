use bevy::prelude::*;

mod components;
mod plugins;

use plugins::{AgentPlugin, DebugPlugin, WorldPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "AgentWorld".to_string(),
                canvas: Some("#bevy-canvas".to_string()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: true,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.18)))
        .add_plugins((WorldPlugin, AgentPlugin, DebugPlugin))
        .run();
}
