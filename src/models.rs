use bevy::{
    asset::AssetServer,
    gltf::GltfAssetLabel,
    prelude::{AnimationGraph, AnimationNodeIndex},
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub const MODEL_BOMB_PATH: &str = "models/bomb.glb";
pub const MODEL_FIRE_PATH: &str = "models/fire.glb";
pub const MODEL_CUBE_METAL_PATH: &str = "models/cube-metal.glb";
pub const MODEL_CUBE_BRICK_PATH: &str = "models/cube-brick.glb";
pub const MODEL_CUBE_PATH1: &str = "models/cube1.glb";
pub const MODEL_CUBE_PATH2: &str = "models/cube2.glb";
pub const MODEL_CUBE_PATH3: &str = "models/cube3.glb";
pub const MODEL_FLAT_FLAME_PATH: &str = "models/flat-flame.glb";
pub const MODEL_FLAT_BOMB_PATH: &str = "models/flat-bomb.glb";
pub const MODEL_ANIMAL_PATH: [&str; 3] = [
    "models/Colobus_Animations.glb",
    "models/Pudu_Animations.glb",
    "models/Inkfish_Animations.glb",
];

#[derive(Clone, Copy, PartialEq, Eq, Hash, EnumIter, Debug)]
pub enum AnimalAnimation {
    Run = 0,
    Walk = 1,
    Idle = 2,
    Jump = 3,
}

impl AnimalAnimation {
    pub fn to_gltf_asset_label(self) -> GltfAssetLabel {
        match self {
            AnimalAnimation::Run => GltfAssetLabel::Animation(13),
            AnimalAnimation::Walk => GltfAssetLabel::Animation(16),
            AnimalAnimation::Idle => GltfAssetLabel::Animation(8),
            AnimalAnimation::Jump => GltfAssetLabel::Animation(11),
        }
    }

    pub fn to_node_index(self) -> AnimationNodeIndex {
        AnimationNodeIndex::new(self as usize + 1)
    }

    pub fn load_graph(asset_server: &AssetServer, asset_path: &str) -> AnimationGraph {
        let (graph, _id) = AnimationGraph::from_clips(AnimalAnimation::iter().map(|anim| {
            asset_server.load(
                anim.to_gltf_asset_label()
                    .from_asset(asset_path.to_string()),
            )
        }));
        graph
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, EnumIter, Debug)]
pub enum BombAnimation {
    Pulsate = 0,
}

impl BombAnimation {
    pub fn to_gltf_asset_label(self) -> GltfAssetLabel {
        GltfAssetLabel::Animation(self as usize)
    }

    pub fn to_node_index(self) -> AnimationNodeIndex {
        AnimationNodeIndex::new(self as usize + 1)
    }

    pub fn load_graph(asset_server: &AssetServer, asset_path: &str) -> AnimationGraph {
        let (graph, _id) = AnimationGraph::from_clips(BombAnimation::iter().map(|anim| {
            asset_server.load(
                anim.to_gltf_asset_label()
                    .from_asset(asset_path.to_string()),
            )
        }));
        graph
    }
}
