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
mod world;

#[cfg(feature = "debug")]
use bevy::log::LogPlugin;
use bevy::prelude::*;
#[cfg(feature = "debug")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

/// Scale of gizmo line width, proportional to the projection scale
const GIZMO_LINE_WIDTH_SCALE: f32 = 0.09;

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
        .add_plugins(state::StatePlugin)
        .add_plugins(world::WorldPlugin);

    #[cfg(feature = "debug")]
    app.add_plugins(WorldInspectorPlugin::new());

    app.add_systems(Startup, configure_gizmos)
        .add_systems(Update, update_gizmos);

    app.run();
}

fn configure_gizmos(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();

    config.line.joints = GizmoLineJoint::None;
}

fn update_gizmos(
    mut config_store: ResMut<GizmoConfigStore>,
    query: Query<&Projection, Changed<Projection>>,
) {
    let Ok(Projection::Orthographic(ortho)) = query.single() else {
        return;
    };

    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();

    config.line.width = GIZMO_LINE_WIDTH_SCALE / ortho.scale;
}
