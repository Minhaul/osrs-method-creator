mod camera;
mod game_ticks;
mod input;
mod movement;
mod npc;
mod player;
mod schedule;
mod sequence;
mod state;

use bevy::prelude::*;

use camera::CameraPlugin;
use game_ticks::GameTickPlugin;
use input::UserInputPlugin;
use movement::MovementPlugin;
use player::PlayerPlugin;
use schedule::SchedulePlugin;
use sequence::SequencePlugin;
use state::StatePlugin;

fn main() {
    let mut app = App::new();

    // Bevy builtins
    app.add_plugins(DefaultPlugins);

    // User defined
    app.add_plugins(CameraPlugin)
        .add_plugins(GameTickPlugin)
        .add_plugins(UserInputPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(SchedulePlugin)
        .add_plugins(SequencePlugin)
        .add_plugins(StatePlugin);

    app.run();
}
