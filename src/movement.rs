use bevy::{math::ops::abs, prelude::*};

use crate::{
    game_ticks::GameTickEvent,
    schedule::{EditingCatchup, EditingCatchupSet, FreeRoamSet},
};

/// Component to indicate a desire to move to the given location
#[derive(Component, Default, Debug)]
#[require(Speed)]
pub struct Destination(pub Vec2);

/// Speed in tiles per tick of the Entity
#[derive(Component, Debug)]
pub struct Speed(pub u8);

impl Default for Speed {
    fn default() -> Self {
        Self(1)
    }
}

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            move_entities
                .run_if(on_event::<GameTickEvent>)
                .in_set(FreeRoamSet::EntityUpdates),
        )
        .add_systems(
            EditingCatchup,
            move_entities
                .run_if(on_event::<GameTickEvent>)
                .in_set(EditingCatchupSet::Movement),
        );
    }
}

fn move_entities(
    mut commands: Commands,
    mut query: Query<(Entity, &Destination, &Speed, &mut Transform)>,
) {
    for (entity, destination, speed, mut transform) in query.iter_mut() {
        let mut distance = destination.0 - transform.translation.truncate();

        let mut speed_left = speed.0;

        // Move to the diagonal if not there already
        while abs(abs(distance.x) - abs(distance.y)) > 0. && speed_left > 0 {
            if abs(distance.x) > abs(distance.y) {
                transform.translation.x += distance.x.signum();
            } else {
                transform.translation.y += distance.y.signum();
            }

            distance = destination.0 - transform.translation.truncate();
            speed_left -= 1;
        }

        // Move along the diagonal as much as we can or until we reach the destination
        if speed_left > 0 {
            let to_travel = (speed_left as f32).min(abs(distance.x));
            transform.translation.x += to_travel * distance.x.signum();
            transform.translation.y += to_travel * distance.y.signum();
        }

        if transform.translation.truncate() == destination.0 {
            commands.entity(entity).remove::<Destination>();
        }
    }
}
