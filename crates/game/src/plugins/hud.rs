use agent_world_core::AgentStatus;
use bevy::prelude::*;
use crate::components::{AgentSprite, MinimapDot, MinimapPanel, MinimapRoom};
use crate::plugins::world::RoomIndex;
use crate::plugins::adapter::ConnectionStatus;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EventLog>()
            .init_resource::<InspectorState>()
            .add_systems(Startup, spawn_hud)
            .add_systems(Update, (
                update_connection_status,
                update_minimap,
                toggle_help_overlay,
            ));
    }
}

/// Which agent (if any) is selected for the inspector (kept for Bevy-side camera follow).
#[derive(Resource, Default)]
pub struct InspectorState {
    pub selected: Option<String>,
}

/// Stores recent event descriptions (still fed by adapter + events plugins).
#[derive(Resource, Default)]
pub struct EventLog {
    pub entries: Vec<String>,
}

impl EventLog {
    pub fn push(&mut self, msg: String) {
        self.entries.push(msg);
        if self.entries.len() > 20 {
            self.entries.remove(0);
        }
    }
}

#[derive(Component)]
struct ConnectionStatusDot;

#[derive(Component)]
struct HelpOverlay;

fn spawn_hud(mut commands: Commands) {
    // Connection status dot — top-left corner
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            left: Val::Px(8.0),
            width: Val::Px(10.0),
            height: Val::Px(10.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
        ConnectionStatusDot,
    ));

    // Help overlay — toggled with H key
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(60.0),
            right: Val::Px(270.0),
            width: Val::Px(200.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(10.0)),
            display: Display::None,
            ..default()
        },
        BackgroundColor(Color::srgba(0.06, 0.06, 0.15, 0.92)),
        HelpOverlay,
    )).with_children(|panel| {
        let help_text = "\
CONTROLS\n\
\n\
1-9    Follow agent\n\
Esc    Stop following\n\
Scroll Zoom in/out\n\
MMB    Pan camera\n\
H      Toggle help\n\
M      Toggle sound\n\
\n\
Click roster to inspect";

        panel.spawn((
            Text::new(help_text),
            TextFont {
                font_size: 10.0,
                ..default()
            },
            TextColor(Color::srgba(0.7, 0.75, 0.9, 0.9)),
        ));
    });

    // Minimap overlay — bottom area above inspector, shows all rooms + agent dots
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(210.0),
            width: Val::Px(280.0),
            height: Val::Px(70.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            padding: UiRect::all(Val::Px(4.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.03, 0.03, 0.08, 0.85)),
        MinimapPanel,
    ));
}

/// Update connection status dot color.
fn update_connection_status(
    status: Res<ConnectionStatus>,
    mut dots: Query<&mut BackgroundColor, With<ConnectionStatusDot>>,
) {
    if !status.is_changed() {
        return;
    }
    let color = match *status {
        ConnectionStatus::Disconnected => Color::srgb(0.5, 0.5, 0.5),
        ConnectionStatus::Connecting => Color::srgb(0.9, 0.7, 0.1),
        ConnectionStatus::Connected => Color::srgb(0.2, 0.9, 0.3),
        ConnectionStatus::Reconnecting => Color::srgb(0.9, 0.4, 0.1),
    };
    for mut bg in &mut dots {
        *bg = BackgroundColor(color);
    }
}

/// Toggle the help overlay with the H key.
fn toggle_help_overlay(
    keys: Res<ButtonInput<KeyCode>>,
    mut overlay: Query<&mut Node, With<HelpOverlay>>,
) {
    if keys.just_pressed(KeyCode::KeyH) {
        for mut node in &mut overlay {
            node.display = if node.display == Display::None {
                Display::Flex
            } else {
                Display::None
            };
        }
    }
}

/// Update minimap with agent positions as colored dots.
fn update_minimap(
    mut commands: Commands,
    agents: Query<(&AgentSprite, &Transform), Without<MinimapDot>>,
    existing_dots: Query<(Entity, &MinimapDot)>,
    existing_rooms: Query<Entity, With<MinimapRoom>>,
    minimap_panel: Query<Entity, With<MinimapPanel>>,
    room_index: Res<RoomIndex>,
) {
    let Ok(panel_entity) = minimap_panel.single() else { return };

    // Dynamic world bounds from actual room positions
    let room_half = 240.0_f32;
    let (world_min_x, world_max_x, world_min_y, world_max_y) = if room_index.positions.is_empty() {
        (-240.0_f32, 1640.0_f32, -240.0_f32, 240.0_f32)
    } else {
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;
        for pos in room_index.positions.values() {
            min_x = min_x.min(pos.x - room_half);
            max_x = max_x.max(pos.x + room_half);
            min_y = min_y.min(pos.y - room_half);
            max_y = max_y.max(pos.y + room_half);
        }
        // Add padding
        (min_x - 50.0, max_x + 50.0, min_y - 50.0, max_y + 50.0)
    };

    let map_w = 272.0_f32;
    let map_h = 62.0_f32;

    let to_minimap = |wx: f32, wy: f32| -> (f32, f32) {
        let nx = ((wx - world_min_x) / (world_max_x - world_min_x)).clamp(0.0, 1.0);
        let ny = ((wy - world_min_y) / (world_max_y - world_min_y)).clamp(0.0, 1.0);
        (nx * map_w, (1.0 - ny) * map_h)
    };

    // Draw room outlines (once, when rooms appear)
    if existing_rooms.is_empty() && !room_index.positions.is_empty() {
        for (_room_id, &room_pos) in &room_index.positions {
            let (left, top) = to_minimap(room_pos.x - room_half, room_pos.y + room_half);
            let (right, bottom) = to_minimap(room_pos.x + room_half, room_pos.y - room_half);
            let w = (right - left).max(2.0);
            let h = (bottom - top).max(2.0);

            commands.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(left),
                    top: Val::Px(top),
                    width: Val::Px(w),
                    height: Val::Px(h),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.15, 0.15, 0.25, 0.4)),
                MinimapRoom,
                ChildOf(panel_entity),
            ));
        }
    }

    // Remove dots for agents that no longer exist
    let agent_ids: Vec<String> = agents.iter().map(|(a, _)| a.agent_id.clone()).collect();
    for (dot_entity, dot) in &existing_dots {
        if !agent_ids.contains(&dot.agent_id) {
            commands.entity(dot_entity).despawn();
        }
    }

    let existing_ids: Vec<String> = existing_dots.iter().map(|(_, d)| d.agent_id.clone()).collect();

    for (agent, transform) in &agents {
        let (dot_x, dot_y) = to_minimap(transform.translation.x, transform.translation.y);
        let color = status_to_color(agent.status);

        if existing_ids.contains(&agent.agent_id) {
            for (dot_entity, dot) in &existing_dots {
                if dot.agent_id == agent.agent_id {
                    commands.entity(dot_entity).insert(Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(dot_x - 3.0),
                        top: Val::Px(dot_y - 3.0),
                        width: Val::Px(6.0),
                        height: Val::Px(6.0),
                        ..default()
                    });
                    commands.entity(dot_entity).insert(BackgroundColor(color));
                }
            }
        } else {
            commands.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(dot_x - 3.0),
                    top: Val::Px(dot_y - 3.0),
                    width: Val::Px(6.0),
                    height: Val::Px(6.0),
                    ..default()
                },
                BackgroundColor(color),
                MinimapDot {
                    agent_id: agent.agent_id.clone(),
                },
                ChildOf(panel_entity),
            ));
        }
    }
}

fn status_to_color(status: AgentStatus) -> Color {
    match status {
        AgentStatus::Idle => Color::srgb(0.5, 0.5, 0.6),
        AgentStatus::Thinking => Color::srgb(0.3, 0.5, 1.0),
        AgentStatus::Acting => Color::srgb(0.2, 0.9, 0.3),
        AgentStatus::Waiting => Color::srgb(0.9, 0.7, 0.1),
        AgentStatus::Error => Color::srgb(1.0, 0.2, 0.2),
        AgentStatus::Paused => Color::srgb(0.5, 0.5, 0.5),
    }
}
