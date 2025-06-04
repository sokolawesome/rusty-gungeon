use crate::GameState;
use crate::arena::{ARENA_HEIGHT_TILES, ARENA_WIDTH_TILES, ArenaGrid, TILE_SIZE, TileType};
use bevy::{prelude::*, window::PrimaryWindow};

pub struct PlayerPlugin;

const PLAYER_DEFAULT_HEALTH: f32 = 100.0;
const PLAYER_DEFAULT_SPEED: f32 = 150.0;
const PLAYER_SPRITE_SIZE: f32 = 10.0;

const WEAPON_DEFAULT_PROJECTILE_SPEED: f32 = 400.0;
const WEAPON_DEFAULT_PROJECTILE_DAMAGE: f32 = 10.0;

const PROJECTILE_SPRITE_WIDTH: f32 = 10.0;
const PROJECTILE_SPRITE_HEIGHT: f32 = 4.0;
const PROJECTILE_LIFETIME_SECONDS: f32 = 2.0;
const PROJECTILE_SPAWN_OFFSET: f32 = 5.0;

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
                current: PLAYER_DEFAULT_HEALTH,
                max: PLAYER_DEFAULT_HEALTH,
            },
            speed: Speed(PLAYER_DEFAULT_SPEED),
            sprite: Sprite {
                color: Color::srgb(0.25, 0.5, 0.75),
                custom_size: Some(Vec2::splat(PLAYER_SPRITE_SIZE)),
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
            projectile_speed: WEAPON_DEFAULT_PROJECTILE_SPEED,
            projectile_damage: WEAPON_DEFAULT_PROJECTILE_DAMAGE,
        }
    }
}

fn spawn_player(mut commands: Commands) {
    commands.spawn(PlayerBundle::default());
}

fn check_aabb_collision(pos1: Vec2, size1: Vec2, pos2: Vec2, size2: Vec2) -> bool {
    let half_size1 = size1 / 2.0;
    let half_size2 = size2 / 2.0;

    let min1 = pos1 - half_size1;
    let max1 = pos1 + half_size1;
    let min2 = pos2 - half_size2;
    let max2 = pos2 + half_size2;

    (min1.x < max2.x && max1.x > min2.x) && (min1.y < max2.y && max1.y > min2.y)
}

fn player_movement_system(
    mut player_query: Query<(&mut Transform, &Speed, &Sprite), With<Player>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    arena_grid: Res<ArenaGrid>,
) {
    if let Ok((mut transform, speed, player)) = player_query.single_mut() {
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

            let move_amount = direction * speed.0 * time.delta_secs();

            let player_size = player
                .custom_size
                .unwrap_or(Vec2::splat(PLAYER_SPRITE_SIZE));

            let current_pos = transform.translation.truncate();

            let next_pos_x = current_pos + Vec2::new(move_amount.x, 0.0);
            let mut collision_x = false;
            if move_amount.x != 0.0 {
                for wall_pos_world in
                    get_nearby_wall_positions_world(&next_pos_x, player_size, &arena_grid)
                {
                    if check_aabb_collision(
                        next_pos_x,
                        player_size,
                        wall_pos_world,
                        Vec2::splat(TILE_SIZE),
                    ) {
                        collision_x = true;
                        break;
                    }
                }
            }
            if !collision_x {
                transform.translation.x += move_amount.x;
            }

            let current_pos_after_x_move = transform.translation.truncate();
            let next_pos_y = current_pos_after_x_move + Vec2::new(0.0, move_amount.y);
            let mut collision_y = false;
            if move_amount.y != 0.0 {
                for wall_pos_world in
                    get_nearby_wall_positions_world(&next_pos_y, player_size, &arena_grid)
                {
                    if check_aabb_collision(
                        next_pos_y,
                        player_size,
                        wall_pos_world,
                        Vec2::splat(TILE_SIZE),
                    ) {
                        collision_y = true;
                        break;
                    }
                }
            }
            if !collision_y {
                transform.translation.y += move_amount.y;
            }
        }
    }
}

fn get_nearby_wall_positions_world(
    object_pos_world: &Vec2,
    object_size: Vec2,
    arena_grid: &Res<ArenaGrid>,
) -> Vec<Vec2> {
    let mut wall_positions = Vec::new();

    let total_arena_width_pixels = ARENA_WIDTH_TILES as f32 * TILE_SIZE;
    let total_arena_height_pixels = ARENA_HEIGHT_TILES as f32 * TILE_SIZE;
    let arena_offset_x = -total_arena_width_pixels / 2.0;
    let arena_offset_y = -total_arena_height_pixels / 2.0;

    let object_half_size = object_size / 2.0;
    let search_min_world = *object_pos_world - object_half_size - Vec2::splat(TILE_SIZE); //check
    let search_max_world = *object_pos_world + object_half_size + Vec2::splat(TILE_SIZE); //check

    let start_x_grid = ((search_min_world.x - arena_offset_x) / TILE_SIZE).floor() as i32;
    let end_x_grid = ((search_max_world.x - arena_offset_x) / TILE_SIZE).ceil() as i32;
    let start_y_grid = ((search_min_world.y - arena_offset_y) / TILE_SIZE).floor() as i32;
    let end_y_grid = ((search_max_world.y - arena_offset_y) / TILE_SIZE).ceil() as i32;

    for gy in start_y_grid.max(0)..=end_y_grid.min(ARENA_HEIGHT_TILES as i32 - 1) {
        for gx in start_x_grid.max(0)..=end_x_grid.min(ARENA_WIDTH_TILES as i32 - 1) {
            let gy_usize = gy as usize;
            let gx_usize = gx as usize;
            if arena_grid.grid[gy_usize][gx_usize] == TileType::Wall {
                let wall_world_x = gx_usize as f32 * TILE_SIZE + arena_offset_x + TILE_SIZE / 2.0;
                let wall_world_y = gy_usize as f32 * TILE_SIZE + arena_offset_y + TILE_SIZE / 2.0;
                wall_positions.push(Vec2::new(wall_world_x, wall_world_y));
            }
        }
    }
    wall_positions
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
                        let direction_to_cursor =
                            world_position - player_transform.translation.truncate();
                        let angle = direction_to_cursor.y.atan2(direction_to_cursor.x);
                        player_transform.rotation = Quat::from_rotation_z(angle);
                    }
                }
            }
        }
    }
}

fn player_shooting_system(
    mut commands: Commands,
    player_query: Query<(&Transform, &Weapon), With<Player>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
) {
    if let Ok((player_transform, weapon)) = player_query.single() {
        if mouse_button_input.just_pressed(MouseButton::Left) {
            let projectile_direction_3d = player_transform.rotation * Vec3::X;

            commands.spawn(ProjectileBundle {
                data: Projectile {
                    direction: projectile_direction_3d.truncate(),
                    speed: weapon.projectile_speed,
                    lifetime: Timer::from_seconds(PROJECTILE_LIFETIME_SECONDS, TimerMode::Once),
                    damage: weapon.projectile_damage,
                },
                sprite: Sprite {
                    color: Color::WHITE,
                    custom_size: Some(Vec2::new(PROJECTILE_SPRITE_WIDTH, PROJECTILE_SPRITE_HEIGHT)),
                    ..default()
                },
                transform: Transform {
                    translation: player_transform.translation
                        + projectile_direction_3d * PROJECTILE_SPAWN_OFFSET,
                    rotation: player_transform.rotation,
                    scale: Vec3::ONE,
                },
                visibility: Visibility::Visible,
            });
        }
    }
}

fn projectile_movement_system(
    mut commands: Commands,
    mut projectile_query: Query<(Entity, &mut Transform, &Projectile, &Sprite)>,
    time: Res<Time>,
    arena_grid: Res<ArenaGrid>,
) {
    for (entity, mut transform, projectile_data, projectile_sprite) in projectile_query.iter_mut() {
        let movement_vector = projectile_data.direction * projectile_data.speed * time.delta_secs();
        let next_pos_2d = transform.translation.truncate() + movement_vector;

        let projectile_size = projectile_sprite
            .custom_size
            .unwrap_or(Vec2::new(PROJECTILE_SPRITE_WIDTH, PROJECTILE_SPRITE_HEIGHT));

        let mut collision_detected = false;
        for wall_pos_world in
            get_nearby_wall_positions_world(&next_pos_2d, projectile_size, &arena_grid)
        {
            if check_aabb_collision(
                next_pos_2d,
                projectile_size,
                wall_pos_world,
                Vec2::splat(TILE_SIZE),
            ) {
                collision_detected = true;
                break;
            }
        }

        if collision_detected {
            commands.entity(entity).despawn();
        } else {
            transform.translation += Vec3::new(movement_vector.x, movement_vector.y, 0.0);
        }
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
