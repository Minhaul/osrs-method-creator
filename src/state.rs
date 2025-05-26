use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum ToolState {
    FreeRoam,
    #[default]
    Editing,
    EditingCatchup,
    Playback,
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ToolState>();
    }
}
