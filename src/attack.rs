use bevy::prelude::*;

use crate::{
    movement::Destination,
    npc::Size,
    schedule::{EditingCatchup, EditingCatchupSet, EditingSet, FreeRoamSet},
};

/// What entity is being targeted?
#[derive(Component, Debug)]
pub struct Target(pub Entity);

/// Speed of the entity's attack in game ticks
#[derive(Component, Debug)]
pub struct AttackSpeed(pub u8);

/// Range of the entity's current attack in tiles
#[derive(Component, Debug)]
pub struct AttackRange(pub u8);

/// Current cooldown until the entity can attack again in game ticks
#[derive(Component, Debug)]
struct Cooldown(u8);

pub struct AttackPlugin;

impl Plugin for AttackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, check_in_range.in_set(FreeRoamSet::AttackChecks))
            .add_systems(Update, check_in_range.in_set(EditingSet::AttackChecks))
            .add_systems(
                EditingCatchup,
                check_in_range.in_set(EditingCatchupSet::AttackChecks),
            );
    }
}

fn check_in_range(
    mut commands: Commands,
    query: Query<(Entity, &Transform, &Target, &AttackRange)>,
    transforms: Query<&Transform>,
    sizes: Query<&Size>,
) -> Result {
    for (entity, transform, target, range) in query.iter() {
        let target_sw_tile = transforms.get(target.0)?;
        let target_size = sizes.get(target.0)?;
        let dist = prv_distance_to_entity(
            transform.translation.truncate(),
            target_sw_tile.translation.truncate(),
            target_size.0,
        );

        if dist.x.abs() > range.0 as f32
            || dist.y.abs() > range.0 as f32
            // special case for when range is 1
            || (dist.x.abs() == 1. && dist.y.abs() == 1. && range.0 == 1)
            // under the entity
            || dist == Vec2::ZERO
        {
            // Out of range, move towards target
            commands
                .entity(entity)
                .insert(Destination(prv_closest_tile_to_entity(
                    transform.translation.truncate(),
                    target_sw_tile.translation.truncate(),
                    target_size.0,
                )));
        } else {
            // In range! No longer need any destination
            commands.entity(entity).try_remove::<Destination>();
        }
    }

    Ok(())
}

// Helper to calculate distance to a target entity.
fn prv_distance_to_entity(start_tile: Vec2, target_sw_tile: Vec2, target_size: u8) -> Vec2 {
    let mut dist = start_tile - target_sw_tile;

    if dist.x.is_sign_positive() {
        // on the east side of the target, take entity size into account
        dist.x -= f32::min(dist.x, (target_size - 1) as f32);
    }

    if dist.y.is_sign_positive() {
        // on the north side of the target, take entity size into account
        dist.y -= f32::min(dist.y, (target_size - 1) as f32);
    }

    dist
}

// Helper to calculate closest tile to a target entity.
fn prv_closest_tile_to_entity(start_tile: Vec2, target_sw_tile: Vec2, target_size: u8) -> Vec2 {
    let dist = prv_distance_to_entity(start_tile, target_sw_tile, target_size);

    let mut dest_tile = Vec2::ZERO;
    if dist == Vec2::ZERO {
        // Under entity, find closest direction to move out
        let mut closest_dist = u32::MAX;

        // Keep the following in this order to prefer directions the same way the game does
        // North
        let temp_dist = (start_tile.y - (target_sw_tile.y + target_size as f32)).abs() as u32;
        if temp_dist <= closest_dist {
            dest_tile.x = start_tile.x;
            dest_tile.y = target_sw_tile.y + target_size as f32;
            closest_dist = temp_dist;
        }

        // South
        let temp_dist = (start_tile.y - (target_sw_tile.y - 1.)).abs() as u32;
        if temp_dist <= closest_dist {
            dest_tile.x = start_tile.x;
            dest_tile.y = target_sw_tile.y - 1.;
            closest_dist = temp_dist;
        }

        // East
        let temp_dist = (start_tile.x - (target_sw_tile.x + target_size as f32)).abs() as u32;
        if temp_dist <= closest_dist {
            dest_tile.x = target_sw_tile.x + target_size as f32;
            dest_tile.y = start_tile.y;
            closest_dist = temp_dist;
        }

        // West
        let temp_dist = (start_tile.x - (target_sw_tile.x - 1.)).abs() as u32;
        if temp_dist <= closest_dist {
            dest_tile.x = target_sw_tile.x - 1.;
            dest_tile.y = start_tile.y;
        }
    } else {
        // Check in this order to prefer east/west movement
        if dist.x.abs() > dist.y.abs() {
            if dist.x.is_sign_positive() {
                // On the east side
                dest_tile.x = target_sw_tile.x + target_size as f32;
            } else {
                // On the west side
                dest_tile.x = target_sw_tile.x - 1.;
            }

            // Choose the closest y coord to the start_tile
            if dist.y == 0. {
                dest_tile.y = start_tile.y;
            } else if dist.y > 0. {
                dest_tile.y = target_sw_tile.y + (target_size - 1) as f32;
            } else {
                dest_tile.y = target_sw_tile.y;
            }
        } else {
            if dist.y.is_sign_positive() {
                // On the north side
                dest_tile.y = target_sw_tile.y + target_size as f32;
            } else {
                // On the south side
                dest_tile.y = target_sw_tile.y - 1.;
            }

            // Choose the closest x coord to the start_tile
            if dist.x == 0. {
                dest_tile.x = start_tile.x;
            } else if dist.x > 0. {
                dest_tile.x = target_sw_tile.x + (target_size - 1) as f32;
            } else {
                dest_tile.x = target_sw_tile.x;
            }
        }
    }

    dest_tile
}
