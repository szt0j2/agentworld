use agent_world_core::AgentStatus;
use bevy::prelude::*;
use crate::components::AgentSprite;
use crate::plugins::adapter::ConnectionStatus;
use crate::plugins::camera::CameraFollow;

/// Number of activity bars in the timeline.
const TIMELINE_BARS: usize = 40;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EventLog>()
            .init_resource::<InspectorState>()
            .init_resource::<ActivityTimeline>()
            .add_systems(Startup, spawn_hud)
            .add_systems(Update, (
                update_agent_roster,
                handle_roster_clicks,
                update_event_log_display,
                update_inspector_panel,
                update_connection_status,
                update_activity_timeline,
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

/// Tracks activity over time bins for the timeline bar.
#[derive(Resource)]
pub struct ActivityTimeline {
    /// Activity count per bin (ring buffer).
    bins: [u32; TIMELINE_BARS],
    /// Current bin index.
    current_bin: usize,
    /// Timer for advancing bins.
    timer: f32,
    /// Seconds per bin.
    bin_duration: f32,
}

impl Default for ActivityTimeline {
    fn default() -> Self {
        Self {
            bins: [0; TIMELINE_BARS],
            current_bin: 0,
            timer: 0.0,
            bin_duration: 2.0, // each bar = 2 seconds of activity
        }
    }
}

impl ActivityTimeline {
    pub fn record_event(&mut self) {
        self.bins[self.current_bin] = self.bins[self.current_bin].saturating_add(1);
    }

    fn advance(&mut self, dt: f32) {
        self.timer += dt;
        while self.timer >= self.bin_duration {
            self.timer -= self.bin_duration;
            self.current_bin = (self.current_bin + 1) % TIMELINE_BARS;
            self.bins[self.current_bin] = 0;
        }
    }
}

#[derive(Component)]
struct TimelinePanel;

#[derive(Component)]
struct TimelineBar {
    index: usize,
}

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

        // Center area (game canvas + inspector + timeline at bottom)
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
                    margin: UiRect::bottom(Val::Px(4.0)),
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

            // Activity timeline — heartbeat bar at bottom
            center.spawn((
                Node {
                    width: Val::Percent(80.0),
                    max_width: Val::Px(600.0),
                    height: Val::Px(24.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::End,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(1.0),
                    padding: UiRect::horizontal(Val::Px(4.0)),
                    margin: UiRect::bottom(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.04, 0.04, 0.10, 0.7)),
                TimelinePanel,
            )).with_children(|panel| {
                for i in 0..TIMELINE_BARS {
                    panel.spawn((
                        Node {
                            width: Val::Px(10.0),
                            height: Val::Px(2.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.3, 0.4, 0.7, 0.3)),
                        TimelineBar { index: i },
                    ));
                }
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

/// Update the activity timeline bars.
fn update_activity_timeline(
    time: Res<Time>,
    mut timeline: ResMut<ActivityTimeline>,
    mut bars: Query<(&mut Node, &mut BackgroundColor, &TimelineBar)>,
) {
    timeline.advance(time.delta_secs());

    // Find max activity for scaling
    let max_activity = timeline.bins.iter().copied().max().unwrap_or(1).max(1);

    for (mut node, mut bg, bar) in &mut bars {
        // Map bar index to ring buffer position (oldest → newest left to right)
        let bin_idx = (timeline.current_bin + 1 + bar.index) % TIMELINE_BARS;
        let activity = timeline.bins[bin_idx];
        let normalized = activity as f32 / max_activity as f32;

        // Height: min 2px, max 20px
        let height = 2.0 + normalized * 18.0;
        node.height = Val::Px(height);

        // Color: dim blue → bright cyan based on activity
        let is_current = bin_idx == timeline.current_bin;
        let alpha = if is_current { 0.9 } else { 0.3 + normalized * 0.5 };
        let green = 0.4 + normalized * 0.5;
        let blue = 0.7 + normalized * 0.3;
        *bg = BackgroundColor(Color::srgba(0.2, green, blue, alpha));
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
