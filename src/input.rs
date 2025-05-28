use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{
    EguiContextPass, EguiContexts, EguiPlugin, egui, input::egui_wants_any_pointer_input,
};

use crate::player::{Player, PlayerAction, PlayerActionEvent, PlayerModifiers};
use crate::schedule::{EditingSet, FreeRoamSet};
use crate::sequence::ActionSequence;
use crate::state::ToolState;

const MAX_UI_TICK_SEQUENCE_LEN: usize = 10;

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

// #[allow(clippy::too_many_arguments)]
fn draw_ui(
    mut contexts: EguiContexts,
    state: Res<State<ToolState>>,
    mut next_state: ResMut<NextState<ToolState>>,
    mut action_sequence: ResMut<ActionSequence>,
    mut player_modifiers: ResMut<PlayerModifiers>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    player_query: Query<&Transform, With<Player>>,
) {
    egui::Window::new("Method Creation")
        .collapsible(false)
        .resizable(false)
        .movable(false)
        .current_pos((0., 0.))
        .min_width(350.)
        .show(contexts.ctx_mut(), |ui| {
            // Mode Selection
            let &(mut desired_state) = state.get();
            ui.horizontal(|ui| {
                ui.selectable_value(&mut desired_state, ToolState::Editing, "Method Creation");
                ui.selectable_value(&mut desired_state, ToolState::FreeRoam, "Free Roam");
            });
            if &desired_state != state.get() {
                next_state.set(desired_state);
            }
            if desired_state != ToolState::Editing {
                return;
            }

            ui.separator();

            // Player Information
            let Ok(current_transform) = player_query.single() else {
                panic!("MORE THAN ONE PLAYER???");
            };
            let current_location = current_transform.translation.truncate();
            let current_action = action_sequence.sequence[action_sequence.current_tick]
                .0
                .clone();

            ui.label(format!("Location: {current_location}"));
            ui.label(format!("Action: {current_action}"));

            // Sequence Modification
            let sequence_len = action_sequence.sequence.len();
            ui.horizontal(|ui| {
                let mut starting_num = 0;
                let mut ending_num = sequence_len;

                if sequence_len > MAX_UI_TICK_SEQUENCE_LEN {
                    if action_sequence.current_tick + (MAX_UI_TICK_SEQUENCE_LEN / 2) < sequence_len
                        && action_sequence.current_tick > (MAX_UI_TICK_SEQUENCE_LEN / 2)
                    {
                        starting_num =
                            action_sequence.current_tick - (MAX_UI_TICK_SEQUENCE_LEN / 2);
                        ending_num = action_sequence.current_tick + (MAX_UI_TICK_SEQUENCE_LEN / 2);
                    } else if action_sequence.current_tick + (MAX_UI_TICK_SEQUENCE_LEN / 2)
                        < sequence_len
                    {
                        ending_num = MAX_UI_TICK_SEQUENCE_LEN;
                    } else if action_sequence.current_tick > (MAX_UI_TICK_SEQUENCE_LEN / 2) {
                        starting_num = sequence_len - MAX_UI_TICK_SEQUENCE_LEN;
                    } else {
                        panic!("SHOULD HAVE COVERED ALL CASES!");
                    }
                }

                ui.add_enabled_ui(action_sequence.current_tick > 0, |ui| {
                    if ui.button("<<").clicked() {
                        action_sequence.current_tick = 0;
                    }
                    if ui.button("<").clicked() && action_sequence.current_tick > 0 {
                        action_sequence.current_tick -= 1;
                    }
                });

                if starting_num > 0 {
                    ui.label("...");
                }
                ui.add_enabled_ui(state.get() == &ToolState::Editing, |ui| {
                    for i in starting_num..ending_num {
                        ui.selectable_value(&mut action_sequence.current_tick, i, format!("{i}"));
                    }
                });
                if ending_num < sequence_len {
                    ui.label("...");
                }

                #[allow(clippy::collapsible_else_if)]
                if action_sequence.current_tick == sequence_len - 1 {
                    if ui.button("+").clicked() {
                        action_sequence
                            .sequence
                            .push((PlayerAction::Idle, player_modifiers.clone()));
                        action_sequence.current_tick += 1;
                    }
                } else {
                    if ui.button(">").clicked() && action_sequence.current_tick < sequence_len - 1 {
                        action_sequence.current_tick += 1;
                    }
                }
                ui.add_enabled_ui(action_sequence.current_tick < sequence_len - 1, |ui| {
                    if ui.button(">>").clicked() {
                        action_sequence.current_tick = sequence_len - 1;
                    }
                });
            });
        });

    let Ok(window) = window_query.single() else {
        panic!("NO WINDOW???");
    };

    let window_width = window.width();

    egui::Window::new("Player Modifiers")
        .collapsible(false)
        .resizable(false)
        .movable(false)
        .current_pos((window_width, 0.))
        .max_width(200.)
        .show(contexts.ctx_mut(), |ui| {
            ui.checkbox(&mut player_modifiers.run, "Run");

            ui.separator();

            ui.label("Weapon");
            ui.add(egui::Slider::new(&mut player_modifiers.weapon_speed, 2..=7).text("Speed"));
            ui.add(egui::Slider::new(&mut player_modifiers.weapon_range, 1..=10).text("Range"));
        });
}

fn mouse_input(
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    input: Res<ButtonInput<MouseButton>>,
    mut player_action_evw: EventWriter<PlayerActionEvent>,
) {
    if !input.just_pressed(MouseButton::Left) {
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

    let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, window_position) else {
        return;
    };

    player_action_evw.write(PlayerActionEvent {
        action: PlayerAction::Move(world_position.round()),
    });
}
