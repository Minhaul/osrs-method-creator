use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum ToolState {
    FreeRoam,
    #[default]
    Editing,
    Playback,
}

#[derive(SubStates, Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[source(ToolState = ToolState::Editing)]
pub enum EditingState {
    #[default]
    Editing,
    Reconciliation,
    Catchup,
    CatchupChecks,
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ToolState>()
            .add_sub_state::<EditingState>();
    }
}
