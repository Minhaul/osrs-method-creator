use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{
    EguiContextPass, EguiContexts, EguiPlugin, egui, input::egui_wants_any_pointer_input,
};

use crate::npc::{Npc, Size};
use crate::player::{Player, PlayerAction, PlayerActionEvent, PlayerModifiers};
use crate::schedule::{EditingSet, FreeRoamSet};
use crate::sequence::ActionSequence;
use crate::state::ToolState;

/// Max length of the tick sequence shown in the UI
const MAX_UI_TICK_SEQUENCE_LEN: usize = 10;

#[derive(Event, Default, Debug)]
pub struct EditingResetEvent;

pub struct UserInputPlugin;

impl Plugin for UserInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_event::<EditingResetEvent>()
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

#[allow(clippy::too_many_arguments)]
fn draw_ui(
    mut contexts: EguiContexts,
    state: Res<State<ToolState>>,
    mut next_state: ResMut<NextState<ToolState>>,
    mut editing_reset_evw: EventWriter<EditingResetEvent>,
    mut action_sequence: ResMut<ActionSequence>,
    mut player_modifiers: ResMut<PlayerModifiers>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    player_query: Query<&Transform, With<Player>>,
    mut reset_window: Local<bool>,
) {
    let window = window_query.single().expect("SHOULD BE ONE WINDOW");

    let window_width = window.width();
    let window_heigth = window.height();

    // Method creation UI section
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

            if ui.button("Reset").clicked() || *reset_window {
                *reset_window = true;
            }

            // Player Information
            let current_transform = player_query.single().expect("SHOULD BE ONE PLAYER");
            let current_location = current_transform.translation.truncate();
            let current_action = action_sequence.sequence[action_sequence.target_tick]
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
                    if action_sequence.target_tick + (MAX_UI_TICK_SEQUENCE_LEN / 2) < sequence_len
                        && action_sequence.target_tick > (MAX_UI_TICK_SEQUENCE_LEN / 2)
                    {
                        starting_num = action_sequence.target_tick - (MAX_UI_TICK_SEQUENCE_LEN / 2);
                        ending_num = action_sequence.target_tick + (MAX_UI_TICK_SEQUENCE_LEN / 2);
                    } else if action_sequence.target_tick + (MAX_UI_TICK_SEQUENCE_LEN / 2)
                        < sequence_len
                    {
                        ending_num = MAX_UI_TICK_SEQUENCE_LEN;
                    } else if action_sequence.target_tick > (MAX_UI_TICK_SEQUENCE_LEN / 2) {
                        starting_num = sequence_len - MAX_UI_TICK_SEQUENCE_LEN;
                    } else {
                        panic!("SHOULD HAVE COVERED ALL CASES!");
                    }
                }

                ui.add_enabled_ui(action_sequence.target_tick > 0, |ui| {
                    if ui.button("<<").clicked() {
                        action_sequence.target_tick = 0;
                    }
                    if ui.button("<").clicked() && action_sequence.target_tick > 0 {
                        action_sequence.target_tick -= 1;
                    }
                });

                if starting_num > 0 {
                    ui.label("...");
                }
                ui.add_enabled_ui(state.get() == &ToolState::Editing, |ui| {
                    for i in starting_num..ending_num {
                        ui.selectable_value(&mut action_sequence.target_tick, i, format!("{i}"));
                    }
                });
                if ending_num < sequence_len {
                    ui.label("...");
                }

                #[allow(clippy::collapsible_else_if)]
                if action_sequence.target_tick == sequence_len - 1 {
                    if ui.button("+").clicked() {
                        action_sequence
                            .sequence
                            .extend_from_within(sequence_len - 1..);
                        action_sequence.target_tick += 1;
                    }
                } else {
                    if ui.button(">").clicked() && action_sequence.target_tick < sequence_len - 1 {
                        action_sequence.target_tick += 1;
                    }
                }
                ui.add_enabled_ui(action_sequence.target_tick < sequence_len - 1, |ui| {
                    if ui.button(">>").clicked() {
                        action_sequence.target_tick = sequence_len - 1;
                    }
                });
            });
        });

    // Player modifier UI section
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

    // Sequence reset confirmation window
    if *reset_window {
        egui::Window::new("Reset")
            .collapsible(false)
            .resizable(false)
            .movable(false)
            .current_pos((window_width / 3., window_heigth / 3.))
            .show(contexts.ctx_mut(), |ui| {
                ui.label("Are you sure you want to reset?");
                ui.horizontal(|ui| {
                    if ui.button("Yes").clicked() {
                        editing_reset_evw.write(EditingResetEvent);
                        *reset_window = false;
                    }
                    if ui.button("No").clicked() {
                        *reset_window = false;
                    }
                });
            });
    }
}

fn mouse_input(
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    input: Res<ButtonInput<MouseButton>>,
    query: Query<(Entity, &Transform, &Size), With<Npc>>,
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

    let clicked_tile = world_position.round();

    let mut target = Entity::PLACEHOLDER;
    for (entity, transform, size) in query.iter() {
        if transform.translation.x <= clicked_tile.x
            && clicked_tile.x <= transform.translation.x + (size.0 as f32 - 1.)
            && transform.translation.y <= clicked_tile.y
            && clicked_tile.y <= transform.translation.y + (size.0 as f32 - 1.)
        {
            target = entity;
            break;
        }
    }

    if target != Entity::PLACEHOLDER {
        player_action_evw.write(PlayerActionEvent {
            action: PlayerAction::Attack(target),
        });
    } else {
        player_action_evw.write(PlayerActionEvent {
            action: PlayerAction::Move(world_position.round()),
        });
    }
}
