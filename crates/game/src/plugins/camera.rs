use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use crate::components::AgentSprite;

pub struct CameraPlugin;

/// Tracks which agent the camera follows (if any).
#[derive(Resource, Default)]
pub struct CameraFollow {
    pub target: Option<String>,
}

/// Tracks current zoom level for the camera.
#[derive(Resource)]
pub struct CameraZoom {
    pub scale: f32,
}

impl Default for CameraZoom {
    fn default() -> Self {
        Self { scale: 1.0 }
    }
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraFollow>()
            .init_resource::<CameraZoom>()
            .add_systems(Update, (
                camera_zoom,
                camera_pan,
                camera_follow_agent,
                camera_follow_toggle,
            ));
    }
}

/// Zoom with mouse scroll wheel.
fn camera_zoom(
    scroll: Res<AccumulatedMouseScroll>,
    mut camera: Query<&mut Projection, With<Camera2d>>,
    mut zoom: ResMut<CameraZoom>,
) {
    if scroll.delta.y == 0.0 {
        return;
    }

    let zoom_delta = match scroll.unit {
        MouseScrollUnit::Line => scroll.delta.y * -0.1,
        MouseScrollUnit::Pixel => scroll.delta.y * -0.001,
    };

    zoom.scale = (zoom.scale + zoom_delta).clamp(0.3, 5.0);
    if let Ok(mut projection) = camera.single_mut() {
        if let Projection::Orthographic(ref mut ortho) = *projection {
            ortho.scale = zoom.scale;
        }
    }
}

/// Pan camera with middle mouse drag.
fn camera_pan(
    mouse_button: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
    zoom: Res<CameraZoom>,
    mut follow: ResMut<CameraFollow>,
) {
    if !mouse_button.pressed(MouseButton::Middle) {
        return;
    }

    let delta = motion.delta;
    if delta != Vec2::ZERO {
        if let Ok(mut transform) = camera.single_mut() {
            follow.target = None; // Break follow on manual pan
            transform.translation.x -= delta.x * zoom.scale;
            transform.translation.y += delta.y * zoom.scale;
        }
    }
}

/// If following an agent, smoothly track their position.
fn camera_follow_agent(
    time: Res<Time>,
    follow: Res<CameraFollow>,
    agents: Query<(&AgentSprite, &Transform), Without<Camera2d>>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
) {
    let Some(ref target_id) = follow.target else {
        return;
    };

    let Some((_, agent_transform)) = agents.iter().find(|(s, _)| &s.agent_id == target_id) else {
        return;
    };

    let target_pos = agent_transform.translation.truncate();

    if let Ok(mut cam_transform) = camera.single_mut() {
        let current = cam_transform.translation.truncate();
        let lerped = current.lerp(target_pos, 5.0 * time.delta_secs());
        cam_transform.translation.x = lerped.x;
        cam_transform.translation.y = lerped.y;
    }
}

/// Press 1/2/3 to follow agents, Escape to stop following.
fn camera_follow_toggle(
    keys: Res<ButtonInput<KeyCode>>,
    mut follow: ResMut<CameraFollow>,
) {
    if keys.just_pressed(KeyCode::Digit1) {
        follow.target = Some("researcher".into());
    } else if keys.just_pressed(KeyCode::Digit2) {
        follow.target = Some("coder".into());
    } else if keys.just_pressed(KeyCode::Digit3) {
        follow.target = Some("reviewer".into());
    } else if keys.just_pressed(KeyCode::Escape) {
        follow.target = None;
    }
}
