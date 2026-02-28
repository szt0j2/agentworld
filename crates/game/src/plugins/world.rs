use bevy::prelude::*;
use crate::components::GridCell;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_camera, spawn_grid));
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Msaa::Off));
}

/// Spawn a grid of colored tiles to represent the room floor.
fn spawn_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let grid_size = 12;
    let tile_size = 48.0;
    let offset = (grid_size as f32 * tile_size) / 2.0 - tile_size / 2.0;

    let tile_mesh = meshes.add(Rectangle::new(tile_size - 1.0, tile_size - 1.0));
    let dark_mat = materials.add(ColorMaterial::from_color(Color::srgb(0.12, 0.12, 0.22)));
    let light_mat = materials.add(ColorMaterial::from_color(Color::srgb(0.22, 0.22, 0.38)));

    for row in 0..grid_size {
        for col in 0..grid_size {
            let is_dark = (row + col) % 2 == 0;
            let mat = if is_dark { dark_mat.clone() } else { light_mat.clone() };

            commands.spawn((
                Mesh2d(tile_mesh.clone()),
                MeshMaterial2d(mat),
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
