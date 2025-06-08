use bevy::prelude::*;

use crate::attack::Target;
use crate::input::EditingResetEvent;
use crate::npc::Npc;
use crate::player::{Player, PlayerAction, PlayerActionEvent, PlayerModifiers};
use crate::schedule::{EditingCatchup, EditingCatchupChecksSet, EditingCatchupSet, EditingSet};
use crate::state::{EditingState, ToolState};

#[derive(Resource, Debug)]
pub struct ActionSequence {
    pub target_tick: usize,
    pub current_tick: usize,
    pub sequence: Vec<(PlayerAction, PlayerModifiers)>,
}

impl Default for ActionSequence {
    fn default() -> Self {
        Self {
            target_tick: Default::default(),
            current_tick: Default::default(),
            sequence: vec![(PlayerAction::Idle, PlayerModifiers::default())],
        }
    }
}

pub struct SequencePlugin;

impl Plugin for SequencePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ActionSequence::default())
            .add_systems(
                Update,
                (setup_sequence, run_editing_catchup)
                    .chain()
                    .run_if(in_state(EditingState::Reconciliation)),
            )
            .add_systems(OnEnter(ToolState::Editing), setup_sequence)
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
                (reset_sequence, setup_sequence)
                    .chain()
                    .run_if(on_event::<EditingResetEvent>)
                    .in_set(EditingSet::SequenceUpdates),
            )
            .add_systems(
                EditingCatchup,
                check_redundancies.in_set(EditingCatchupChecksSet::SequenceChecks),
            )
            .add_systems(
                EditingCatchup,
                send_actions.in_set(EditingCatchupChecksSet::SendActions),
            )
            .add_systems(
                EditingCatchup,
                transition_check.in_set(EditingCatchupChecksSet::Transition),
            )
            .add_systems(
                EditingCatchup,
                transition_to_catchup_checks.in_set(EditingCatchupSet::Transition),
            );
    }
}

#[allow(clippy::type_complexity)]
fn setup_sequence(
    mut commands: Commands,
    mut action_sequence: ResMut<ActionSequence>,
    mut player_query: Query<(Entity, &mut Transform), With<Player>>,
    mut npc_query: Query<(Entity, &mut Transform), (With<Npc>, Without<Player>)>,
    mut player_action_evw: EventWriter<PlayerActionEvent>,
) {
    // Not .expect() ing here because right now bevy runs the StateTransition schedule
    // before PreStartup (i.e. the very first schedule run in the whole app),
    // so this will be run before any of the startup systems get a chance to run.
    let Ok((player, mut player_transform)) = player_query.single_mut() else {
        return;
    };
    // TODO: save a starting location
    player_transform.translation.x = 0.;
    player_transform.translation.y = 0.;

    player_action_evw.write(PlayerActionEvent {
        action: action_sequence.sequence[0].0.clone(),
    });

    for (npc, mut npc_transform) in npc_query.iter_mut() {
        commands.entity(npc).insert(Target(player));

        // TODO: have a starting location saved from some config
        npc_transform.translation.x = 1.;
        npc_transform.translation.y = 1.;
    }

    action_sequence.current_tick = 0;
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
    *action_sequence = ActionSequence::default();
}

fn check_redundancies(
    mut action_sequence: ResMut<ActionSequence>,
    query: Query<&Transform, With<Player>>,
) {
    // Check if the current action is redundant due to changes earlier in the sequence
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
    action_sequence: Res<ActionSequence>,
    mut player_action_evw: EventWriter<PlayerActionEvent>,
    mut player_modifiers: ResMut<PlayerModifiers>,
) {
    let current_tick = action_sequence.current_tick;
    *player_modifiers = action_sequence.sequence[current_tick].1.clone();

    player_action_evw.write(PlayerActionEvent {
        action: action_sequence.sequence[current_tick].0.clone(),
    });
}

fn transition_check(
    mut action_sequence: ResMut<ActionSequence>,
    mut next_state: ResMut<NextState<EditingState>>,
) {
    // Do the proper state transition
    if action_sequence.current_tick >= action_sequence.target_tick {
        next_state.set(EditingState::Editing);
    } else {
        next_state.set(EditingState::Catchup);
        action_sequence.current_tick += 1;
    }
}

fn transition_to_catchup_checks(mut next_state: ResMut<NextState<EditingState>>) {
    next_state.set(EditingState::CatchupChecks);
}

// TODO:
//   - implement playback mode that goes through the whole sequence in real time
