use agent_world_core::WorldEvent;
use bevy::prelude::*;
use crate::components::{AmbientParticle, GridCell, PortalSprite};
use crate::plugins::events::PendingEvents;
use std::collections::HashMap;

pub struct WorldPlugin;

/// Maps room_id → world-space center position.
#[derive(Resource, Default)]
pub struct RoomIndex {
    pub positions: HashMap<String, Vec2>,
}

/// Maps portal_id → (world_pos, target_room_id, target_local_pos).
#[derive(Resource, Default)]
pub struct PortalIndex {
    pub portals: HashMap<String, PortalInfo>,
}

pub struct PortalInfo {
    pub world_pos: Vec2,
    pub target_room: String,
    pub target_local_pos: Vec2,
}

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RoomIndex>()
            .init_resource::<PortalIndex>()
            .add_systems(Startup, setup_camera)
            .add_systems(Update, (handle_room_events, animate_portals, emit_ambient_particles, animate_ambient_particles));
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Msaa::Off,
        Projection::Orthographic(OrthographicProjection {
            scale: 1.5,
            ..OrthographicProjection::default_2d()
        }),
    ));
}

/// Process RoomCreate events and spawn room geometry.
fn handle_room_events(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    pending: Res<PendingEvents>,
    mut room_count: Local<usize>,
    mut room_index: ResMut<RoomIndex>,
    mut portal_index: ResMut<PortalIndex>,
) {
    for event in &pending.queue {
        if let WorldEvent::RoomCreate(room) = event {
            // Layout rooms in a horizontal strip with gaps
            let room_spacing = 700.0;
            let room_x = *room_count as f32 * room_spacing;
            let room_y = 0.0;

            // Room theme colors based on purpose
            let (dark_color, light_color, border_color) = match room.purpose.as_str() {
                "workspace" => (
                    Color::srgb(0.12, 0.12, 0.22),
                    Color::srgb(0.22, 0.22, 0.38),
                    Color::srgba(0.3, 0.3, 0.6, 0.5),
                ),
                "review" => (
                    Color::srgb(0.15, 0.10, 0.20),
                    Color::srgb(0.28, 0.18, 0.35),
                    Color::srgba(0.5, 0.3, 0.6, 0.5),
                ),
                "deploy" => (
                    Color::srgb(0.10, 0.15, 0.12),
                    Color::srgb(0.18, 0.28, 0.20),
                    Color::srgba(0.3, 0.6, 0.3, 0.5),
                ),
                _ => (
                    Color::srgb(0.12, 0.12, 0.22),
                    Color::srgb(0.22, 0.22, 0.38),
                    Color::srgba(0.3, 0.3, 0.6, 0.5),
                ),
            };

            let grid_size = 10;
            let tile_size = 48.0;
            let half = (grid_size as f32 * tile_size) / 2.0 - tile_size / 2.0;

            let tile_mesh = meshes.add(Rectangle::new(tile_size - 1.0, tile_size - 1.0));
            let dark_mat = materials.add(ColorMaterial::from_color(dark_color));
            let light_mat = materials.add(ColorMaterial::from_color(light_color));

            // Grid tiles
            for row in 0..grid_size {
                for col in 0..grid_size {
                    let is_dark = (row + col) % 2 == 0;
                    let mat = if is_dark { dark_mat.clone() } else { light_mat.clone() };

                    commands.spawn((
                        Mesh2d(tile_mesh.clone()),
                        MeshMaterial2d(mat),
                        Transform::from_xyz(
                            room_x + col as f32 * tile_size - half,
                            room_y + row as f32 * tile_size - half,
                            0.0,
                        ),
                        GridCell,
                    ));
                }
            }

            // Room border (4 edges)
            let border_w = grid_size as f32 * tile_size + 4.0;
            let border_thickness = 3.0;
            let border_mat = materials.add(ColorMaterial::from_color(border_color));

            // Top border
            let h_mesh = meshes.add(Rectangle::new(border_w, border_thickness));
            let v_mesh = meshes.add(Rectangle::new(border_thickness, border_w));
            let border_offset = half + tile_size / 2.0 + 2.0;

            for (x, y, is_horizontal) in [
                (room_x, room_y + border_offset, true),
                (room_x, room_y - border_offset, true),
                (room_x + border_offset, room_y, false),
                (room_x - border_offset, room_y, false),
            ] {
                let mesh = if is_horizontal { h_mesh.clone() } else { v_mesh.clone() };
                commands.spawn((
                    Mesh2d(mesh),
                    MeshMaterial2d(border_mat.clone()),
                    Transform::from_xyz(x, y, 0.1),
                ));
            }

            // Room name label (above room)
            commands.spawn((
                Text2d::new(&room.name),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgba(0.6, 0.6, 0.8, 0.6)),
                Transform::from_xyz(room_x, room_y + border_offset + 16.0, 0.2),
            ));

            // Room purpose label (inside room, faint)
            let purpose_icon = match room.purpose.as_str() {
                "workspace" => "{ }",
                "review" => "< >",
                "deploy" => ">>>",
                _ => "...",
            };
            commands.spawn((
                Text2d::new(purpose_icon),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgba(0.2, 0.2, 0.3, 0.15)),
                Transform::from_xyz(room_x, room_y, 0.05),
            ));

            // Desk markers — subtle rectangles showing workstation areas
            let desk_mesh = meshes.add(Rectangle::new(50.0, 4.0));
            let desk_mat = materials.add(ColorMaterial::from_color(
                Color::srgba(border_color.to_srgba().red, border_color.to_srgba().green, border_color.to_srgba().blue, 0.15),
            ));
            // Place 2-3 desks per room based on purpose
            let desk_positions: Vec<(f32, f32)> = match room.purpose.as_str() {
                "workspace" => vec![(-100.0, 80.0), (50.0, -60.0), (-60.0, -100.0)],
                "review" => vec![(0.0, 50.0), (-80.0, -80.0), (80.0, -40.0)],
                "deploy" => vec![(0.0, 0.0), (-60.0, 80.0)],
                _ => vec![(0.0, 0.0)],
            };
            for (dx, dy) in desk_positions {
                commands.spawn((
                    Mesh2d(desk_mesh.clone()),
                    MeshMaterial2d(desk_mat.clone()),
                    Transform::from_xyz(room_x + dx, room_y + dy - 16.0, 0.08),
                ));
            }

            // Register room position
            room_index.positions.insert(room.id.clone(), Vec2::new(room_x, room_y));

            // Spawn portals
            for portal in &room.portals {
                let portal_mesh = meshes.add(Circle::new(14.0));
                let portal_mat = materials.add(ColorMaterial::from_color(
                    Color::srgba(0.4, 0.2, 0.9, 0.6),
                ));

                let portal_entity = commands.spawn((
                    Mesh2d(portal_mesh),
                    MeshMaterial2d(portal_mat),
                    Transform::from_xyz(
                        room_x + portal.position.x,
                        room_y + portal.position.y,
                        0.5,
                    ),
                    PortalSprite {
                        portal_id: portal.id.clone(),
                        target_room: portal.target_room.clone(),
                    },
                )).id();

                // Register portal in index
                portal_index.portals.insert(portal.id.clone(), PortalInfo {
                    world_pos: Vec2::new(room_x + portal.position.x, room_y + portal.position.y),
                    target_room: portal.target_room.clone(),
                    target_local_pos: Vec2::new(portal.target_position.x, portal.target_position.y),
                });

                // Portal label as child
                commands.spawn((
                    Text2d::new(format!("→ {}", portal.target_room)),
                    TextFont {
                        font_size: 9.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.7, 0.5, 1.0, 0.8)),
                    Transform::from_xyz(0.0, 20.0, 0.5),
                    ChildOf(portal_entity),
                ));
            }

            *room_count += 1;
        }
    }
}

/// Animate portals with a gentle pulse.
fn animate_portals(
    time: Res<Time>,
    mut portals: Query<(&mut Transform, &MeshMaterial2d<ColorMaterial>), With<PortalSprite>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let t = time.elapsed_secs();
    for (mut tf, mat_handle) in &mut portals {
        let pulse = 1.0 + (t * 2.0).sin() * 0.15;
        tf.scale = Vec3::splat(pulse);

        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            let alpha = 0.4 + (t * 1.5).sin().abs() * 0.3;
            mat.color = Color::srgba(0.4, 0.2, 0.9, alpha);
        }
    }
}

/// Periodically spawn ambient floating particles in rooms.
fn emit_ambient_particles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    room_index: Res<RoomIndex>,
    time: Res<Time>,
    mut timer: Local<f32>,
    existing: Query<&AmbientParticle>,
) {
    *timer += time.delta_secs();
    if *timer < 1.0 || room_index.positions.is_empty() {
        return;
    }
    *timer = 0.0;

    // Cap total particles
    if existing.iter().count() > 30 {
        return;
    }

    let t = time.elapsed_secs();
    let dot_mesh = meshes.add(Circle::new(2.0));

    for (_, &room_pos) in &room_index.positions {
        // Spawn 1 particle per room per second, at random-ish position
        let offset_x = ((t * 7.3).sin() * 200.0 + room_pos.x) % 200.0 - 100.0;
        let offset_y = ((t * 5.1).cos() * 200.0 + room_pos.y) % 200.0 - 100.0;
        let drift_x = (t * 3.7).sin() * 5.0;
        let drift_y = 3.0 + (t * 2.1).cos().abs() * 4.0;

        let dot_mat = materials.add(ColorMaterial::from_color(
            Color::srgba(0.3, 0.3, 0.6, 0.08),
        ));

        commands.spawn((
            Mesh2d(dot_mesh.clone()),
            MeshMaterial2d(dot_mat),
            Transform::from_xyz(room_pos.x + offset_x, room_pos.y + offset_y, 0.04),
            AmbientParticle {
                lifetime: 0.0,
                max_lifetime: 6.0,
                drift: Vec2::new(drift_x, drift_y),
            },
        ));
    }
}

/// Animate and despawn ambient particles (slow upward drift + fade).
fn animate_ambient_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut AmbientParticle, &mut Transform, &MeshMaterial2d<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, mut particle, mut tf, mat_handle) in &mut particles {
        particle.lifetime += time.delta_secs();
        let frac = particle.lifetime / particle.max_lifetime;

        // Drift upward
        tf.translation.x += particle.drift.x * time.delta_secs();
        tf.translation.y += particle.drift.y * time.delta_secs();

        // Fade
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            let alpha = 0.08 * (1.0 - frac).max(0.0);
            mat.color = Color::srgba(0.3, 0.3, 0.6, alpha);
        }

        if particle.lifetime >= particle.max_lifetime {
            commands.entity(entity).despawn();
        }
    }
}
