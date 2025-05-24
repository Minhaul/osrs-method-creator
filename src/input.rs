use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{
    EguiContextPass, EguiContexts, EguiPlugin, egui, input::egui_wants_any_pointer_input,
};

use crate::player::{Player, PlayerAction, PlayerActionEvent};
use crate::schedule::{EditingSet, FreeRoamSet};
use crate::sequence::ActionSequence;
use crate::state::ToolState;

pub struct UserInputPlugin;

impl Plugin for UserInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_systems(EguiContextPass, draw_ui)
        .add_systems(
            Update,
            mouse_input
                .run_if(not(egui_wants_any_pointer_input))
                .in_set(FreeRoamSet::UserInput),
        )
        .add_systems(
            Update,
            mouse_input
                .run_if(not(egui_wants_any_pointer_input))
                .in_set(EditingSet::UserInput),
        );
    }
}

fn draw_ui(
    mut contexts: EguiContexts,
    mut action_sequence: ResMut<ActionSequence>,
    state: Res<State<ToolState>>,
    mut next_state: ResMut<NextState<ToolState>>,
    player_query: Query<&Transform, With<Player>>,
) {
    egui::Window::new("Tick Sequence").show(contexts.ctx_mut(), |ui| {
        let mut editing = state.get() == &ToolState::Editing;
        ui.checkbox(&mut editing, "Sequence Creation Mode");
        match (state.get(), editing) {
            (ToolState::FreeRoam, true) => next_state.set(ToolState::Editing),
            (ToolState::Editing, false) => next_state.set(ToolState::FreeRoam),
            _ => (),
        };
        if !editing {
            return;
        }

        let sequence_len = action_sequence.sequence.len();

        ui.add(
            egui::Slider::new(&mut action_sequence.current_tick, 0..=sequence_len - 1)
                .text("Current tick"),
        );

        let Ok(current_transform) = player_query.single() else {
            panic!("MORE THAN ONE PLAYER???");
        };
        let current_location = current_transform.translation.truncate();
        let current_action = action_sequence.sequence[action_sequence.current_tick].clone();

        ui.label(format!("Location: {current_location}"));
        ui.label(format!("Action: {current_action}"));

        if ui.button("Add tick").clicked() {
            action_sequence.sequence.push(PlayerAction::Idle);
        }
    });
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
            action: PlayerAction::Move(world_position.round()),
        });
    }
}
