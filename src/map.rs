use crate::tile::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct Map {
    width: usize,
    height: usize,
    grid_entities: Vec<Entity>,
    player_spawn_points: Vec<IVec2>,
}

impl Map {
    pub fn new_empty(commands: &mut Commands, width: usize, height: usize) -> Self {
        let mut entities = Vec::new();

        for y in 0..height {
            for x in 0..width {
                let entity = commands.spawn((
                    Tile {
                        tile_type: TileType::Empty,
                    },
                    Transform::from_xyz(
                        x as f32 * TILE_SIZE,
                        TILE_SIZE / 2.0,
                        y as f32 * TILE_SIZE,
                    ),
                ));

                entities.push(entity.id());
            }
        }

        Self {
            width,
            height,
            grid_entities: entities,
            player_spawn_points: vec![],
        }
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn spawn_points(&self) -> &[IVec2] {
        &self.player_spawn_points
    }

    pub fn set_spawn_points(&mut self, spawn_points: Vec<IVec2>) {
        self.player_spawn_points = spawn_points;
    }

    pub fn center(&self) -> Vec2 {
        Vec2::new(self.width() as f32 / 2.0, self.height() as f32 / 2.0)
    }

    pub fn contains(&self, pos: IVec2) -> bool {
        pos.x >= 0 && pos.x < self.width() as i32 && pos.y >= 0 && pos.y < self.height() as i32
    }

    pub fn get_index_from_position(&self, pos: Vec2) -> Option<IVec2> {
        let pos = pos.round();
        let idx = IVec2::new(pos.x as i32, pos.y as i32);
        if self.contains(idx) {
            Some(idx)
        } else {
            None
        }
    }

    fn index_from_position(&self, pos: IVec2) -> usize {
        (pos.y as usize * self.width() as usize) + pos.x as usize
    }

    pub fn pos_iter(&self) -> impl Iterator<Item = IVec2> {
        let width = self.width() as i32;
        let height = self.height() as i32;
        (0..width).flat_map(move |x| (0..height).map(move |y| IVec2::new(x, y)))
    }

    pub fn is_edge(&self, pos: IVec2) -> bool {
        pos.x == 0
            || pos.y == 0
            || pos.x == self.width() as i32 - 1
            || pos.y == self.height() as i32 - 1
    }
}

impl std::ops::Index<IVec2> for Map {
    type Output = Entity;

    fn index(&self, pos: IVec2) -> &Self::Output {
        &self.grid_entities[self.index_from_position(pos)]
    }
}

impl std::ops::IndexMut<IVec2> for Map {
    fn index_mut(&mut self, pos: IVec2) -> &mut Self::Output {
        let idx = self.index_from_position(pos);
        &mut self.grid_entities[idx]
    }
}
