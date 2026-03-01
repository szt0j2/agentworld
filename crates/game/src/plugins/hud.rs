use agent_world_core::AgentStatus;
use bevy::prelude::*;
use crate::components::{AgentSprite, MinimapDot, MinimapPanel, MinimapRoom};
use crate::plugins::world::RoomIndex;
use crate::plugins::adapter::ConnectionStatus;
use crate::plugins::camera::CameraFollow;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EventLog>()
            .init_resource::<InspectorState>()
            .add_systems(Startup, spawn_hud)
            .add_systems(Update, (
                update_agent_roster,
                handle_roster_clicks,
                update_event_log_display,
                update_inspector_panel,
                update_connection_status,
                update_minimap,
                toggle_help_overlay,
            ));
    }
}

/// Which agent (if any) is selected for the inspector.
#[derive(Resource, Default)]
pub struct InspectorState {
    pub selected: Option<String>,
}

/// Stores recent event descriptions for the event log panel.
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
struct HudRoot;

#[derive(Component)]
struct AgentRosterPanel;

#[derive(Component)]
struct AgentRosterEntry {
    agent_id: String,
}

#[derive(Component)]
struct EventLogPanel;

#[derive(Component)]
struct EventLogText;

#[derive(Component)]
struct InspectorPanel;

#[derive(Component)]
struct InspectorText;

#[derive(Component)]
struct ConnectionStatusDot;

#[derive(Component)]
struct HelpOverlay;

fn spawn_hud(mut commands: Commands) {
    // Root container — full screen overlay
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Stretch,
            ..default()
        },
        HudRoot,
    )).with_children(|parent| {
        // Left sidebar — Agent Roster
        parent.spawn((
            Node {
                width: Val::Px(180.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.12, 0.85)),
            AgentRosterPanel,
        )).with_children(|panel| {
            // Title row with connection status dot
            panel.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            )).with_children(|title_row| {
                title_row.spawn((
                    Text::new("AGENTS"),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.7, 0.7, 0.9, 1.0)),
                ));
                // Connection status dot
                title_row.spawn((
                    Node {
                        width: Val::Px(8.0),
                        height: Val::Px(8.0),
                        margin: UiRect::left(Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
                    ConnectionStatusDot,
                ));
            });
        });

        // Center area (game canvas + inspector at bottom)
        parent.spawn((
            Node {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::End,
                align_items: AlignItems::Center,
                ..default()
            },
        )).with_children(|center| {
            // Inspector panel — hidden by default, shown when agent selected
            center.spawn((
                Node {
                    width: Val::Px(360.0),
                    min_height: Val::Px(80.0),
                    max_height: Val::Px(200.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(10.0)),
                    margin: UiRect::bottom(Val::Px(10.0)),
                    display: Display::None,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.06, 0.06, 0.15, 0.92)),
                InspectorPanel,
            )).with_children(|panel| {
                panel.spawn((
                    Text::new(""),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.8, 0.85, 0.95, 1.0)),
                    InspectorText,
                ));
            });
        });

        // Right side — Event Log
        parent.spawn((
            Node {
                width: Val::Px(220.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.12, 0.85)),
            EventLogPanel,
        )).with_children(|panel| {
            // Title
            panel.spawn((
                Text::new("EVENT LOG"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgba(0.7, 0.7, 0.9, 1.0)),
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            ));

            // Log text
            panel.spawn((
                Text::new("Waiting for events..."),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(Color::srgba(0.6, 0.6, 0.7, 0.9)),
                EventLogText,
            ));
        });
    });

    // Help overlay — initially hidden, toggled with H key
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(60.0),
            right: Val::Px(230.0),
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

    // Minimap overlay — bottom area, shows all rooms + agent dots (scales for multi-team)
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(40.0),
            left: Val::Px(190.0),
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

/// Team header in the roster sidebar.
#[derive(Component)]
struct TeamRosterHeader {
    team_name: String,
}

/// Extract team name from agent_id format "team/agent".
/// Agents without a "/" are grouped under "agents".
fn agent_team(agent_id: &str) -> &str {
    if agent_id.contains('/') {
        agent_id.split('/').next().unwrap_or("agents")
    } else {
        "agents"
    }
}

/// Update the agent roster grouped by team.
fn update_agent_roster(
    mut commands: Commands,
    agents: Query<&AgentSprite>,
    existing_entries: Query<(Entity, &AgentRosterEntry)>,
    existing_headers: Query<(Entity, &TeamRosterHeader)>,
    roster_panel: Query<Entity, With<AgentRosterPanel>>,
    follow: Res<CameraFollow>,
) {
    let Ok(panel_entity) = roster_panel.single() else { return };

    let agent_ids: Vec<String> = agents.iter().map(|a| a.agent_id.clone()).collect();
    let existing_ids: Vec<String> = existing_entries.iter().map(|(_, e)| e.agent_id.clone()).collect();
    let existing_team_names: Vec<String> = existing_headers.iter().map(|(_, h)| h.team_name.clone()).collect();

    // Collect agents grouped by team
    let mut teams: std::collections::BTreeMap<String, Vec<&AgentSprite>> = std::collections::BTreeMap::new();
    for agent in agents.iter() {
        let team = agent_team(&agent.agent_id).to_string();
        teams.entry(team).or_default().push(agent);
    }

    // Spawn team headers and agent entries
    for (team_name, team_agents) in &teams {
        // Add team header if missing
        if !existing_team_names.contains(team_name) {
            commands.spawn((
                Node {
                    margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(6.0), Val::Px(2.0)),
                    ..default()
                },
                TeamRosterHeader { team_name: team_name.clone() },
                ChildOf(panel_entity),
            )).with_children(|parent| {
                parent.spawn((
                    Text::new(format!("[{}]", team_name.to_uppercase())),
                    TextFont { font_size: 10.0, ..default() },
                    TextColor(Color::srgba(0.5, 0.7, 1.0, 0.8)),
                ));
            });
        }

        for agent in team_agents {
            if !existing_ids.contains(&agent.agent_id) {
                let status_color = status_to_color(agent.status);
                let is_followed = follow.target.as_ref() == Some(&agent.agent_id);

                let entry = commands.spawn((
                    Button,
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        margin: UiRect::bottom(Val::Px(3.0)),
                        padding: UiRect::new(Val::Px(8.0), Val::Px(4.0), Val::Px(3.0), Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(if is_followed {
                        Color::srgba(0.2, 0.2, 0.4, 0.5)
                    } else {
                        Color::srgba(0.1, 0.1, 0.15, 0.3)
                    }),
                    AgentRosterEntry {
                        agent_id: agent.agent_id.clone(),
                    },
                    ChildOf(panel_entity),
                )).id();

                // Status dot
                commands.spawn((
                    Node {
                        width: Val::Px(8.0),
                        height: Val::Px(8.0),
                        margin: UiRect::right(Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(status_color),
                    ChildOf(entry),
                ));

                // Agent name + role
                commands.spawn((
                    Text::new(format!("{} ({})", agent.name, agent.role)),
                    TextFont { font_size: 11.0, ..default() },
                    TextColor(Color::srgba(0.85, 0.85, 0.95, 1.0)),
                    ChildOf(entry),
                ));
            }
        }
    }

    // Update existing entries (follow highlight, status color)
    for (entry_entity, roster_entry) in &existing_entries {
        if let Some(agent) = agents.iter().find(|a| a.agent_id == roster_entry.agent_id) {
            let is_followed = follow.target.as_ref() == Some(&agent.agent_id);
            commands.entity(entry_entity).insert(
                BackgroundColor(if is_followed {
                    Color::srgba(0.2, 0.2, 0.4, 0.5)
                } else {
                    Color::NONE
                }),
            );
        }

        if !agent_ids.contains(&roster_entry.agent_id) {
            commands.entity(entry_entity).despawn();
        }
    }

    // Remove team headers for teams with no agents
    let active_teams: Vec<String> = teams.keys().cloned().collect();
    for (header_entity, header) in &existing_headers {
        if !active_teams.contains(&header.team_name) {
            commands.entity(header_entity).despawn();
        }
    }
}

/// Handle clicks on agent roster entries to follow and inspect.
fn handle_roster_clicks(
    entries: Query<(&Interaction, &AgentRosterEntry), Changed<Interaction>>,
    mut follow: ResMut<CameraFollow>,
    mut inspector: ResMut<InspectorState>,
) {
    for (interaction, entry) in &entries {
        if *interaction == Interaction::Pressed {
            if follow.target.as_ref() == Some(&entry.agent_id) {
                follow.target = None;
                inspector.selected = None;
            } else {
                follow.target = Some(entry.agent_id.clone());
                inspector.selected = Some(entry.agent_id.clone());
            }
        }
    }
}

/// Update the event log text.
fn update_event_log_display(
    log: Res<EventLog>,
    mut text_query: Query<&mut Text, With<EventLogText>>,
) {
    if !log.is_changed() {
        return;
    }

    let display = if log.entries.is_empty() {
        "Waiting for events...".to_string()
    } else {
        log.entries
            .iter()
            .rev()
            .take(15)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    };

    for mut text in &mut text_query {
        **text = display.clone();
    }
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

/// Update the inspector panel to show details of the selected agent.
fn update_inspector_panel(
    inspector: Res<InspectorState>,
    agents: Query<&AgentSprite>,
    mut panel_query: Query<&mut Node, With<InspectorPanel>>,
    mut text_query: Query<&mut Text, With<InspectorText>>,
) {
    let Ok(mut panel_node) = panel_query.single_mut() else { return };
    let Ok(mut text) = text_query.single_mut() else { return };

    match &inspector.selected {
        None => {
            panel_node.display = Display::None;
        }
        Some(agent_id) => {
            panel_node.display = Display::Flex;

            if let Some(agent) = agents.iter().find(|a| &a.agent_id == agent_id) {
                let status_str = format!("{:?}", agent.status);
                let tool_str = agent.last_tool.as_deref().unwrap_or("none");
                let thought_str = agent.last_thought.as_deref().unwrap_or("");
                let thought_display = if thought_str.len() > 40 {
                    format!("{}...", &thought_str[..37])
                } else {
                    thought_str.to_string()
                };
                let mut info = format!(
                    "[ {} ]\nRole: {}  |  Status: {}\nTools used: {}  |  Last: {}",
                    agent.name, agent.role, status_str, agent.tool_count, tool_str,
                );
                if !thought_display.is_empty() {
                    info.push_str(&format!("\nThinking: {thought_display}"));
                }
                **text = info;
            } else {
                **text = format!("Agent {} not found", agent_id);
            }
        }
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
