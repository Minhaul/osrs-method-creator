use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Running,
    Paused,
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>();
    }
}
