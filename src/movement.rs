use bevy::prelude::*;

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

#[derive(Component, Debug)]
pub enum MovementType {
    CardinalFirst,
    DiagonalFirst,
}

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
                .in_set(FreeRoamSet::Movement),
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
    mut query: Query<(Entity, &Destination, &Speed, &MovementType, &mut Transform)>,
) {
    for (entity, destination, speed, movement_type, mut transform) in query.iter_mut() {
        let first_movement: fn(Vec2, Vec2, u8) -> (Vec2, u8);
        let second_movement: fn(Vec2, Vec2, u8) -> (Vec2, u8);
        match movement_type {
            MovementType::CardinalFirst => {
                first_movement = prv_move_cardinally;
                second_movement = prv_move_diagonally;
            }
            MovementType::DiagonalFirst => {
                first_movement = prv_move_diagonally;
                second_movement = prv_move_cardinally;
            }
        }

        let (delta_translation, speed_left) =
            first_movement(transform.translation.truncate(), destination.0, speed.0);
        transform.translation.x += delta_translation.x;
        transform.translation.y += delta_translation.y;

        let (delta_translation, _) =
            second_movement(transform.translation.truncate(), destination.0, speed_left);
        transform.translation.x += delta_translation.x;
        transform.translation.y += delta_translation.y;

        if transform.translation.truncate() == destination.0 {
            commands.entity(entity).remove::<Destination>();
        }
    }
}

// Helper to move diagonally, returns the delta movement and speed left
fn prv_move_diagonally(start: Vec2, destination: Vec2, speed: u8) -> (Vec2, u8) {
    let mut distance = destination - start;

    let mut delta_translation = Vec2::ZERO;
    let mut speed_left = speed;

    while distance.x != 0. && distance.y != 0. && speed_left > 0 {
        delta_translation.x += distance.x.signum();
        delta_translation.y += distance.y.signum();

        distance = destination - (start + delta_translation);
        speed_left -= 1;
    }

    (delta_translation, speed_left)
}

// Helper to move cardinally, returns the delta movement and speed left
fn prv_move_cardinally(start: Vec2, destination: Vec2, speed: u8) -> (Vec2, u8) {
    let mut distance = destination - start;

    let mut delta_translation = Vec2::ZERO;
    let mut speed_left = speed;

    while (distance.x.abs() - distance.y.abs()).abs() > 0. && speed_left > 0 {
        if distance.x.abs() > distance.y.abs() {
            delta_translation.x += distance.x.signum();
        } else {
            delta_translation.y += distance.y.signum();
        }

        distance = destination - (start + delta_translation);
        speed_left -= 1;
    }

    (delta_translation, speed_left)
}
