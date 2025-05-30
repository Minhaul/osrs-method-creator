use bevy::prelude::*;

use crate::input::EditingResetEvent;
use crate::player::{Player, PlayerAction, PlayerActionEvent, PlayerModifiers};
use crate::schedule::{EditingCatchup, EditingCatchupChecksSet, EditingCatchupSet, EditingSet};
use crate::state::{EditingState, ToolState};

#[derive(Resource, Default, Debug)]
pub struct ActionSequence {
    pub target_tick: usize,
    pub current_tick: usize,
    pub sequence: Vec<(PlayerAction, PlayerModifiers)>,
}

pub struct SequencePlugin;

impl Plugin for SequencePlugin {
    fn build(&self, app: &mut App) {
        let mut action_sequence = ActionSequence::default();
        action_sequence
            .sequence
            .push((PlayerAction::Idle, PlayerModifiers::default()));
        app.insert_resource(action_sequence)
            .add_systems(
                Update,
                (editing_catchup_setup, run_editing_catchup)
                    .chain()
                    .run_if(in_state(EditingState::Reconciliation)),
            )
            .add_systems(OnExit(ToolState::FreeRoam), reset_current_tick)
            .add_systems(
                Update,
                check_target_tick.in_set(EditingSet::ReconcileSequenceLocation),
            )
            .add_systems(
                Update,
                (update_current_action, update_current_modifiers)
                    .in_set(EditingSet::SequenceUpdates),
            )
            .add_systems(
                Update,
                reset_sequence
                    .run_if(on_event::<EditingResetEvent>)
                    .in_set(EditingSet::SequenceUpdates),
            )
            .add_systems(
                EditingCatchup,
                (send_actions, check_conditions)
                    .chain()
                    .in_set(EditingCatchupChecksSet::Checks),
            )
            .add_systems(
                EditingCatchup,
                transition_to_catchup_checks.in_set(EditingCatchupSet::TransitionToChecks),
            );
    }
}

fn reset_current_tick(mut action_sequence: ResMut<ActionSequence>) {
    action_sequence.current_tick = 0;
}

fn editing_catchup_setup(
    mut action_sequence: ResMut<ActionSequence>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    if action_sequence.target_tick < action_sequence.current_tick {
        // Reset sequence and player location, we have to go back!
        let mut player_transform = query.single_mut().expect("SHOULD BE ONE PLAYER");
        player_transform.translation.x = 0.;
        player_transform.translation.y = 0.;

        action_sequence.current_tick = 0;
    }
}

fn run_editing_catchup(world: &mut World) {
    loop {
        let state = world.resource::<State<EditingState>>();
        match **state {
            EditingState::Reconciliation => {
                // Should only be here at the start of catchup
                world
                    .resource_mut::<NextState<EditingState>>()
                    .set(EditingState::CatchupChecks);
            }
            EditingState::Editing => {
                // Done with catchup, break out of loop
                break;
            }
            _ => (),
        }

        world.run_schedule(EditingCatchup);
        world.run_schedule(StateTransition);
    }
}

fn check_target_tick(
    mut next_state: ResMut<NextState<EditingState>>,
    action_sequence: Res<ActionSequence>,
) {
    if action_sequence.current_tick != action_sequence.target_tick {
        next_state.set(EditingState::Reconciliation);
    }
}

fn update_current_action(
    mut player_action_evr: EventReader<PlayerActionEvent>,
    mut action_sequence: ResMut<ActionSequence>,
) {
    for player_action_event in player_action_evr.read() {
        let current_tick = action_sequence.current_tick;
        action_sequence.sequence[current_tick].0 = player_action_event.action.clone();
    }
}

fn update_current_modifiers(
    mut action_sequence: ResMut<ActionSequence>,
    player_modifiers: Res<PlayerModifiers>,
) {
    let current_tick = action_sequence.current_tick;
    action_sequence.sequence[current_tick].1 = player_modifiers.clone();
}

fn reset_sequence(mut action_sequence: ResMut<ActionSequence>) {
    action_sequence.target_tick = 0;
    action_sequence.current_tick = 0;
    action_sequence.sequence = vec![(PlayerAction::Idle, PlayerModifiers::default())];
    // TODO: Move player if needed, should be safe to do so with set ordering
}

fn send_actions(
    action_sequence: ResMut<ActionSequence>,
    mut player_action_evw: EventWriter<PlayerActionEvent>,
    mut player_modifiers: ResMut<PlayerModifiers>,
) {
    let current_tick = action_sequence.current_tick;
    *player_modifiers = action_sequence.sequence[current_tick].1.clone();

    player_action_evw.write(PlayerActionEvent {
        action: action_sequence.sequence[current_tick].0.clone(),
    });
}

fn check_conditions(
    mut action_sequence: ResMut<ActionSequence>,
    mut next_state: ResMut<NextState<EditingState>>,
    query: Query<&Transform, With<Player>>,
) {
    // Check if the current action is redundant due to changes earlier in the sequence
    let current_tick = action_sequence.current_tick;
    if let PlayerAction::Move(dest) = action_sequence.sequence[current_tick].0 {
        let transform = query.single().expect("SHOULD BE ONE PLAYER");
        if dest == transform.translation.truncate() {
            // Already at target location, so change actions from here on to idle
            let sequence_len = action_sequence.sequence.len();
            for i in current_tick..sequence_len {
                if let PlayerAction::Move(dest2) = action_sequence.sequence[i].0 {
                    if dest != dest2 {
                        break;
                    }

                    action_sequence.sequence[i].0 = PlayerAction::Idle;
                } else {
                    break;
                }
            }
        }
    }

    // Do the proper state transition
    if current_tick >= action_sequence.target_tick {
        next_state.set(EditingState::Editing);
        return;
    } else {
        next_state.set(EditingState::Catchup);
    }

    action_sequence.current_tick += 1;
}

fn transition_to_catchup_checks(mut next_state: ResMut<NextState<EditingState>>) {
    next_state.set(EditingState::CatchupChecks);
}

// TODO:
//   - implement playback mode that goes through the whole sequence in real time
