use crate::GameState;
use bevy::{prelude::*, window::PrimaryWindow};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), spawn_player)
            .add_systems(
                Update,
                (player_movement_system, player_aiming_system).run_if(in_state(GameState::InGame)),
            );
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component)]
pub struct Speed(pub f32);

#[derive(Bundle)]
pub struct PlayerBundle {
    player_marker: Player,
    health: Health,
    speed: Speed,
    sprite: Sprite,
    transform: Transform,
    visibility: Visibility,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            player_marker: Player,
            health: Health {
                current: 100.0,
                max: 100.0,
            },
            speed: Speed(250.0),
            sprite: Sprite {
                color: Color::srgb(0.25, 0.5, 0.75),
                custom_size: Some(Vec2::new(32.0, 32.0)),
                ..default()
            },
            transform: Transform::default(),
            visibility: Visibility::Visible,
        }
    }
}

fn spawn_player(mut commands: Commands) {
    commands.spawn(PlayerBundle::default());
}

fn player_movement_system(
    mut player_query: Query<(&mut Transform, &Speed), With<Player>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    if let Ok((mut transform, speed)) = player_query.single_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
            direction.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
            direction.x += 1.0;
        }

        if direction.length_squared() > 0.0 {
            direction = direction.normalize();
            transform.translation += direction * speed.0 * time.delta_secs();
        }
    }
}

fn player_aiming_system(
    mut player_query: Query<&mut Transform, With<Player>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    if let Ok(mut player_transform) = player_query.single_mut() {
        if let Ok(primary_window) = window_query.single() {
            if let Some(cursor_position) = primary_window.cursor_position() {
                if let Ok((camera, camera_global_transform)) = camera_query.single() {
                    if let Ok(world_position) =
                        camera.viewport_to_world_2d(camera_global_transform, cursor_position)
                    {
                        let direction = world_position - player_transform.translation.truncate();
                        let angle = direction.y.atan2(direction.x);
                        player_transform.rotation = Quat::from_rotation_z(angle)
                    }
                }
            }
        }
    }
}
