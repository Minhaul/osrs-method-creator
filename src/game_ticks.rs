use bevy::prelude::*;

use crate::schedule::{EditingCatchup, EditingCatchupSet, FreeRoamSet};

const GAME_TICK_SECONDS: f32 = 0.6;

/// Event to declare that a new game tick has started
#[derive(Event, Debug)]
pub struct GameTickEvent;

#[derive(Resource, Debug, Default)]
struct GameTickTimer {
    timer: Timer,
}

pub struct GameTickPlugin;

impl Plugin for GameTickPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameTickTimer {
            timer: Timer::from_seconds(GAME_TICK_SECONDS, TimerMode::Repeating),
        })
        .add_event::<GameTickEvent>()
        .add_systems(Update, game_tick_real_time.in_set(FreeRoamSet::GameTick))
        .add_systems(
            EditingCatchup,
            game_tick_update.in_set(EditingCatchupSet::GameTick),
        );
    }
}

fn game_tick_real_time(
    mut tick_timer: ResMut<GameTickTimer>,
    time: Res<Time>,
    mut game_tick_evw: EventWriter<GameTickEvent>,
) {
    tick_timer.timer.tick(time.delta());

    if tick_timer.timer.finished() {
        game_tick_evw.write(GameTickEvent);
    }
}

fn game_tick_update(mut game_tick_evw: EventWriter<GameTickEvent>) {
    game_tick_evw.write(GameTickEvent);
}
