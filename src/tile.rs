use bevy::prelude::*;
use web_time::Instant;

pub const TILE_SIZE: f32 = 1.0;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Bomb {
    pub when_to_explode: Instant,
    pub firepower: i32,
    pub player_entity: Entity,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PowerUpType {
    Firepower,
    ExtraBomb,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Default)]
pub enum TileType {
    #[default]
    Empty,
    SolidWall,
    BreakableWall(Box<TileType>),
    Bomb(Option<Bomb>),
    Explosion(Option<Instant>, Box<TileType>),
    PowerUp(PowerUpType),
}

#[derive(Component)]
pub struct Tile {
    pub tile_type: TileType,
}
