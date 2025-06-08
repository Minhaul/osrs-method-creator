use bevy::prelude::*;

use crate::{
    movement::Destination,
    npc::Size,
    schedule::{EditingCatchup, EditingCatchupSet, EditingSet, FreeRoamSet},
};

/// What entity is being targeted?
#[derive(Component, Debug)]
#[relationship(relationship_target = TargetedBy)]
pub struct Target(pub Entity);

/// What entities are targeting this one?
#[derive(Component, Debug)]
#[relationship_target(relationship = Target)]
pub struct TargetedBy(Vec<Entity>);

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
    query: Query<(Entity, &Transform, &Target, &AttackRange, &Size)>,
    transforms: Query<&Transform>,
    sizes: Query<&Size>,
) -> Result {
    for (entity, transform, target, range, size) in query.iter() {
        let target_sw_tile = transforms.get(target.0)?;
        let target_size = sizes.get(target.0)?;
        let dist = prv_distance_to_entity(
            transform.translation.truncate(),
            size.0,
            target_sw_tile.translation.truncate(),
            target_size.0,
        );

        if dist.x.abs() > range.0 as f32
            || dist.y.abs() > range.0 as f32
            // special case for when range is 1
            || (dist.x.abs() == 1. && dist.y.abs() == 1. && range.0 == 1)
            // under the entity
            // TODO: Handle this separately for npcs and players
            || dist == Vec2::ZERO
        {
            // Out of range, move towards target
            // TODO: Some incorrect interactions when attacking while npc is moving.
            //       Seems that, in game, your player will always path for a melee attack
            //       as if your target has already moved before you on that tick.
            //       EXCEPT when one of x/y dist is 1 and the other is 3. In that case,
            //       your player paths for a melee attack as if the target hasn't moved
            //       before you and you end up under them if they're bigger than 1x1.
            //       Why???? I can't figure it out, because every other tile around those
            //       works normally. But it's a set of conditions I can check for to encode
            //       the edge case so I'll do it.
            commands
                .entity(entity)
                .insert(Destination(prv_closest_tile_to_entity(
                    transform.translation.truncate(),
                    size.0,
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
fn prv_distance_to_entity(
    start_sw_tile: Vec2,
    size: u8,
    target_sw_tile: Vec2,
    target_size: u8,
) -> Vec2 {
    let mut dist = start_sw_tile - target_sw_tile;

    if dist.x.is_sign_negative() {
        // on the west side of the target, take size into account
        dist.x += f32::min(dist.x.abs(), (size - 1) as f32);
    }

    if dist.x.is_sign_positive() {
        // on the east side of the target, take target size into account
        dist.x -= f32::min(dist.x, (target_size - 1) as f32);
    }

    if dist.y.is_sign_negative() {
        // on the south side of the target, take size into account
        dist.y += f32::min(dist.y.abs(), (size - 1) as f32);
    }

    if dist.y.is_sign_positive() {
        // on the north side of the target, take target size into account
        dist.y -= f32::min(dist.y, (target_size - 1) as f32);
    }

    dist
}

// Helper to calculate closest tile to a target entity.
fn prv_closest_tile_to_entity(
    start_sw_tile: Vec2,
    size: u8,
    target_sw_tile: Vec2,
    target_size: u8,
) -> Vec2 {
    let dist = prv_distance_to_entity(start_sw_tile, size, target_sw_tile, target_size);

    let mut dest_tile = Vec2::ZERO;
    if dist == Vec2::ZERO {
        // Under entity, find closest direction to move out
        let mut closest_dist = u32::MAX;

        // Keep the following in this order to prefer directions the same way the game does
        // North
        let temp_dist = (start_sw_tile.y - (target_sw_tile.y + target_size as f32)).abs() as u32;
        if temp_dist <= closest_dist {
            dest_tile.x = start_sw_tile.x;
            dest_tile.y = target_sw_tile.y + target_size as f32;
            closest_dist = temp_dist;
        }

        // South
        let temp_dist = (start_sw_tile.y - (target_sw_tile.y - 1.)).abs() as u32;
        if temp_dist <= closest_dist {
            dest_tile.x = start_sw_tile.x;
            dest_tile.y = target_sw_tile.y - 1.;
            closest_dist = temp_dist;
        }

        // East
        let temp_dist = (start_sw_tile.x - (target_sw_tile.x + target_size as f32)).abs() as u32;
        if temp_dist <= closest_dist {
            dest_tile.x = target_sw_tile.x + target_size as f32;
            dest_tile.y = start_sw_tile.y;
            closest_dist = temp_dist;
        }

        // West
        let temp_dist = (start_sw_tile.x - (target_sw_tile.x - 1.)).abs() as u32;
        if temp_dist <= closest_dist {
            dest_tile.x = target_sw_tile.x - 1.;
            dest_tile.y = start_sw_tile.y;
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
                if target_size > 1 {
                    dest_tile.y = start_sw_tile.y;
                } else {
                    dest_tile.y = target_sw_tile.y;
                }
            } else if dist.y.is_sign_positive() {
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
                if dist.x.abs() == 1. && dist.y.abs() == 1. {
                    // Edge case for when touching diagonally
                    dest_tile.y = start_sw_tile.y;
                } else {
                    dest_tile.y = target_sw_tile.y - 1.;
                }
            }

            // Choose the closest x coord to the start_tile
            if dist.x == 0. {
                if target_size > 1 {
                    dest_tile.x = start_sw_tile.x;
                } else {
                    dest_tile.x = target_sw_tile.x;
                }
            } else if dist.x.is_sign_positive() {
                dest_tile.x = target_sw_tile.x + (target_size - 1) as f32;
            } else {
                dest_tile.x = target_sw_tile.x;
            }
        }
    }

    dest_tile
}
