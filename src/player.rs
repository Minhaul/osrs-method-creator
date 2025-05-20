use bevy::prelude::*;

use crate::{npc::Npc, utils::Coord};

#[derive(Component, Default, Debug)]
struct Player {
    location: Coord,
    current_action: Action,
}

#[derive(Default, Debug)]
enum Action {
    #[default]
    Idle,
    Move(Coord),
    Attack(Npc),
}
