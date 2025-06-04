use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use rand::{Rng, rng};

use crate::GameState;

pub const ARENA_WIDTH_TILES: usize = 86;
pub const ARENA_HEIGHT_TILES: usize = 49;
pub const TILE_SIZE: f32 = 15.0;
const WALL_COLOR: Color = Color::srgb(0.5, 0.5, 0.5);
const FLOOR_COLOR: Color = Color::srgb(0.15, 0.15, 0.18);

const NOISE_SCALE: f64 = 0.4;
const NOISE_THRESHOLD: f64 = 0.1;

const SMOOTHING_ITERATIONS: usize = 3;
const WALL_CONVERSION_THRESHOLD: usize = 5;
const FLOOR_CONVERSION_THRESHOLD: usize = 4;

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
        let perlin = Perlin::new(rng().random());

        for y in 0..height {
            for x in 0..width {
                let noise_val = perlin.get([x as f64 * NOISE_SCALE, y as f64 * NOISE_SCALE]);

                if noise_val > NOISE_THRESHOLD {
                    grid[y][x] = TileType::Wall;
                } else {
                    grid[y][x] = TileType::Floor;
                }
            }
        }

        for _ in 0..SMOOTHING_ITERATIONS {
            let mut next_grid = grid.clone();
            for y in 1..(height - 1) {
                for x in 1..(width - 1) {
                    let wall_neighbors = count_wall_neighbors(&grid, x, y, width, height);

                    if grid[y][x] == TileType::Wall {
                        if wall_neighbors < FLOOR_CONVERSION_THRESHOLD {
                            next_grid[y][x] = TileType::Floor;
                        }
                    } else {
                        if wall_neighbors >= WALL_CONVERSION_THRESHOLD {
                            next_grid[y][x] = TileType::Wall;
                        }
                    }
                }
            }
            grid = next_grid;
        }

        for x in 0..width {
            grid[0][x] = TileType::Wall;
            grid[height - 1][x] = TileType::Wall;
        }
        for y in 0..height {
            grid[y][0] = TileType::Wall;
            grid[y][width - 1] = TileType::Wall;
        }

        let center_x = width / 2;
        let center_y = height / 2;
        for _r in 0..=1 {
            for c_offset in -1..=1 {
                for r_offset in -1..=1 {
                    let clear_x = (center_x as i32 + c_offset) as usize;
                    let clear_y = (center_y as i32 + r_offset) as usize;
                    if clear_x > 0 && clear_x < width - 1 && clear_y > 0 && clear_y < height - 1 {
                        grid[clear_y][clear_x] = TileType::Floor;
                    }
                }
            }
        }

        Self {
            grid,
            width,
            height,
        }
    }
}

fn count_wall_neighbors(
    grid: &Vec<Vec<TileType>>,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) -> usize {
    let mut count = 0;
    for i in -1..=1 {
        for j in -1..=1 {
            if i == 0 && j == 0 {
                continue;
            }
            let check_x = x as i32 + i;
            let check_y = y as i32 + j;

            if check_x >= 0 && check_x < width as i32 && check_y >= 0 && check_y < height as i32 {
                if grid[check_y as usize][check_x as usize] == TileType::Wall {
                    count += 1;
                }
            } else {
                count += 1;
            }
        }
    }
    count
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

    commands.spawn((
        ArenaFloor,
        Sprite {
            color: FLOOR_COLOR,
            custom_size: Some(Vec2::new(
                total_arena_width_pixels,
                total_arena_height_pixels,
            )),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -1.0),
        Visibility::Visible,
    ));

    for y in 0..arena_grid.height {
        for x in 0..arena_grid.width {
            if arena_grid.grid[y][x] == TileType::Wall {
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
            }
        }
    }

    commands.insert_resource(arena_grid);
    info!("Arena setup complete with walls.");
}
