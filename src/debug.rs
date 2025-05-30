use bevy::prelude::*;
use std::time::Duration;

#[allow(unused)]
use crate::state::{EditingState, ToolState};

#[derive(Resource, Default, Debug)]
#[allow(unused)]
struct StartTime(Duration);

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, _app: &mut App) {
        #[cfg(feature = "debug")]
        _app.insert_resource(StartTime::default())
            .add_systems(
                OnExit(EditingState::Editing),
                log_catchup_start_time.run_if(in_state(ToolState::Editing)),
            )
            .add_systems(
                Update,
                log_catchup_end_time.run_if(in_state(EditingState::Editing)),
            );
    }
}

#[allow(unused)]
fn log_catchup_start_time(time: Res<Time>, mut start_time: ResMut<StartTime>) {
    start_time.0 = time.elapsed();
}

#[allow(unused)]
fn log_catchup_end_time(time: Res<Time>, mut start_time: ResMut<StartTime>) {
    if start_time.0 != Duration::default() {
        debug!("catchup frame time: {:?}", time.elapsed() - start_time.0);
        start_time.0 = Duration::default();
    }
}
