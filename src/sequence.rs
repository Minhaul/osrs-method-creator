use bevy::prelude::*;

use crate::player::{Player, PlayerAction, PlayerActionEvent, PlayerModifiers};
use crate::schedule::{EditingCatchupSet, EditingSet};
use crate::state::ToolState;

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
            .add_systems(OnEnter(ToolState::EditingCatchup), reset_current_tick)
            .add_systems(
                Update,
                check_target_tick.in_set(EditingSet::ReconcileSequenceLocation),
            )
            .add_systems(
                Update,
                (update_current_action, update_current_modifiers).in_set(EditingSet::EntityUpdates),
            )
            .add_systems(Update, check_conditions.in_set(EditingCatchupSet::Checks))
            .add_systems(Update, send_actions.in_set(EditingCatchupSet::Events));
    }
}

fn reset_current_tick(mut action_sequence: ResMut<ActionSequence>) {
    action_sequence.current_tick = 0;
}

fn check_target_tick(
    mut next_state: ResMut<NextState<ToolState>>,
    action_sequence: Res<ActionSequence>,
) {
    if action_sequence.current_tick != action_sequence.target_tick {
        // TODO: Can we not reset if moving forward? Would be better performance
        next_state.set(ToolState::EditingCatchup);
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

fn check_conditions(
    mut action_sequence: ResMut<ActionSequence>,
    query: Query<&Transform, With<Player>>,
) {
    if action_sequence.current_tick != action_sequence.target_tick {
        return;
    }

    // End of catchup, do some checks
    let current_tick = action_sequence.current_tick;
    if let PlayerAction::Move(dest) = action_sequence.sequence[current_tick].0 {
        let transform = query.single().expect("SHOULD BE ONE PLAYER");
        if dest != transform.translation.truncate() {
            return;
        }

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

fn send_actions(
    mut action_sequence: ResMut<ActionSequence>,
    mut player_action_evw: EventWriter<PlayerActionEvent>,
    mut player_modifiers: ResMut<PlayerModifiers>,
    mut next_state: ResMut<NextState<ToolState>>,
) {
    let current_tick = action_sequence.current_tick;
    *player_modifiers = action_sequence.sequence[current_tick].1.clone();

    player_action_evw.write(PlayerActionEvent {
        action: action_sequence.sequence[current_tick].0.clone(),
    });

    if action_sequence.current_tick == action_sequence.target_tick {
        next_state.set(ToolState::Editing);
        return;
    }

    action_sequence.current_tick += 1;
}

// TODO:
//   - implement playback mode that goes through the whole sequence in real time
