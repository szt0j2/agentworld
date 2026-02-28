use agent_world_core::AgentStatus;
use bevy::prelude::*;
use crate::components::AgentSprite;
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
        // Don't block picking/interaction with game world

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
}

/// Update the agent roster based on spawned agents.
fn update_agent_roster(
    mut commands: Commands,
    agents: Query<&AgentSprite>,
    existing_entries: Query<(Entity, &AgentRosterEntry)>,
    roster_panel: Query<Entity, With<AgentRosterPanel>>,
    follow: Res<CameraFollow>,
) {
    let Ok(panel_entity) = roster_panel.single() else { return };

    // Check which agents exist
    let agent_ids: Vec<String> = agents.iter().map(|a| a.agent_id.clone()).collect();

    // Check which entries already exist
    let existing_ids: Vec<String> = existing_entries.iter().map(|(_, e)| e.agent_id.clone()).collect();

    // Add missing entries
    for agent in agents.iter() {
        if !existing_ids.contains(&agent.agent_id) {
            let status_color = status_to_color(agent.status);
            let is_followed = follow.target.as_ref() == Some(&agent.agent_id);

            let entry = commands.spawn((
                Button,
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(4.0)),
                    padding: UiRect::all(Val::Px(4.0)),
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
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgba(0.85, 0.85, 0.95, 1.0)),
                ChildOf(entry),
            ));
        }
    }

    // Update existing entries (status color changes)
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

        // Remove entries for despawned agents
        if !agent_ids.contains(&roster_entry.agent_id) {
            commands.entity(entry_entity).despawn();
        }
    }
}

/// Handle clicks on agent roster entries to follow and inspect that agent.
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
