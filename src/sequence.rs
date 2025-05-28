use bevy::prelude::*;

use crate::player::{PlayerAction, PlayerActionEvent, PlayerModifiers};
use crate::schedule::EditingSet;

#[derive(Resource, Default, Debug)]
pub struct ActionSequence {
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
        app.insert_resource(action_sequence).add_systems(
            Update,
            (update_current_action, update_current_modifiers).in_set(EditingSet::EntityUpdates),
        );
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

// TODO
/* fn move_forward_one_tick(

) {} */

// TODO:
//   - system for moving through the sequence
//   - move through the sequence to the current tick in edit mode
//   - implement playback mode that goes through the whole sequence in real time
