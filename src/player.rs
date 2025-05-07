use crate::GameState;
use bevy::{prelude::*, window::PrimaryWindow};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), spawn_player)
            .add_systems(
                Update,
                (
                    player_movement_system,
                    player_aiming_system,
                    player_shooting_system,
                    projectile_movement_system,
                    projectile_lifetime_system,
                )
                    .run_if(in_state(GameState::InGame)),
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

#[derive(Component)]
pub struct Weapon {
    pub projectile_speed: f32,
    pub projectile_damage: f32,
}

#[derive(Component)]
pub struct Projectile {
    pub direction: Vec2,
    pub speed: f32,
    pub lifetime: Timer,
    pub damage: f32,
}

#[derive(Bundle)]
pub struct ProjectileBundle {
    data: Projectile,
    sprite: Sprite,
    transform: Transform,
    visibility: Visibility,
}

#[derive(Bundle)]
pub struct PlayerBundle {
    player_marker: Player,
    health: Health,
    speed: Speed,
    sprite: Sprite,
    transform: Transform,
    visibility: Visibility,
    weapon: Weapon,
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
            weapon: Weapon::default(),
        }
    }
}

impl Default for Weapon {
    fn default() -> Self {
        Self {
            projectile_speed: 600.0,
            projectile_damage: 10.0,
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

fn player_shooting_system(
    mut commands: Commands,
    mut player_query: Query<(&Transform, &mut Weapon), With<Player>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
) {
    if let Ok((player_transform, weapon)) = player_query.single_mut() {
        if mouse_button_input.just_pressed(MouseButton::Left) {
            let projectile_direction = player_transform.rotation * Vec3::X;

            commands.spawn(ProjectileBundle {
                data: Projectile {
                    direction: projectile_direction.truncate().normalize_or_zero(),
                    speed: weapon.projectile_speed,
                    lifetime: Timer::from_seconds(2.0, TimerMode::Once),
                    damage: weapon.projectile_damage,
                },
                sprite: Sprite {
                    color: Color::WHITE,
                    custom_size: Some(Vec2::new(10.0, 4.0)),
                    ..default()
                },
                transform: Transform {
                    translation: player_transform.translation + projectile_direction * 20.0,
                    rotation: player_transform.rotation,
                    scale: Vec3::ONE,
                },
                visibility: Visibility::Visible,
            });
        }
    }
}

fn projectile_movement_system(
    mut projectile_query: Query<(&mut Transform, &Projectile)>,
    time: Res<Time>,
) {
    for (mut transform, projectile) in projectile_query.iter_mut() {
        let movement = projectile.direction * projectile.speed * time.delta_secs();
        transform.translation += Vec3::new(movement.x, movement.y, 0.0);
    }
}

fn projectile_lifetime_system(
    mut commands: Commands,
    mut projectile_query: Query<(Entity, &mut Projectile)>,
    time: Res<Time>,
) {
    for (entity, mut projectile) in projectile_query.iter_mut() {
        projectile.lifetime.tick(time.delta());
        if projectile.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}
