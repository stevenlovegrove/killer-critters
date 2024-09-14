use crate::map::Map;
use crate::tile::PowerUpType;
use crate::tile::TileType;
use crate::tile::Tile;
use bevy::prelude::*;

const PROB_FIREPOWER: f32 = 0.08;
const PROB_EXTRABOMB: f32 = 0.08;

fn within_distance_of_spawn_points(pos: IVec2, spawn_points: &[IVec2], distance: i32) -> bool {
    spawn_points.iter().any(|spawn_point| spawn_point.distance_squared(pos) <= distance * distance)
}

pub fn make_really_basic_map1(commands: &mut Commands) -> Map {
    let mut map = Map::new_empty(commands, 3,3);
    map.set_spawn_points(vec![
        IVec2::new(1, 1),
        IVec2::new(1, 1),
    ]);

    for pos in map.pos_iter() {
        let tile_type = if map.is_edge(pos) {
            TileType::SolidWall
        } else {
            TileType::Empty
        };
        commands.entity(map[pos]).insert(Tile { tile_type });
    }

    map
}

pub fn make_really_basic_map2(commands: &mut Commands) -> Map {
    let mut map = Map::new_empty(commands, 5,5);
    map.set_spawn_points(vec![
        IVec2::new(1, 1),
        IVec2::new(1, 1),
    ]);

    for pos in map.pos_iter() {
        let tile_type = if map.is_edge(pos) || (pos.x == 2 && pos.y == 2) {
            TileType::SolidWall
        } else {
            TileType::Empty
        };
        commands.entity(map[pos]).insert(Tile { tile_type });
    }

    map
}

pub fn make_basic_map(commands: &mut Commands, width: usize, height: usize) -> Map {
    let mut map = Map::new_empty(commands, width, height);
    map.set_spawn_points(vec![
        IVec2::new(1, 1),
        IVec2::new(1, height as i32 - 2),
        IVec2::new(width as i32 - 2, 1),
        IVec2::new(width as i32 - 2, height as i32 - 2),
    ]);

    for pos in map.pos_iter() {
        let tile_type = if map.is_edge(pos) {
            TileType::SolidWall
        } else if pos.x % 2 == 0 && pos.y % 2 == 0 {
            TileType::SolidWall
        } else if rand::random::<f32>() < 0.8 && !within_distance_of_spawn_points(pos, &map.spawn_points(), 1) {
            let contents = if rand::random::<f32>() < PROB_FIREPOWER {
                Box::new(TileType::PowerUp(PowerUpType::Firepower))
            } else if rand::random::<f32>() < PROB_EXTRABOMB {
                Box::new(TileType::PowerUp(PowerUpType::ExtraBomb))
            } else {
                Box::new(TileType::Empty)
            };
            TileType::BreakableWall(contents)
        } else {
            TileType::Empty
        };
        commands.entity(map[pos]).insert(Tile { tile_type });
    }

    map
}
