use bevy::prelude::*;

const STARTING_BOMBS: i32 = 1;
const STARTING_FIREPOWER: i32 = 1;

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
    pub player_index: usize,
    pub controller: PlayerController,
    pub num_bombs: i32,
    pub firepower: i32,
}

impl Player {
    pub fn new(player_index: usize, controller: PlayerController) -> Self {
        Self {
            player_index,
            controller,
            num_bombs: STARTING_BOMBS,
            firepower: STARTING_FIREPOWER,
        }
    }

    pub fn reset(&mut self) {
        self.num_bombs = STARTING_BOMBS;
        self.firepower = STARTING_FIREPOWER;
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

impl PlayerControl {
    pub fn is_something(&self) -> bool {
        self.motion != Vec2::ZERO || self.action != PlayerAction::None
    }
}
