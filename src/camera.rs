use bevy::prelude::*;

/// How many pixels equals one world unit
const PIXEL_SCALE: f32 = 60.0;

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
        Projection::Orthographic(OrthographicProjection {
            scale: PIXEL_SCALE,
            ..OrthographicProjection::default_2d()
        }),
    ));
}
