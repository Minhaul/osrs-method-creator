use bevy::prelude::*;

use crate::utils::Coord;

#[derive(Component, Debug)]
pub struct Npc {
    location: Coord,
    // TODO: encode attacks w/ range and cds, maybe list with conditions based on range
}
