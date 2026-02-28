use agent_world_core::{ArtifactKind, WorldEvent};
use bevy::prelude::*;
use crate::components::*;
use crate::plugins::events::PendingVisualEvents;

pub struct VisualsPlugin;

impl Plugin for VisualsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            handle_visual_events,
            animate_thought_bubbles,
            animate_message_projectiles,
            animate_tool_effects,
            animate_artifact_glow,
            animate_connection_lines,
        ));
    }
}

/// Process visual events (thoughts, messages, tool use, artifacts).
fn handle_visual_events(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut pending: ResMut<PendingVisualEvents>,
    mut agents: Query<(&mut AgentSprite, &Transform)>,
    mut artifact_query: Query<(&mut ArtifactSprite, &mut MovementTarget, &mut Transform), Without<AgentSprite>>,
    existing_bubbles: Query<(Entity, &ThoughtBubble)>,
) {
    let events: Vec<WorldEvent> = pending.queue.drain(..).collect();

    for event in events {
        match event {
            WorldEvent::AgentThink { ref agent_id, ref thought } => {
                // Track on agent sprite
                for (mut sprite, _) in &mut agents {
                    if sprite.agent_id == *agent_id {
                        sprite.last_thought = Some(thought.clone());
                    }
                }
                // Find the agent and spawn a thought bubble above them
                if let Some((_, agent_tf)) = agents.iter().find(|(s, _)| s.agent_id == *agent_id) {
                    // Despawn existing bubbles for this agent (debounce)
                    for (bubble_entity, bubble) in &existing_bubbles {
                        if bubble.agent_id == *agent_id {
                            commands.entity(bubble_entity).despawn();
                        }
                    }

                    let pos = agent_tf.translation;

                    // Clean up tool name display
                    let display = format_thought(thought);

                    // Background pill
                    let pill_width = (display.len() as f32 * 6.5 + 16.0).min(280.0);
                    let pill_mesh = meshes.add(Rectangle::new(pill_width, 20.0));
                    let pill_mat = materials.add(ColorMaterial::from_color(
                        Color::srgba(0.1, 0.1, 0.2, 0.8),
                    ));

                    let bubble = commands.spawn((
                        Mesh2d(pill_mesh),
                        MeshMaterial2d(pill_mat),
                        Transform::from_xyz(pos.x, pos.y + 40.0, 4.5),
                        ThoughtBubble {
                            agent_id: agent_id.clone(),
                            lifetime: 0.0,
                            max_lifetime: 3.0,
                        },
                    )).id();

                    // Text on top of pill
                    commands.spawn((
                        Text2d::new(display),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(Color::srgba(0.9, 0.9, 1.0, 0.9)),
                        Transform::from_xyz(0.0, 0.0, 0.5),
                        ChildOf(bubble),
                    ));
                }
            }
            WorldEvent::MessageSend(ref msg) => {
                // Find sender position
                if let Some((_, from_tf)) = agents.iter().find(|(s, _)| s.agent_id == msg.from) {
                    let from_pos = from_tf.translation.truncate();
                    let preview = if msg.content_preview.len() > 20 {
                        format!("{}...", &msg.content_preview[..17])
                    } else {
                        msg.content_preview.clone()
                    };

                    for to_id in &msg.to {
                        // Spawn a projectile mesh that travels to target
                        let proj_mesh = meshes.add(Circle::new(6.0));
                        let proj_mat = materials.add(ColorMaterial::from_color(
                            Color::srgba(1.0, 0.85, 0.2, 0.9),
                        ));

                        commands.spawn((
                            Mesh2d(proj_mesh),
                            MeshMaterial2d(proj_mat),
                            Transform::from_xyz(from_pos.x, from_pos.y, 4.0),
                            MessageProjectile {
                                from_pos,
                                to_agent_id: to_id.clone(),
                                progress: 0.0,
                                speed: 1.5,
                                content_preview: preview.clone(),
                            },
                        ));

                        // Connection line between sender and receiver
                        if let Some((_, to_tf)) = agents.iter().find(|(s, _)| s.agent_id == *to_id) {
                            let to_pos = to_tf.translation.truncate();
                            let mid = (from_pos + to_pos) / 2.0;
                            let diff = to_pos - from_pos;
                            let length = diff.length();
                            let angle = diff.y.atan2(diff.x);

                            let line_mesh = meshes.add(Rectangle::new(length, 1.5));
                            let line_mat = materials.add(ColorMaterial::from_color(
                                Color::srgba(1.0, 0.85, 0.2, 0.2),
                            ));

                            commands.spawn((
                                Mesh2d(line_mesh),
                                MeshMaterial2d(line_mat),
                                Transform::from_xyz(mid.x, mid.y, 0.6)
                                    .with_rotation(Quat::from_rotation_z(angle)),
                                ConnectionLine {
                                    from_agent: msg.from.clone(),
                                    to_agent: to_id.clone(),
                                    lifetime: 0.0,
                                    max_lifetime: 2.0,
                                },
                            ));
                        }
                    }
                }
            }
            WorldEvent::AgentUseTool { ref agent_id, ref tool_id, .. } => {
                // Track on agent sprite
                for (mut sprite, _) in &mut agents {
                    if sprite.agent_id == *agent_id {
                        sprite.last_tool = Some(tool_id.clone());
                        sprite.tool_count += 1;
                    }
                }
                // Flash effect on the agent
                if let Some((_, agent_tf)) = agents.iter().find(|(s, _)| s.agent_id == *agent_id) {
                    let pos = agent_tf.translation;
                    let effect_mesh = meshes.add(Circle::new(24.0));
                    let effect_mat = materials.add(ColorMaterial::from_color(
                        Color::srgba(1.0, 1.0, 0.3, 0.6),
                    ));

                    commands.spawn((
                        Mesh2d(effect_mesh),
                        MeshMaterial2d(effect_mat),
                        Transform::from_xyz(pos.x, pos.y, 3.0),
                        ToolEffect {
                            lifetime: 0.0,
                            max_lifetime: 0.5,
                            success: None,
                        },
                    ));
                }
            }
            WorldEvent::AgentToolResult { ref agent_id, success, .. } => {
                if let Some((_, agent_tf)) = agents.iter().find(|(s, _)| s.agent_id == *agent_id) {
                    let pos = agent_tf.translation;
                    let color = if success {
                        Color::srgba(0.2, 1.0, 0.4, 0.7) // green sparkle
                    } else {
                        Color::srgba(1.0, 0.2, 0.2, 0.7) // red fizzle
                    };
                    let effect_mesh = meshes.add(Circle::new(20.0));
                    let effect_mat = materials.add(ColorMaterial::from_color(color));

                    commands.spawn((
                        Mesh2d(effect_mesh),
                        MeshMaterial2d(effect_mat),
                        Transform::from_xyz(pos.x, pos.y, 3.0),
                        ToolEffect {
                            lifetime: 0.0,
                            max_lifetime: 0.8,
                            success: Some(success),
                        },
                    ));
                }
            }
            WorldEvent::ArtifactCreate(ref artifact) => {
                let color = match artifact.kind {
                    ArtifactKind::Document => Color::srgba(0.9, 0.85, 0.6, 0.9),
                    ArtifactKind::Code => Color::srgba(0.4, 0.9, 0.5, 0.9),
                    ArtifactKind::Data => Color::srgba(0.4, 0.7, 1.0, 0.9),
                    ArtifactKind::Image => Color::srgba(0.9, 0.5, 0.8, 0.9),
                    ArtifactKind::Plan => Color::srgba(0.8, 0.6, 1.0, 0.9),
                    ArtifactKind::MessageBundle => Color::srgba(1.0, 0.8, 0.3, 0.9),
                };

                let art_mesh = meshes.add(Rectangle::new(16.0, 16.0));
                let art_mat = materials.add(ColorMaterial::from_color(color));

                let entity = commands.spawn((
                    Mesh2d(art_mesh),
                    MeshMaterial2d(art_mat),
                    Transform::from_xyz(artifact.position.x, artifact.position.y, 0.8),
                    ArtifactSprite {
                        artifact_id: artifact.id.clone(),
                        name: artifact.name.clone(),
                        kind: artifact.kind.clone(),
                        owner: artifact.owner.clone(),
                        quality: artifact.quality,
                    },
                    MovementTarget {
                        target: Vec2::new(artifact.position.x, artifact.position.y),
                        speed: 60.0,
                    },
                )).id();

                // Name label
                commands.spawn((
                    Text2d::new(&artifact.name),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.8, 0.8, 0.6, 0.8)),
                    Transform::from_xyz(0.0, 14.0, 2.0),
                    ChildOf(entity),
                ));
            }
            WorldEvent::AgentPickUp { ref agent_id, ref artifact_id } => {
                // Move artifact to agent position and mark as owned
                for (mut art, mut target, _) in &mut artifact_query {
                    if art.artifact_id == *artifact_id {
                        art.owner = Some(agent_id.clone());
                        // Find agent position
                        if let Some((_, agent_tf)) = agents.iter().find(|(s, _)| s.agent_id == *agent_id) {
                            target.target = agent_tf.translation.truncate() + Vec2::new(20.0, -10.0);
                        }
                    }
                }
            }
            WorldEvent::AgentDrop { ref artifact_id, position, .. } => {
                for (mut art, mut target, _) in &mut artifact_query {
                    if art.artifact_id == *artifact_id {
                        art.owner = None;
                        target.target = Vec2::new(position.x, position.y);
                    }
                }
            }
            WorldEvent::AgentTransfer { ref to_id, ref artifact_id, .. } => {
                for (mut art, mut target, _) in &mut artifact_query {
                    if art.artifact_id == *artifact_id {
                        art.owner = Some(to_id.clone());
                        if let Some((_, agent_tf)) = agents.iter().find(|(s, _)| s.agent_id == *to_id) {
                            target.target = agent_tf.translation.truncate() + Vec2::new(20.0, -10.0);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// Fade and despawn thought bubbles.
fn animate_thought_bubbles(
    mut commands: Commands,
    time: Res<Time>,
    mut bubbles: Query<(Entity, &mut ThoughtBubble, &mut Transform, &MeshMaterial2d<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, mut bubble, mut tf, mat_handle) in &mut bubbles {
        bubble.lifetime += time.delta_secs();
        let frac = bubble.lifetime / bubble.max_lifetime;

        // Float upward
        tf.translation.y += 12.0 * time.delta_secs();

        // Fade out the background pill
        let alpha = (1.0 - frac).max(0.0);
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.color = Color::srgba(0.1, 0.1, 0.2, 0.8 * alpha);
        }

        if bubble.lifetime >= bubble.max_lifetime {
            commands.entity(entity).despawn();
        }
    }
}

/// Animate message projectiles traveling between agents.
fn animate_message_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    agents: Query<(&AgentSprite, &Transform), Without<MessageProjectile>>,
    mut projectiles: Query<(Entity, &mut MessageProjectile, &mut Transform)>,
) {
    for (entity, mut proj, mut tf) in &mut projectiles {
        proj.progress += proj.speed * time.delta_secs();

        // Find target agent position
        let to_pos = agents
            .iter()
            .find(|(s, _)| s.agent_id == proj.to_agent_id)
            .map(|(_, t)| t.translation.truncate())
            .unwrap_or(proj.from_pos + Vec2::new(100.0, 0.0));

        if proj.progress >= 1.0 {
            commands.entity(entity).despawn();
        } else {
            // Lerp with arc
            let t = proj.progress;
            let linear = proj.from_pos.lerp(to_pos, t);
            let arc_height = 30.0 * (t * std::f32::consts::PI).sin();
            tf.translation.x = linear.x;
            tf.translation.y = linear.y + arc_height;

            // Scale down as it arrives
            let scale = 1.0 - t * 0.5;
            tf.scale = Vec3::splat(scale);
        }
    }
}

/// Animate and despawn tool effects.
fn animate_tool_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut effects: Query<(Entity, &mut ToolEffect, &mut Transform, &MeshMaterial2d<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, mut effect, mut tf, mat_handle) in &mut effects {
        effect.lifetime += time.delta_secs();
        let frac = effect.lifetime / effect.max_lifetime;

        // Expand outward
        let scale = 1.0 + frac * 1.5;
        tf.scale = Vec3::splat(scale);

        // Fade out
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            let base = mat.color.to_srgba();
            mat.color = Color::srgba(base.red, base.green, base.blue, (1.0 - frac).max(0.0) * 0.7);
        }

        if effect.lifetime >= effect.max_lifetime {
            commands.entity(entity).despawn();
        }
    }
}

/// Make artifacts glow/pulse based on quality.
fn animate_artifact_glow(
    time: Res<Time>,
    artifacts: Query<(&ArtifactSprite, &MeshMaterial2d<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let t = time.elapsed_secs();
    for (art, mat_handle) in &artifacts {
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            // Higher quality = brighter glow
            let glow = 0.7 + art.quality * 0.3 * (t * 2.0).sin().abs();
            let base = mat.color.to_srgba();
            mat.color = Color::srgba(
                base.red * glow,
                base.green * glow,
                base.blue * glow,
                base.alpha,
            );
        }
    }
}

/// Fade and despawn connection lines between agents.
fn animate_connection_lines(
    mut commands: Commands,
    time: Res<Time>,
    mut lines: Query<(Entity, &mut ConnectionLine, &MeshMaterial2d<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, mut line, mat_handle) in &mut lines {
        line.lifetime += time.delta_secs();
        let frac = line.lifetime / line.max_lifetime;

        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            let alpha = (1.0 - frac).max(0.0) * 0.2;
            mat.color = Color::srgba(1.0, 0.85, 0.2, alpha);
        }

        if line.lifetime >= line.max_lifetime {
            commands.entity(entity).despawn();
        }
    }
}

/// Format a thought string for display — clean up tool names and truncate.
fn format_thought(thought: &str) -> String {
    // Strip common prefixes and clean up tool names
    let cleaned = thought
        .replace("mcp__playwright__", "")
        .replace("mcp__opnsense__", "opn:")
        .replace("mcp__local__", "local:")
        .replace("mcp__winrm__", "win:");

    if cleaned.len() > 35 {
        format!("{}...", &cleaned[..32])
    } else {
        cleaned
    }
}

/// Keep owned artifacts near their owner agent.
fn _follow_owner(
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
