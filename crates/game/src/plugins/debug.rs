use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use crate::components::AgentSprite;
use crate::plugins::adapter::ConnectionStatus;

pub struct DebugPlugin;

#[derive(Component)]
struct FpsText;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(Startup, spawn_fps_counter)
            .add_systems(Update, update_fps_counter);
    }
}

fn spawn_fps_counter(mut commands: Commands) {
    commands.spawn((
        Text::new("FPS: --"),
        TextFont {
            font_size: 12.0,
            ..default()
        },
        TextColor(Color::srgba(0.6, 0.6, 0.6, 0.7)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(4.0),
            left: Val::Px(190.0),
            ..default()
        },
        FpsText,
    ));
}

fn update_fps_counter(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
    agents: Query<&AgentSprite>,
    status: Res<ConnectionStatus>,
) {
    let fps = diagnostics
        .get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    let agent_count = agents.iter().count();

    let conn = match *status {
        ConnectionStatus::Disconnected => "demo",
        ConnectionStatus::Connecting => "connecting...",
        ConnectionStatus::Connected => "live",
        ConnectionStatus::Reconnecting => "reconnecting...",
    };

    for mut text in &mut query {
        **text = format!("FPS:{fps:.0} | {agent_count} agents | {conn}");
    }
}
