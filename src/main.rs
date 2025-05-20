mod camera;
mod npc;
mod player;
mod sequence;
mod utils;

use bevy::prelude::*;

use camera::CameraPlugin;

fn main() {
    let mut app = App::new();

    // Bevy builtins
    app.add_plugins(DefaultPlugins);

    // User defined
    app.add_plugins(CameraPlugin);

    app.run();
}
