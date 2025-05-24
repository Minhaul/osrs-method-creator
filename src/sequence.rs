use bevy::prelude::*;

use crate::player::{PlayerAction, PlayerActionEvent};
use crate::schedule::EditingSet;
use crate::state::ToolState;

#[derive(Resource, Default, Debug)]
pub struct ActionSequence {
    pub current_tick: usize,
    pub sequence: Vec<PlayerAction>,
}

pub struct SequencePlugin;

impl Plugin for SequencePlugin {
    fn build(&self, app: &mut App) {
        let mut action_sequence = ActionSequence::default();
        action_sequence.sequence.push(PlayerAction::Idle);
        app.insert_resource(action_sequence)
            .add_systems(OnEnter(ToolState::FreeRoam), reset_current_tick)
            .add_systems(
                Update,
                update_current_action.in_set(EditingSet::EntityUpdates),
            );
    }
}

fn reset_current_tick(mut action_sequence: ResMut<ActionSequence>) {
    action_sequence.current_tick = 0;
}

fn update_current_action(
    mut player_action_event_reader: EventReader<PlayerActionEvent>,
    mut action_sequence: ResMut<ActionSequence>,
) {
    for player_action_event in player_action_event_reader.read() {
        let current_tick = action_sequence.current_tick;
        action_sequence.sequence[current_tick] = player_action_event.action.clone();
    }
}

// TODO:
//   - system for moving through the sequence
//   - move through the sequence to the current tick in edit mode
//   - implement playback mode that goes through the whole sequence in real time
