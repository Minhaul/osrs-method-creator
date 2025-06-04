mod attack;
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
    app.add_plugins(attack::AttackPlugin)
        .add_plugins(camera::CameraPlugin)
        .add_plugins(debug::DebugPlugin)
        .add_plugins(game_ticks::GameTickPlugin)
        .add_plugins(input::UserInputPlugin)
        .add_plugins(movement::MovementPlugin)
        .add_plugins(npc::NpcPlugin)
        .add_plugins(player::PlayerPlugin)
        .add_plugins(schedule::SchedulePlugin)
        .add_plugins(sequence::SequencePlugin)
        .add_plugins(state::StatePlugin);

    app.run();
}
