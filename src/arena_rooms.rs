use bevy::prelude::*;
use rand::Rng;

use crate::GameState;

pub const ARENA_WIDTH_TILES: usize = 50;
pub const ARENA_HEIGHT_TILES: usize = 40;
pub const TILE_SIZE: f32 = 20.0;

const WALL_COLOR: Color = Color::srgb(0.5, 0.5, 0.5);
const FLOOR_COLOR: Color = Color::srgb(0.15, 0.15, 0.18);

const ROOM_PADDING_TILES: usize = 3;
const MIN_ROOM_SIZE_TILES: usize = 8;

const OBSTACLE_SPAWN_PROBABILITY: f64 = 0.15;
const MIN_OBSTACLE_SIZE: usize = 1;
const MAX_OBSTACLE_SIZE: usize = 3;

#[derive(Component)]
pub struct Wall;

#[derive(Component)]
pub struct ArenaFloor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileType {
    Floor,
    Wall,
}

#[derive(Resource)]
pub struct ArenaGrid {
    pub grid: Vec<Vec<TileType>>,
    pub width: usize,
    pub height: usize,
}

impl ArenaGrid {
    fn new(width: usize, height: usize) -> Self {
        let mut grid = vec![vec![TileType::Floor; width]; height];
        let mut rng = rand::rng();

        let max_room_width = width - 2 * ROOM_PADDING_TILES;
        let max_room_height = height - 2 * ROOM_PADDING_TILES;

        let room_width = max_room_width.max(MIN_ROOM_SIZE_TILES);
        let room_height = max_room_height.max(MIN_ROOM_SIZE_TILES);

        let room_start_x = (width - room_width) / 2;
        let room_start_y = (height - room_height) / 2;
        let room_end_x = room_start_x + room_width;
        let room_end_y = room_start_y + room_height;

        for (y, row_mut) in grid.iter_mut().enumerate() {
            for (x, cell_mut) in row_mut.iter_mut().enumerate() {
                if x >= room_start_x && x < room_end_x && y >= room_start_y && y < room_end_y {
                    if x == room_start_x
                        || x == room_end_x - 1
                        || y == room_start_y
                        || y == room_end_y - 1
                    {
                        *cell_mut = TileType::Wall;
                    } else {
                        *cell_mut = TileType::Floor;
                    }
                } else {
                    *cell_mut = TileType::Wall;
                }
            }
        }

        for y in (room_start_y + 1)..(room_end_y - 1) {
            for x in (room_start_x + 1)..(room_end_x - 1) {
                if grid[y][x] == TileType::Floor && rng.random_bool(OBSTACLE_SPAWN_PROBABILITY) {
                    let obs_width = rng.random_range(MIN_OBSTACLE_SIZE..=MAX_OBSTACLE_SIZE);
                    let obs_height = rng.random_range(MIN_OBSTACLE_SIZE..=MAX_OBSTACLE_SIZE);

                    for oy in 0..obs_height {
                        for ox in 0..obs_width {
                            let current_x = x + ox;
                            let current_y = y + oy;
                            if current_x < (room_end_x - 1) && current_y < (room_end_y - 1) {
                                grid[current_y][current_x] = TileType::Wall;
                            }
                        }
                    }
                }
            }
        }

        let room_center_x = room_start_x + room_width / 2;
        let room_center_y = room_start_y + room_height / 2;

        for r_offset in -1..=1 {
            for c_offset in -1..=1 {
                let clear_y = room_center_y as i32 + r_offset;
                let clear_x = room_center_x as i32 + c_offset;

                if clear_y > room_start_y as i32
                    && clear_y < (room_end_y - 1) as i32
                    && clear_x > room_start_x as i32
                    && clear_x < (room_end_x - 1) as i32
                {
                    grid[clear_y as usize][clear_x as usize] = TileType::Floor;
                }
            }
        }
        if room_center_x > room_start_x
            && room_center_x < room_end_x - 1
            && room_center_y > room_start_y
            && room_center_y < room_end_y - 1
        {
            grid[room_center_y][room_center_x] = TileType::Floor;
        }

        Self {
            grid,
            width,
            height,
        }
    }
}

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), setup_arena);
    }
}

fn setup_arena(mut commands: Commands) {
    let arena_grid = ArenaGrid::new(ARENA_WIDTH_TILES, ARENA_HEIGHT_TILES);

    let total_arena_width_pixels = ARENA_WIDTH_TILES as f32 * TILE_SIZE;
    let total_arena_height_pixels = ARENA_HEIGHT_TILES as f32 * TILE_SIZE;

    for (y, row) in arena_grid.grid.iter().enumerate() {
        for (x, tile_type) in row.iter().enumerate() {
            if *tile_type == TileType::Wall {
                let pos_x =
                    (x as f32 * TILE_SIZE) - (total_arena_width_pixels / 2.0) + (TILE_SIZE / 2.0);
                let pos_y =
                    (y as f32 * TILE_SIZE) - (total_arena_height_pixels / 2.0) + (TILE_SIZE / 2.0);

                commands.spawn((
                    Wall,
                    Sprite {
                        color: WALL_COLOR,
                        custom_size: Some(Vec2::splat(TILE_SIZE)),
                        ..default()
                    },
                    Transform::from_xyz(pos_x, pos_y, 0.0),
                    Visibility::Visible,
                ));
            } else if *tile_type == TileType::Floor {
                let total_room_width_pixels = arena_grid.width as f32 * TILE_SIZE;
                let total_room_height_pixels = arena_grid.height as f32 * TILE_SIZE;

                let pos_x =
                    (x as f32 * TILE_SIZE) - (total_room_width_pixels / 2.0) + (TILE_SIZE / 2.0);
                let pos_y =
                    (y as f32 * TILE_SIZE) - (total_room_height_pixels / 2.0) + (TILE_SIZE / 2.0);

                commands.spawn((
                    ArenaFloor,
                    Sprite {
                        color: FLOOR_COLOR,
                        custom_size: Some(Vec2::splat(TILE_SIZE)),
                        ..default()
                    },
                    Transform::from_xyz(pos_x, pos_y, -1.0),
                    Visibility::Visible,
                ));
            }
        }
    }

    commands.insert_resource(arena_grid);
    info!("Arena setup complete with walls.");
}
