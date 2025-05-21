use bevy::prelude::*;

use crate::state::GameState;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum RunningSet {
    UserInput,
    SpawnEntities,
    EntityUpdates,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum PausedSet {
    UserInput,
    EntityUpdates,
}

pub struct SchedulePlugin;

impl Plugin for SchedulePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                RunningSet::UserInput,
                RunningSet::SpawnEntities,
                RunningSet::EntityUpdates,
            )
                .chain()
                .run_if(in_state(GameState::Running)),
        )
        .configure_sets(
            Update,
            (PausedSet::UserInput, PausedSet::EntityUpdates)
                .chain()
                .run_if(in_state(GameState::Paused)),
        );
    }
}
