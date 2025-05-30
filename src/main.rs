mod camera;
mod debug;
mod game_ticks;
mod input;
mod movement;
mod npc;
mod player;
mod schedule;
mod sequence;
mod state;

#[cfg(feature = "debug")]
use bevy::log::LogPlugin;
use bevy::prelude::*;

use camera::CameraPlugin;
use debug::DebugPlugin;
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
    #[cfg(not(feature = "debug"))]
    app.add_plugins(DefaultPlugins);
    #[cfg(feature = "debug")]
    app.add_plugins(DefaultPlugins.set(LogPlugin {
        filter: "info,osrs_method_creator=debug".into(),
        level: bevy::log::Level::DEBUG,
        ..default()
    }));

    // User defined
    app.add_plugins(CameraPlugin)
        .add_plugins(DebugPlugin)
        .add_plugins(GameTickPlugin)
        .add_plugins(UserInputPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(SchedulePlugin)
        .add_plugins(SequencePlugin)
        .add_plugins(StatePlugin);

    app.run();
}
