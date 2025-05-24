use bevy::{math::ops::abs, prelude::*};

use crate::{game_ticks::GameTickEvent, schedule::FreeRoamSet};

#[derive(Component, Default, Debug)]
#[require(Speed)]
pub struct Destination(pub Vec2);

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
        while abs(abs(distance.x) - abs(distance.y)) > 0. && speed_left > 0 {
            if abs(distance.x) > abs(distance.y) {
                transform.translation.x += distance.x.signum();
            } else {
                transform.translation.y += distance.y.signum();
            }

            distance = destination.0 - transform.translation.truncate();
            speed_left -= 1;
        }

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
