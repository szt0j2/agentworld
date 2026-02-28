use bevy::prelude::*;
use crate::components::GridCell;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_camera, spawn_grid));
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Spawn a grid of colored tiles to represent the room floor.
fn spawn_grid(mut commands: Commands) {
    let grid_size = 12;
    let tile_size = 48.0;
    let offset = (grid_size as f32 * tile_size) / 2.0 - tile_size / 2.0;

    for row in 0..grid_size {
        for col in 0..grid_size {
            let is_dark = (row + col) % 2 == 0;
            let color = if is_dark {
                Color::srgb(0.15, 0.15, 0.25)
            } else {
                Color::srgb(0.18, 0.18, 0.30)
            };

            commands.spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::splat(tile_size - 1.0)),
                    ..default()
                },
                Transform::from_xyz(
                    col as f32 * tile_size - offset,
                    row as f32 * tile_size - offset,
                    0.0,
                ),
                GridCell,
            ));
        }
    }
}
