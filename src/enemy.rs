use bevy::prelude::*;
use rand::Rng;

use crate::GameState;
use crate::arena::{
    ARENA_HEIGHT_TILES, ARENA_WIDTH_TILES, ArenaGrid, TILE_SIZE, TileType,
    setup_arena as setup_arena_system,
};
use crate::player::{Health, Player, Speed};

const ENEMY_SPRITE_SIZE: f32 = 10.0;
const ENEMY_DEFAULT_SPEED: f32 = 75.0;
const ENEMY_DEFAULT_HEALTH: f32 = 50.0;
const ENEMY_COLOR: Color = Color::srgb(0.8, 0.2, 0.2);
const MAX_ENEMIES_SPAWN: usize = 10;

#[derive(Component)]
pub struct Enemy;

#[derive(Bundle)]
pub struct EnemyBundle {
    enemy_marker: Enemy,
    health: Health,
    speed: Speed,
    sprite: Sprite,
    transform: Transform,
    visibility: Visibility,
}

impl EnemyBundle {
    fn new(position: Vec3) -> Self {
        Self {
            enemy_marker: Enemy,
            health: Health {
                current: ENEMY_DEFAULT_HEALTH,
                max: ENEMY_DEFAULT_HEALTH,
            },
            speed: Speed(ENEMY_DEFAULT_SPEED),
            sprite: Sprite {
                color: ENEMY_COLOR,
                custom_size: Some(Vec2::splat(ENEMY_SPRITE_SIZE)),
                ..default()
            },
            transform: Transform::from_translation(position),
            visibility: Visibility::Visible,
        }
    }
}

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::InGame),
            spawn_enemies.after(setup_arena_system),
        )
        .add_systems(
            Update,
            enemy_movement_system.run_if(in_state(GameState::InGame)),
        );
    }
}

fn spawn_enemies(mut commands: Commands, arena_grid: Res<ArenaGrid>) {
    let mut rng = rand::rng();
    let mut floor_tiles = Vec::new();

    for (y, row) in arena_grid.grid.iter().enumerate() {
        for (x, tile_type) in row.iter().enumerate() {
            if *tile_type == TileType::Floor {
                let center_x = arena_grid.width / 2;
                let center_y = arena_grid.height / 2;
                let dist_to_center_sq = ((x as i32 - center_x as i32).pow(2)
                    + (y as i32 - center_y as i32).pow(2))
                    as f32;

                if x > 1
                    && x < arena_grid.width - 2
                    && y > 1
                    && y < arena_grid.height - 2
                    && dist_to_center_sq > 25.0
                {
                    floor_tiles.push((x, y));
                }
            }
        }
    }

    if floor_tiles.is_empty() {
        warn!("No valid floor tiles found to spawn enemies.");
        return;
    }

    let total_arena_width_pixels = ARENA_WIDTH_TILES as f32 * TILE_SIZE;
    let total_arena_height_pixels = ARENA_HEIGHT_TILES as f32 * TILE_SIZE;
    let arena_offset_x = -total_arena_width_pixels / 2.0;
    let arena_offset_y = -total_arena_height_pixels / 2.0;

    for _ in 0..MAX_ENEMIES_SPAWN {
        if let Some(idx) = floor_tiles
            .get(rng.random_range(0..floor_tiles.len()))
            .copied()
        {
            let (grid_x, grid_y) = idx;
            let world_x = grid_x as f32 * TILE_SIZE + arena_offset_x + TILE_SIZE / 2.0;
            let world_y = grid_y as f32 * TILE_SIZE + arena_offset_y + TILE_SIZE / 2.0;

            commands.spawn(EnemyBundle::new(Vec3::new(world_x, world_y, 0.0)));
        }
    }
    info!(
        "Spawned {} enemies.",
        MAX_ENEMIES_SPAWN.min(floor_tiles.len())
    );
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
    let search_min_world = *object_pos_world - object_half_size - Vec2::splat(TILE_SIZE * 0.5);
    let search_max_world = *object_pos_world + object_half_size + Vec2::splat(TILE_SIZE * 0.5);
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

fn enemy_movement_system(
    mut enemy_query: Query<(&mut Transform, &Speed, &Sprite), (With<Enemy>, Without<Player>)>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    time: Res<Time>,
    arena_grid: Res<ArenaGrid>,
) {
    if let Ok(player_transform) = player_query.single() {
        let player_pos = player_transform.translation.truncate();

        for (mut enemy_transform, enemy_speed, enemy_sprite) in enemy_query.iter_mut() {
            let enemy_current_pos = enemy_transform.translation.truncate();
            let direction_to_player = (player_pos - enemy_current_pos).normalize_or_zero();

            if direction_to_player != Vec2::ZERO {
                let move_amount_total = direction_to_player * enemy_speed.0 * time.delta_secs();
                let enemy_size = enemy_sprite
                    .custom_size
                    .unwrap_or(Vec2::splat(ENEMY_SPRITE_SIZE));

                let next_pos_x = enemy_current_pos + Vec2::new(move_amount_total.x, 0.0);
                let mut collision_x = false;
                if move_amount_total.x.abs() > f32::EPSILON {
                    for wall_pos_world in
                        get_nearby_wall_positions_world(&next_pos_x, enemy_size, &arena_grid)
                    {
                        if check_aabb_collision(
                            next_pos_x,
                            enemy_size,
                            wall_pos_world,
                            Vec2::splat(TILE_SIZE),
                        ) {
                            collision_x = true;
                            break;
                        }
                    }
                }
                if !collision_x {
                    enemy_transform.translation.x += move_amount_total.x;
                }

                let enemy_current_pos_after_x = enemy_transform.translation.truncate();
                let next_pos_y = enemy_current_pos_after_x + Vec2::new(0.0, move_amount_total.y);
                let mut collision_y = false;
                if move_amount_total.y.abs() > f32::EPSILON {
                    for wall_pos_world in
                        get_nearby_wall_positions_world(&next_pos_y, enemy_size, &arena_grid)
                    {
                        if check_aabb_collision(
                            next_pos_y,
                            enemy_size,
                            wall_pos_world,
                            Vec2::splat(TILE_SIZE),
                        ) {
                            collision_y = true;
                            break;
                        }
                    }
                }
                if !collision_y {
                    enemy_transform.translation.y += move_amount_total.y;
                }

                let final_enemy_pos = enemy_transform.translation.truncate();
                let final_direction_to_player = (player_pos - final_enemy_pos).normalize_or_zero();
                if final_direction_to_player != Vec2::ZERO {
                    let angle = final_direction_to_player
                        .y
                        .atan2(final_direction_to_player.x);
                    enemy_transform.rotation = Quat::from_rotation_z(angle);
                }
            }
        }
    }
}
