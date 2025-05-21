use bevy::prelude::*;

/// How zoomed
const PIXEL_SCALE: f32 = 0.1;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Msaa::Off,
        // TODO: remove this and programatically change based on room size
        Projection::Orthographic(OrthographicProjection {
            scale: PIXEL_SCALE,
            ..OrthographicProjection::default_2d()
        }),
    ));
}
