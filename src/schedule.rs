use bevy::{ecs::schedule::ScheduleLabel, prelude::*};

use crate::state::{EditingState, ToolState};

#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub struct EditingCatchup;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum FreeRoamSet {
    UserInput,
    EntityUpdates,
    GameTick,
    SimultaneousAttackChecks,
    FirstAttackChecks,
    FirstMovement,
    SecondAttackChecks,
    SecondMovement,
    Attacks,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum EditingSet {
    ReconcileSequenceLocation,
    UserInput,
    EntityUpdates,
    AttackChecks,
    SequenceUpdates,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum EditingCatchupChecksSet {
    SequenceChecks,
    SendActions,
    Transition,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum EditingCatchupSet {
    EntityUpdates,
    GameTick,
    SimultaneousAttackChecks,
    FirstAttackChecks,
    FirstMovement,
    SecondAttackChecks,
    SecondMovement,
    Attacks,
    Transition,
}

pub struct SchedulePlugin;

impl Plugin for SchedulePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                FreeRoamSet::UserInput,
                FreeRoamSet::EntityUpdates,
                FreeRoamSet::GameTick,
                FreeRoamSet::SimultaneousAttackChecks,
                FreeRoamSet::FirstAttackChecks,
                FreeRoamSet::FirstMovement,
                FreeRoamSet::SecondAttackChecks,
                FreeRoamSet::SecondMovement,
                FreeRoamSet::Attacks,
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
                EditingSet::AttackChecks,
                EditingSet::SequenceUpdates,
            )
                .chain()
                .run_if(in_state(ToolState::Editing))
                .run_if(in_state(EditingState::Editing)),
        )
        .configure_sets(
            EditingCatchup,
            (
                EditingCatchupChecksSet::SequenceChecks,
                EditingCatchupChecksSet::SendActions,
                EditingCatchupChecksSet::Transition,
            )
                .chain()
                .run_if(in_state(ToolState::Editing))
                .run_if(in_state(EditingState::CatchupChecks)),
        )
        .configure_sets(
            EditingCatchup,
            (
                EditingCatchupSet::EntityUpdates,
                EditingCatchupSet::GameTick,
                EditingCatchupSet::SimultaneousAttackChecks,
                EditingCatchupSet::FirstAttackChecks,
                EditingCatchupSet::FirstMovement,
                EditingCatchupSet::SecondAttackChecks,
                EditingCatchupSet::SecondMovement,
                EditingCatchupSet::Attacks,
                EditingCatchupSet::Transition,
            )
                .chain()
                .run_if(in_state(ToolState::Editing))
                .run_if(in_state(EditingState::Catchup)),
        );
    }
}
