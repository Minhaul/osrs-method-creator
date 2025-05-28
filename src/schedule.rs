use bevy::prelude::*;

use crate::state::ToolState;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum FreeRoamSet {
    UserInput,
    SpawnEntities,
    // TODO: More granular sets i.e. GameTickUpdates, MovementChanges, Attacks
    EntityUpdates,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum EditingSet {
    ReconcileSequenceLocation,
    UserInput,
    EntityUpdates,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum EditingCatchupSet {
    Events,
    GameTick,
    EntityUpdates,
    Movement,
    Checks,
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
            (
                EditingSet::ReconcileSequenceLocation,
                EditingSet::UserInput,
                EditingSet::EntityUpdates,
            )
                .chain()
                .run_if(in_state(ToolState::Editing)),
        )
        .configure_sets(
            Update,
            (
                EditingCatchupSet::GameTick,
                EditingCatchupSet::EntityUpdates,
                EditingCatchupSet::Movement,
                EditingCatchupSet::Checks,
                EditingCatchupSet::Events,
            )
                .chain()
                .run_if(in_state(ToolState::EditingCatchup)),
        );
    }
}
