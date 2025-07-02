use std::ops::DerefMut;

use bevy::{
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    prelude::*,
};

/// How zoomed by default
const DEFAULT_PROJ_SCALE: f32 = 0.05;

const PROJ_SCALE_MIN: f32 = 0.005;
const PROJ_SCALE_MAX: f32 = 0.1;

/// The amount to zoom per scroll line unit
const SCROLL_LINE_SCALE: f32 = 1.1;

/// The amount to zoom per scroll pixel unit
const SCROLL_PIXEL_SCALE: f32 = 1.001;

/// The speed of panning using keyboard keys
const KEY_PAN_SPEED: f32 = 4.0;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, zoom_camera)
            .add_systems(Update, pan_camera);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Msaa::Sample2,
        Projection::Orthographic(OrthographicProjection {
            scale: DEFAULT_PROJ_SCALE,
            ..OrthographicProjection::default_2d()
        }),
    ));
}

fn zoom_camera(mut query: Query<&mut Projection>, mut mouse_scroll_evr: EventReader<MouseWheel>) {
    for scroll in mouse_scroll_evr.read() {
        let mut projection = query.single_mut().expect("SHOULD BE ONE PROJECTION");

        match projection.deref_mut() {
            Projection::Orthographic(ortho) => {
                let scale = match scroll.unit {
                    // e.g. hardware mouse wheels
                    MouseScrollUnit::Line => SCROLL_LINE_SCALE,
                    // e.g. trackpad scrolling
                    MouseScrollUnit::Pixel => SCROLL_PIXEL_SCALE,
                };

                // Do one operation per scroll amount
                for _ in 0..scroll.y.abs() as u32 {
                    if scroll.y.is_sign_positive() {
                        ortho.scale /= scale;
                    } else {
                        ortho.scale *= scale;
                    }
                }

                // Clamp the zoom between some reasonable values
                ortho.scale = ortho.scale.clamp(PROJ_SCALE_MIN, PROJ_SCALE_MAX);
            }
            _ => panic!("SHOULD ONLY HAVE ORTHO!"),
        }
    }
}

fn pan_camera(
    mouse_input: Res<ButtonInput<MouseButton>>,
    key_input: Res<ButtonInput<KeyCode>>,
    mut motion_evr: EventReader<MouseMotion>,
    mut query: Query<(&mut Projection, &mut Transform), With<Camera>>,
) {
    let mut delta = Vec2::ZERO;

    if mouse_input.pressed(MouseButton::Middle) {
        for motion in motion_evr.read() {
            delta += motion.delta;
        }
    } else {
        if key_input.pressed(KeyCode::KeyW) || key_input.pressed(KeyCode::ArrowUp) {
            delta.y += KEY_PAN_SPEED;
        }
        if key_input.pressed(KeyCode::KeyA) || key_input.pressed(KeyCode::ArrowLeft) {
            delta.x += KEY_PAN_SPEED;
        }
        if key_input.pressed(KeyCode::KeyS) || key_input.pressed(KeyCode::ArrowDown) {
            delta.y -= KEY_PAN_SPEED;
        }
        if key_input.pressed(KeyCode::KeyD) || key_input.pressed(KeyCode::ArrowRight) {
            delta.x -= KEY_PAN_SPEED;
        }
    }

    if delta == Vec2::ZERO {
        return;
    }

    let (mut projection, mut transform) = query.single_mut().expect("SHOULD BE ONE OF THESE");

    // We don't actually change the projection here but we kind of are in a sense so deref_mut() to
    // trigger change detection
    match projection.deref_mut() {
        Projection::Orthographic(ortho) => {
            transform.translation.x -= delta.x * ortho.scale;
            transform.translation.y += delta.y * ortho.scale;
        }
        _ => panic!("SHOULD ONLY HAVE ORTHO!"),
    }
}
