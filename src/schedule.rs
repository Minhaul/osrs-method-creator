use bevy::prelude::*;

use crate::state::ToolState;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum FreeRoamSet {
    UserInput,
    SpawnEntities,
    EntityUpdates,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum EditingSet {
    UserInput,
    EntityUpdates,
}

pub struct SchedulePlugin;

impl Plugin for SchedulePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                FreeRoamSet::UserInput,
                FreeRoamSet::SpawnEntities,
                FreeRoamSet::EntityUpdates,
            )
                .chain()
                .run_if(in_state(ToolState::FreeRoam)),
        )
        .configure_sets(
            Update,
            (EditingSet::UserInput, EditingSet::EntityUpdates)
                .chain()
                .run_if(in_state(ToolState::Editing)),
        );
    }
}
