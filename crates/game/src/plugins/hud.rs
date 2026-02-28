use agent_world_core::AgentStatus;
use bevy::prelude::*;
use crate::components::AgentSprite;
use crate::plugins::camera::CameraFollow;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EventLog>()
            .add_systems(Startup, spawn_hud)
            .add_systems(Update, (
                update_agent_roster,
                update_event_log_display,
            ));
    }
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
            // Title
            panel.spawn((
                Text::new("AGENTS"),
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
        });

        // Spacer (game canvas area)
        parent.spawn((
            Node {
                flex_grow: 1.0,
                ..default()
            },
    
        ));

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
                    Color::NONE
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
