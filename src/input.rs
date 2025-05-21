use bevy::{prelude::*, window::PrimaryWindow};

use crate::player::{Action, PlayerActionEvent};
use crate::schedule::RunningSet;

pub struct UserInputPlugin;

impl Plugin for UserInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, mouse_input.in_set(RunningSet::UserInput));
    }
}

fn mouse_input(
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    input: Res<ButtonInput<MouseButton>>,
    mut player_action_event_writer: EventWriter<PlayerActionEvent>,
) {
    if input.pressed(MouseButton::Left) {
        if !(input.just_pressed(MouseButton::Left)) {
            return;
        }

        let Ok(window) = window_query.single() else {
            return;
        };

        let Some(window_position) = window.cursor_position() else {
            return;
        };

        let Ok((camera, camera_transform)) = camera_query.single() else {
            return;
        };

        let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, window_position)
        else {
            return;
        };
 
        player_action_event_writer.write(PlayerActionEvent {
            action: Action::Move(world_position.round()),
        });
    }
}
