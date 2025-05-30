use bevy::{ecs::schedule::ScheduleLabel, prelude::*};

use crate::state::{EditingState, ToolState};

#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub struct EditingCatchup;

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
    SequenceUpdates,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum EditingCatchupChecksSet {
    Checks,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum EditingCatchupSet {
    GameTick,
    EntityUpdates,
    Movement,
    TransitionToChecks,
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
                EditingSet::SequenceUpdates,
            )
                .chain()
                .run_if(in_state(ToolState::Editing))
                .run_if(in_state(EditingState::Editing)),
        )
        .configure_sets(
            EditingCatchup,
            EditingCatchupChecksSet::Checks
                .chain()
                .run_if(in_state(ToolState::Editing))
                .run_if(in_state(EditingState::CatchupChecks)),
        )
        .configure_sets(
            EditingCatchup,
            (
                EditingCatchupSet::GameTick,
                EditingCatchupSet::EntityUpdates,
                EditingCatchupSet::Movement,
                EditingCatchupSet::TransitionToChecks,
            )
                .chain()
                .run_if(in_state(ToolState::Editing))
                .run_if(in_state(EditingState::Catchup)),
        );
    }
}
