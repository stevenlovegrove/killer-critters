use bevy::prelude::*;
use rand::Rng;

use crate::{models::*, tile::*};

#[derive(Clone)]
pub enum GameAsset {
    None,
    Scene(SceneBundle),
    AnimatedScene((SceneBundle, Handle<AnimationGraph>)),
}

#[derive(Resource)]
pub struct ResourceTileFactory {
    asset_bomb: GameAsset,
    asset_explosion: GameAsset,
    asset_unbreakable_wall: GameAsset,
    asset_breakable_wall: Vec<GameAsset>,
    asset_firepower: GameAsset,
    asset_extrabomb: GameAsset,
}

impl ResourceTileFactory {
    pub fn new(
        animation_graphs: &mut ResMut<Assets<AnimationGraph>>,
        asset_server: &Res<AssetServer>,
    ) -> Self {
        Self {
            asset_unbreakable_wall: GameAsset::Scene(SceneBundle {
                scene: asset_server
                    .load(GltfAssetLabel::Scene(0).from_asset(MODEL_CUBE_BRICK_PATH)),
                ..default()
            }),
            asset_breakable_wall: vec![
                GameAsset::Scene(SceneBundle {
                    scene: asset_server.load(GltfAssetLabel::Scene(0).from_asset(MODEL_CUBE_PATH1)),
                    ..default()
                }),
                // GameAsset::Scene(SceneBundle {
                //     scene: asset_server.load(GltfAssetLabel::Scene(0).from_asset(MODEL_CUBE_PATH2)),
                //     ..default()
                // }),
                // GameAsset::Scene(SceneBundle {
                //     scene: asset_server.load(GltfAssetLabel::Scene(0).from_asset(MODEL_CUBE_PATH3)),
                //     ..default()
                // }),
            ],
            asset_bomb: GameAsset::AnimatedScene((
                SceneBundle {
                    scene: asset_server.load(GltfAssetLabel::Scene(0).from_asset(MODEL_BOMB_PATH)),
                    ..default()
                },
                animation_graphs.add(BombAnimation::load_graph(&asset_server, MODEL_BOMB_PATH)),
            )),
            asset_explosion: GameAsset::AnimatedScene((
                SceneBundle {
                    scene: asset_server.load(GltfAssetLabel::Scene(0).from_asset(MODEL_FIRE_PATH)),
                    ..default()
                },
                animation_graphs.add(BombAnimation::load_graph(&asset_server, MODEL_FIRE_PATH)),
            )),
            asset_firepower: GameAsset::AnimatedScene((
                SceneBundle {
                    scene: asset_server
                        .load(GltfAssetLabel::Scene(0).from_asset(MODEL_FLAT_FLAME_PATH)),
                    ..default()
                },
                animation_graphs.add(BombAnimation::load_graph(
                    &asset_server,
                    MODEL_FLAT_FLAME_PATH,
                )),
            )),
            asset_extrabomb: GameAsset::AnimatedScene((
                SceneBundle {
                    scene: asset_server
                        .load(GltfAssetLabel::Scene(0).from_asset(MODEL_FLAT_BOMB_PATH)),
                    ..default()
                },
                animation_graphs.add(BombAnimation::load_graph(
                    &asset_server,
                    MODEL_FLAT_BOMB_PATH,
                )),
            )),
        }
    }

    pub fn make_tile(&self, tile_type: &TileType) -> GameAsset {
        match tile_type {
            TileType::Empty => GameAsset::None,
            TileType::SolidWall => self.asset_unbreakable_wall.clone(),
            TileType::BreakableWall(_) => {
                let index = rand::thread_rng().gen_range(0..self.asset_breakable_wall.len());
                self.asset_breakable_wall[index].clone()
            }
            TileType::Bomb(_) => self.asset_bomb.clone(),
            TileType::Explosion(_, _) => self.asset_explosion.clone(),
            TileType::PowerUp(PowerUpType::Firepower) => self.asset_firepower.clone(),
            TileType::PowerUp(PowerUpType::ExtraBomb) => self.asset_extrabomb.clone(),
        }
    }
}

pub fn random_orthogonal_rotation() -> Quat {
    use rand::seq::SliceRandom;

    let rotations = vec![
        Quat::IDENTITY,
        Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
        Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
        Quat::from_rotation_x(std::f32::consts::PI),
        Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
        Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2),
        Quat::from_rotation_y(std::f32::consts::PI),
        Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2),
        Quat::from_rotation_z(std::f32::consts::PI),
    ];

    let mut rng = rand::thread_rng();
    *rotations.choose(&mut rng).unwrap()
}
