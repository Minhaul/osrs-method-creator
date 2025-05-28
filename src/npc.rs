use bevy::prelude::*;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct Npc {
    pub name: String,
    // TODO: encode attacks w/ range and cds, maybe list with conditions based on range
}
