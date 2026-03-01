use agent_world_core::{AgentStatus, WorldEvent};
use bevy::prelude::*;
use crate::components::{AgentLabel, AgentSprite, ArtifactSprite, EnergyBar, HealthBar, MovementTarget, PortalPhase, PortalTransition, StatusRing, TrailDot};
use crate::plugins::events::PendingEvents;
use crate::plugins::world::{PortalIndex, RoomIndex};

pub struct AgentPlugin;

/// Resource mapping agent_id → Entity for fast lookups.
#[derive(Resource, Default)]
pub struct AgentIndex {
    pub map: std::collections::HashMap<String, Entity>,
}

impl Plugin for AgentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AgentIndex>()
            .add_systems(Update, (
                handle_agent_events,
                handle_portal_transitions,
                animate_portal_warp,
                move_agents,
                emit_trail_dots,
                fade_trail_dots,
                artifacts_follow_owners,
                update_status_visuals,
            ).chain());
    }
}

/// Drain PendingEvents and spawn/move/update agents accordingly.
fn handle_agent_events(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut pending: ResMut<PendingEvents>,
    mut agents: Query<(Entity, &mut AgentSprite, &mut MovementTarget)>,
    mut index: ResMut<AgentIndex>,
    room_index: Res<RoomIndex>,
    portal_index: Res<PortalIndex>,
) {
    let events: Vec<WorldEvent> = pending.queue.drain(..).collect();

    for event in events {
        match event {
            WorldEvent::AgentSpawn(agent) => {
                // Skip if already spawned (idempotent)
                if index.map.contains_key(&agent.id) {
                    continue;
                }

                let color = Color::srgba_u8(
                    agent.sprite.color[0],
                    agent.sprite.color[1],
                    agent.sprite.color[2],
                    agent.sprite.color[3],
                );

                let agent_mesh = meshes.add(Rectangle::new(32.0, 32.0));
                let mat = materials.add(ColorMaterial::from_color(color));

                // Status ring (slightly larger, behind agent)
                let ring_mesh = meshes.add(Rectangle::new(40.0, 40.0));
                let ring_mat = materials.add(ColorMaterial::from_color(
                    Color::srgba(0.5, 0.5, 0.5, 0.3),
                ));

                let agent_entity = commands
                    .spawn((
                        Mesh2d(agent_mesh),
                        MeshMaterial2d(mat),
                        Transform::from_xyz(agent.position.x, agent.position.y, 1.0),
                        AgentSprite {
                            agent_id: agent.id.clone(),
                            name: agent.name.clone(),
                            role: agent.role.clone(),
                            status: agent.status,
                            last_tool: None,
                            last_thought: None,
                            tool_count: 0,
                        },
                        MovementTarget {
                            target: Vec2::new(agent.position.x, agent.position.y),
                            speed: 80.0,
                        },
                    ))
                    .id();

                index.map.insert(agent.id.clone(), agent_entity);

                // Status ring as child
                commands.spawn((
                    Mesh2d(ring_mesh),
                    MeshMaterial2d(ring_mat),
                    Transform::from_xyz(0.0, 0.0, -0.1),
                    StatusRing { base_scale: 1.0 },
                    ChildOf(agent_entity),
                ));

                // Name label as child
                commands.spawn((
                    Text2d::new(&agent.name),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Transform::from_xyz(0.0, 26.0, 2.0),
                    AgentLabel,
                    ChildOf(agent_entity),
                ));

                // Health bar (green) — full width = 30px
                let bar_bg = meshes.add(Rectangle::new(32.0, 3.0));
                let bar_bg_mat = materials.add(ColorMaterial::from_color(
                    Color::srgba(0.2, 0.2, 0.2, 0.5),
                ));
                let health_fill = meshes.add(Rectangle::new(30.0, 3.0));
                let health_mat = materials.add(ColorMaterial::from_color(
                    Color::srgba(0.2, 0.8, 0.2, 0.8),
                ));

                commands.spawn((
                    Mesh2d(bar_bg.clone()),
                    MeshMaterial2d(bar_bg_mat.clone()),
                    Transform::from_xyz(0.0, -22.0, 1.5),
                    ChildOf(agent_entity),
                ));
                commands.spawn((
                    Mesh2d(health_fill),
                    MeshMaterial2d(health_mat),
                    Transform::from_xyz(0.0, -22.0, 1.6),
                    HealthBar,
                    ChildOf(agent_entity),
                ));

                // Energy bar (blue) below health
                let energy_fill = meshes.add(Rectangle::new(30.0, 3.0));
                let energy_mat = materials.add(ColorMaterial::from_color(
                    Color::srgba(0.2, 0.4, 0.9, 0.8),
                ));

                commands.spawn((
                    Mesh2d(bar_bg),
                    MeshMaterial2d(bar_bg_mat),
                    Transform::from_xyz(0.0, -27.0, 1.5),
                    ChildOf(agent_entity),
                ));
                commands.spawn((
                    Mesh2d(energy_fill),
                    MeshMaterial2d(energy_mat),
                    Transform::from_xyz(0.0, -27.0, 1.6),
                    EnergyBar,
                    ChildOf(agent_entity),
                ));

                // Spawn flash effect
                let flash_mesh = meshes.add(Circle::new(30.0));
                let flash_mat = materials.add(ColorMaterial::from_color(
                    Color::srgba(
                        agent.sprite.color[0] as f32 / 255.0,
                        agent.sprite.color[1] as f32 / 255.0,
                        agent.sprite.color[2] as f32 / 255.0,
                        0.6,
                    ),
                ));
                commands.spawn((
                    Mesh2d(flash_mesh),
                    MeshMaterial2d(flash_mat),
                    Transform::from_xyz(agent.position.x, agent.position.y, 0.9),
                    crate::components::ToolEffect {
                        lifetime: 0.0,
                        max_lifetime: 0.6,
                        success: Some(true),
                    },
                ));
            }
            WorldEvent::AgentMove { ref agent_id, to } => {
                for (_, sprite, mut target) in &mut agents {
                    if sprite.agent_id == *agent_id {
                        target.target = Vec2::new(to.x, to.y);
                    }
                }
            }
            WorldEvent::AgentDespawn { ref agent_id } => {
                if let Some(entity) = index.map.remove(agent_id) {
                    // Spawn a fade-out effect at the agent's position
                    if let Ok((_, sprite, _)) = agents.get(entity) {
                        let _ = sprite; // just need entity
                    }
                    commands.entity(entity).despawn();
                }
            }
            WorldEvent::AgentStatusChange { ref agent_id, status, .. } => {
                for (_, mut sprite, _) in &mut agents {
                    if sprite.agent_id == *agent_id {
                        sprite.status = status;
                    }
                }
            }
            WorldEvent::AgentError { ref agent_id, .. } => {
                for (_, mut sprite, _) in &mut agents {
                    if sprite.agent_id == *agent_id {
                        sprite.status = AgentStatus::Error;
                    }
                }
            }
            WorldEvent::RoomEnter { ref agent_id, ref room_id } => {
                // Find the nearest portal that targets this room
                // and initiate a portal transition
                if let Some(entity) = index.map.get(agent_id).copied() {
                    // Find a portal whose target_room matches room_id (case-insensitive)
                    let room_lower = room_id.to_lowercase();
                    let source_portal = portal_index.portals.iter()
                        .find(|(_, info)| info.target_room.to_lowercase() == room_lower);

                    if let Some((_, portal_info)) = source_portal {
                        // Compute destination: target room position + target local offset
                        let dest = if let Some(room_pos) = room_index.positions.get(&room_lower) {
                            *room_pos + portal_info.target_local_pos
                        } else {
                            // Fallback: try to find room by name match
                            room_index.positions.iter()
                                .find(|(k, _)| k.to_lowercase() == room_lower)
                                .map(|(_, pos)| *pos + portal_info.target_local_pos)
                                .unwrap_or(portal_info.world_pos)
                        };

                        // Start portal transition: move to source portal first
                        commands.entity(entity).insert(PortalTransition {
                            phase: PortalPhase::Approaching,
                            timer: 0.0,
                            source_pos: portal_info.world_pos,
                            dest_pos: dest,
                            phase_duration: 0.4,
                        });

                        // Override movement target to go to portal
                        if let Ok((_, _, mut target)) = agents.get_mut(entity) {
                            target.target = portal_info.world_pos;
                            target.speed = 200.0; // Move fast toward portal
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// Smooth movement toward target position (agents and artifacts).
fn move_agents(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &MovementTarget)>,
) {
    for (mut transform, target) in &mut query {
        let current = Vec2::new(transform.translation.x, transform.translation.y);
        let direction = target.target - current;
        let distance = direction.length();

        if distance > 1.0 {
            let velocity = direction.normalize() * target.speed * time.delta_secs();
            if velocity.length() > distance {
                transform.translation.x = target.target.x;
                transform.translation.y = target.target.y;
            } else {
                transform.translation.x += velocity.x;
                transform.translation.y += velocity.y;
            }
        }
    }
}

/// Emit trail dots behind moving agents.
fn emit_trail_dots(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    agents: Query<(&AgentSprite, &Transform, &MovementTarget)>,
    mut timer: Local<f32>,
    time: Res<Time>,
) {
    *timer += time.delta_secs();
    if *timer < 0.3 {
        return; // emit a dot every 0.3s
    }
    *timer = 0.0;

    let dot_mesh = meshes.add(Circle::new(3.0));

    for (sprite, transform, target) in &agents {
        let pos = transform.translation.truncate();
        let dist = (target.target - pos).length();
        if dist < 2.0 {
            continue; // not moving
        }

        let color = match sprite.status {
            AgentStatus::Acting => Color::srgba(0.2, 0.9, 0.3, 0.15),
            AgentStatus::Thinking => Color::srgba(0.3, 0.5, 1.0, 0.12),
            _ => Color::srgba(0.5, 0.5, 0.6, 0.10),
        };

        let dot_mat = materials.add(ColorMaterial::from_color(color));
        commands.spawn((
            Mesh2d(dot_mesh.clone()),
            MeshMaterial2d(dot_mat),
            Transform::from_xyz(pos.x, pos.y, 0.3),
            TrailDot {
                lifetime: 0.0,
                max_lifetime: 4.0,
            },
        ));
    }
}

/// Fade and despawn trail dots.
fn fade_trail_dots(
    mut commands: Commands,
    time: Res<Time>,
    mut dots: Query<(Entity, &mut TrailDot, &MeshMaterial2d<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, mut dot, mat_handle) in &mut dots {
        dot.lifetime += time.delta_secs();
        let frac = dot.lifetime / dot.max_lifetime;

        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            let base = mat.color.to_srgba();
            mat.color = Color::srgba(base.red, base.green, base.blue, base.alpha * (1.0 - frac));
        }

        if dot.lifetime >= dot.max_lifetime {
            commands.entity(entity).despawn();
        }
    }
}

/// Keep owned artifacts near their owner agent.
fn artifacts_follow_owners(
    agents: Query<(&AgentSprite, &Transform), Without<ArtifactSprite>>,
    mut artifacts: Query<(&ArtifactSprite, &mut MovementTarget)>,
) {
    for (art, mut target) in &mut artifacts {
        if let Some(ref owner_id) = art.owner {
            if let Some((_, agent_tf)) = agents.iter().find(|(s, _)| s.agent_id == *owner_id) {
                target.target = agent_tf.translation.truncate() + Vec2::new(20.0, -10.0);
            }
        }
    }
}

/// Check if approaching agents have reached the source portal.
fn handle_portal_transitions(
    mut agents: Query<(Entity, &Transform, &MovementTarget, &mut PortalTransition)>,
) {
    for (_entity, transform, _target, mut transition) in &mut agents {
        if transition.phase == PortalPhase::Approaching {
            let pos = transform.translation.truncate();
            let dist = (transition.source_pos - pos).length();
            if dist < 10.0 {
                // Arrived at portal — start warp out
                transition.phase = PortalPhase::WarpOut;
                transition.timer = 0.0;
            }
        }
    }
}

/// Animate portal warp (shrink at source, teleport, grow at destination).
fn animate_portal_warp(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
    mut agents: Query<(Entity, &AgentSprite, &mut Transform, &mut PortalTransition, &mut MovementTarget)>,
) {
    for (entity, _sprite, mut transform, mut transition, mut target) in &mut agents {
        transition.timer += time.delta_secs();
        let t = (transition.timer / transition.phase_duration).min(1.0);

        match transition.phase {
            PortalPhase::Approaching => {
                // Handled by handle_portal_transitions
            }
            PortalPhase::WarpOut => {
                // Shrink + spin at source portal
                let scale = 1.0 - t;
                transform.scale = Vec3::splat(scale.max(0.05));
                transform.rotation = Quat::from_rotation_z(t * std::f32::consts::TAU);

                if t >= 1.0 {
                    // Teleport to destination
                    transform.translation.x = transition.dest_pos.x;
                    transform.translation.y = transition.dest_pos.y;
                    target.target = transition.dest_pos;

                    // Spawn warp flash at source
                    let flash_mesh = meshes.add(Circle::new(20.0));
                    let flash_mat = materials.add(ColorMaterial::from_color(
                        Color::srgba(0.4, 0.2, 0.9, 0.8),
                    ));
                    commands.spawn((
                        Mesh2d(flash_mesh.clone()),
                        MeshMaterial2d(flash_mat),
                        Transform::from_xyz(transition.source_pos.x, transition.source_pos.y, 3.0),
                        crate::components::ToolEffect {
                            lifetime: 0.0,
                            max_lifetime: 0.5,
                            success: Some(true),
                        },
                    ));

                    // Spawn warp flash at destination
                    let dest_mat = materials.add(ColorMaterial::from_color(
                        Color::srgba(0.4, 0.2, 0.9, 0.8),
                    ));
                    commands.spawn((
                        Mesh2d(flash_mesh),
                        MeshMaterial2d(dest_mat),
                        Transform::from_xyz(transition.dest_pos.x, transition.dest_pos.y, 3.0),
                        crate::components::ToolEffect {
                            lifetime: 0.0,
                            max_lifetime: 0.5,
                            success: Some(true),
                        },
                    ));

                    // Start warp in
                    transition.phase = PortalPhase::WarpIn;
                    transition.timer = 0.0;
                }
            }
            PortalPhase::WarpIn => {
                // Grow + unwind at destination
                let scale = t;
                transform.scale = Vec3::splat(scale.max(0.05));
                transform.rotation = Quat::from_rotation_z((1.0 - t) * std::f32::consts::TAU);

                if t >= 1.0 {
                    // Done — restore normal state
                    transform.scale = Vec3::ONE;
                    transform.rotation = Quat::IDENTITY;
                    target.speed = 80.0; // Reset to normal speed
                    commands.entity(entity).remove::<PortalTransition>();
                }
            }
        }
    }
}

/// Update status ring color and pulse based on agent status.
/// Also applies a gentle breathing animation to idle agents.
fn update_status_visuals(
    time: Res<Time>,
    mut agents: Query<(&AgentSprite, &Children, &mut Transform)>,
    mut rings: Query<(&mut Transform, &StatusRing, &MeshMaterial2d<ColorMaterial>), Without<AgentSprite>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let t = time.elapsed_secs();

    for (sprite, children, mut agent_tf) in &mut agents {
        // Gentle breathing animation for idle/thinking agents
        let breathe = match sprite.status {
            AgentStatus::Idle => 1.0 + (t * 1.5).sin() * 0.03,
            AgentStatus::Thinking => 1.0 + (t * 2.0).sin() * 0.02,
            _ => 1.0,
        };
        agent_tf.scale = Vec3::splat(breathe);

        for child in children.iter() {
            if let Ok((mut ring_transform, ring, mat_handle)) = rings.get_mut(child) {
                let (color, pulse) = match sprite.status {
                    AgentStatus::Idle => (Color::srgba(0.4, 0.4, 0.5, 0.2), false),
                    AgentStatus::Thinking => (Color::srgba(0.3, 0.5, 1.0, 0.5), true),
                    AgentStatus::Acting => (Color::srgba(0.2, 0.9, 0.3, 0.5), false),
                    AgentStatus::Waiting => (Color::srgba(0.9, 0.7, 0.1, 0.4), true),
                    AgentStatus::Error => (Color::srgba(1.0, 0.2, 0.2, 0.6), true),
                    AgentStatus::Paused => (Color::srgba(0.5, 0.5, 0.5, 0.3), false),
                };

                if let Some(mat) = materials.get_mut(&mat_handle.0) {
                    mat.color = color;
                }

                let scale = if pulse {
                    ring.base_scale + (t * 3.0).sin() * 0.15
                } else {
                    ring.base_scale
                };
                ring_transform.scale = Vec3::splat(scale);
            }
        }
    }
}
