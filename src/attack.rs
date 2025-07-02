use bevy::prelude::*;

use rand::prelude::*;
use rand::rng;

use crate::{
    movement::{Destination, MovementOrder},
    npc::Size,
    player::Player,
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

/// What to do when your target is underneath you
#[derive(Component, Debug)]
pub enum TargetUnderBehavior {
    MoveOut,
    RandomCardinal,
    StayStill,
}

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
        app.add_systems(
            Update,
            simultaneous_check.in_set(FreeRoamSet::SimultaneousAttackChecks),
        )
        .add_systems(
            Update,
            first_check_in_range.in_set(FreeRoamSet::FirstAttackChecks),
        )
        .add_systems(
            Update,
            second_check_in_range.in_set(FreeRoamSet::SecondAttackChecks),
        )
        .add_systems(Update, check_in_range.in_set(EditingSet::AttackChecks))
        .add_systems(
            EditingCatchup,
            simultaneous_check.in_set(EditingCatchupSet::SimultaneousAttackChecks),
        )
        .add_systems(
            EditingCatchup,
            first_check_in_range.in_set(EditingCatchupSet::FirstAttackChecks),
        )
        .add_systems(
            EditingCatchup,
            second_check_in_range.in_set(EditingCatchupSet::SecondAttackChecks),
        );
    }
}

fn simultaneous_check(
    mut commands: Commands,
    player_query: Query<Entity, With<Player>>,
    query: Query<(&Transform, &Target, &AttackRange, &Size), With<Player>>,
    transforms: Query<&Transform>,
    sizes: Query<&Size>,
) -> Result {
    // There's a weird special case where the player moves simultaneously with npcs if:
    // - Their range is 1
    // - Their dist is (2,2) or one of their x/y dist is 1 and the other is 3
    //
    // This actually seems to happen in more cases than just this, but it seems like a pain to
    // actually figure out all the details of and implement right now so I'm just going to leave it
    // as it is for now.
    let entity = player_query.single()?;
    let Ok((transform, target, range, size)) = query.single() else {
        // No target, just default to second
        commands.entity(entity).insert(MovementOrder::Second);
        return Ok(());
    };

    let target_sw_tile = transforms.get(target.0)?;
    let target_size = sizes.get(target.0)?;

    let dist = prv_distance_to_entity(
        transform.translation.truncate(),
        size.0,
        target_sw_tile.translation.truncate(),
        target_size.0,
    );

    let order = if range.0 == 1
        && (dist.abs() == Vec2::splat(2.)
            || dist.abs() == Vec2::new(1., 3.)
            || dist.abs() == Vec2::new(3., 1.))
    {
        MovementOrder::First
    } else {
        MovementOrder::Second
    };

    commands.entity(entity).insert(order);

    Ok(())
}

fn first_check_in_range(
    mut commands: Commands,
    query: Query<(
        Entity,
        &Transform,
        &Target,
        &AttackRange,
        &Size,
        &TargetUnderBehavior,
        &MovementOrder,
    )>,
    transforms: Query<&Transform>,
    sizes: Query<&Size>,
) -> Result {
    for entry in query.iter() {
        if entry.6 == &MovementOrder::First {
            prv_check_in_range(
                &mut commands,
                (entry.0, entry.1, entry.2, entry.3, entry.4, entry.5),
                transforms,
                sizes,
            )?;
        }
    }

    Ok(())
}

fn second_check_in_range(
    mut commands: Commands,
    query: Query<(
        Entity,
        &Transform,
        &Target,
        &AttackRange,
        &Size,
        &TargetUnderBehavior,
        &MovementOrder,
    )>,
    transforms: Query<&Transform>,
    sizes: Query<&Size>,
) -> Result {
    for entry in query.iter() {
        if entry.6 == &MovementOrder::Second {
            prv_check_in_range(
                &mut commands,
                (entry.0, entry.1, entry.2, entry.3, entry.4, entry.5),
                transforms,
                sizes,
            )?;
        }
    }

    Ok(())
}

fn check_in_range(
    mut commands: Commands,
    query: Query<(
        Entity,
        &Transform,
        &Target,
        &AttackRange,
        &Size,
        &TargetUnderBehavior,
    )>,
    transforms: Query<&Transform>,
    sizes: Query<&Size>,
) -> Result {
    for entry in query.iter() {
        prv_check_in_range(&mut commands, entry, transforms, sizes)?;
    }

    Ok(())
}

fn prv_check_in_range(
    commands: &mut Commands,
    entry: (
        Entity,
        &Transform,
        &Target,
        &AttackRange,
        &Size,
        &TargetUnderBehavior,
    ),
    transforms: Query<&Transform>,
    sizes: Query<&Size>,
) -> Result {
    let (entity, transform, target, range, size, under_behavior) = entry;
    let target_sw_tile = transforms.get(target.0)?;
    let target_size = sizes.get(target.0)?;
    let dist = prv_distance_to_entity(
        transform.translation.truncate(),
        size.0,
        target_sw_tile.translation.truncate(),
        target_size.0,
    );

    let destination: Option<Vec2>;
    if dist.x.abs() > range.0 as f32
        || dist.y.abs() > range.0 as f32
        // special case for when range is 1
        || (dist.x.abs() == 1. && dist.y.abs() == 1. && range.0 == 1)
    {
        // Out of range, move towards target
        destination = Some(prv_closest_tile_to_entity(
            transform.translation.truncate(),
            size.0,
            target_sw_tile.translation.truncate(),
            target_size.0,
        ));
    } else if dist == Vec2::ZERO {
        // under the entity
        destination = match under_behavior {
            TargetUnderBehavior::MoveOut => Some(prv_closest_tile_to_entity(
                transform.translation.truncate(),
                size.0,
                target_sw_tile.translation.truncate(),
                target_size.0,
            )),
            TargetUnderBehavior::RandomCardinal => {
                // Nothing is going to have speed > 10 so this is probably fine
                let directions = [
                    Vec2::new(-10., 0.),
                    Vec2::new(10., 0.),
                    Vec2::new(0., -10.),
                    Vec2::new(0., 10.),
                ];
                // TODO: In editing catchup the npc moves erratically because it's not deterministic
                // random. Should probably use bevy_rand.
                let mut rng = rng();
                let Some(direction) = directions.choose(&mut rng) else {
                    panic!("SHOULD ALWAYS GET DIRECTION");
                };
                Some(Vec2::new(
                    transform.translation.x + direction.x,
                    transform.translation.y + direction.y,
                ))
            }
            TargetUnderBehavior::StayStill => None,
        };
    } else {
        // In range! No longer need any destination
        destination = None;
    }
    if let Some(dest) = destination {
        commands.entity(entity).insert(Destination(dest));
    } else {
        commands.entity(entity).try_remove::<Destination>();
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
