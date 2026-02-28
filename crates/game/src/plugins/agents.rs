use bevy::prelude::*;
use crate::components::{AgentLabel, AgentSprite, MovementTarget};

pub struct AgentPlugin;

impl Plugin for AgentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_demo_agents)
            .add_systems(Update, (move_agents, assign_new_targets));
    }
}

struct AgentDef {
    id: &'static str,
    name: &'static str,
    role: &'static str,
    color: Color,
    start: Vec2,
}

fn spawn_demo_agents(mut commands: Commands) {
    let agents = [
        AgentDef {
            id: "researcher",
            name: "Researcher",
            role: "researcher",
            color: Color::srgb(0.2, 0.6, 1.0),
            start: Vec2::new(-100.0, 80.0),
        },
        AgentDef {
            id: "coder",
            name: "Coder",
            role: "coder",
            color: Color::srgb(0.2, 0.9, 0.4),
            start: Vec2::new(50.0, -60.0),
        },
        AgentDef {
            id: "reviewer",
            name: "Reviewer",
            role: "reviewer",
            color: Color::srgb(0.9, 0.5, 0.2),
            start: Vec2::new(120.0, 100.0),
        },
    ];

    for def in &agents {
        // Agent square
        let agent_entity = commands
            .spawn((
                Sprite {
                    color: def.color,
                    custom_size: Some(Vec2::splat(32.0)),
                    ..default()
                },
                Transform::from_xyz(def.start.x, def.start.y, 1.0),
                AgentSprite {
                    agent_id: def.id.to_string(),
                    name: def.name.to_string(),
                    role: def.role.to_string(),
                },
                MovementTarget {
                    target: def.start,
                    speed: 60.0,
                },
            ))
            .id();

        // Name label above agent
        commands.spawn((
            Text2d::new(def.name),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_xyz(def.start.x, def.start.y + 26.0, 2.0),
            AgentLabel,
            ChildOf(agent_entity),
        ));
    }
}

/// Move agents toward their movement target.
fn move_agents(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &MovementTarget), With<AgentSprite>>,
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

/// Periodically assign new random targets so agents wander.
fn assign_new_targets(
    time: Res<Time>,
    mut query: Query<(&Transform, &mut MovementTarget), With<AgentSprite>>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();
    if *timer < 2.0 {
        return;
    }
    *timer = 0.0;

    let bounds = 200.0;
    // Simple pseudo-random based on time
    let t = time.elapsed_secs();

    for (i, (_transform, mut target)) in query.iter_mut().enumerate() {
        let seed = t + i as f32 * 137.5;
        let x = (seed.sin() * 1000.0).fract() * bounds * 2.0 - bounds;
        let y = (seed.cos() * 1000.0).fract() * bounds * 2.0 - bounds;
        target.target = Vec2::new(x, y);
    }
}
