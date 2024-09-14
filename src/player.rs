use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum PlayerController {
    #[default]
    KeyboardArrows,
    KeyboardWASD,
    Gamepad(usize),
}

#[derive(Component)]
pub struct Alive {
}

#[derive(Component)]
pub struct Player {
    pub controller: PlayerController,
    pub num_bombs: i32,
    pub firepower: i32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            controller: PlayerController::default(),
            num_bombs: 1,
            firepower: 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerAction {
    None,
    DropBomb,
}

#[derive(Clone, Copy, Debug)]
pub struct PlayerControl {
    pub motion: Vec2,
    pub action: PlayerAction,
}
