use bevy::prelude::*;

use crate::attack::{AttackRange, AttackSpeed, Target};
use crate::input::EditingResetEvent;
use crate::movement::{Destination, MovementType, Speed};
use crate::npc::Size;
use crate::schedule::{EditingCatchup, EditingCatchupSet, EditingSet, FreeRoamSet};

/// Player marker component
#[derive(Component, Default, Debug)]
#[require(Transform, Speed)]
pub struct Player;

/// Marker component for the player's destination tile
#[derive(Component, Default, Debug)]
struct DestinationMarker;

/// What actions a player can take
#[derive(Default, Debug, Clone, PartialEq)]
pub enum PlayerAction {
    #[default]
    Idle,
    Move(Vec2),
    Attack(Entity),
}

impl std::fmt::Display for PlayerAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayerAction::Idle => write!(f, "Idle"),
            PlayerAction::Move(dest) => write!(f, "Move: {dest}"),
            PlayerAction::Attack(target) => write!(f, "Attack {target}"),
        }
    }
}

/// Modifiers to the player's behavior
#[derive(Resource, Debug, Clone)]
pub struct PlayerModifiers {
    pub run: bool,
    pub weapon_speed: u8,
    pub weapon_range: u8,
}

impl Default for PlayerModifiers {
    fn default() -> Self {
        Self {
            run: true,
            weapon_speed: 4,
            weapon_range: 1,
        }
    }
}

/// Event to communicate a desired player action
#[derive(Event, Default, Debug)]
pub struct PlayerActionEvent {
    pub action: PlayerAction,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerModifiers::default())
            .add_event::<PlayerActionEvent>()
            .add_systems(Startup, (spawn_player, spawn_destination))
            .add_systems(
                Update,
                (update_action, update_modifiers, highlight_destination)
                    .in_set(FreeRoamSet::EntityUpdates),
            )
            .add_systems(
                Update,
                (update_action, update_modifiers, highlight_destination)
                    .in_set(EditingSet::EntityUpdates),
            )
            .add_systems(
                Update,
                (despawn_player, spawn_player)
                    .chain()
                    .run_if(on_event::<EditingResetEvent>)
                    .in_set(EditingSet::EntityUpdates),
            )
            .add_systems(
                EditingCatchup,
                (update_action, update_modifiers).in_set(EditingCatchupSet::EntityUpdates),
            );
    }
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mods = PlayerModifiers::default();
    commands.spawn((
        Player,
        Mesh2d(meshes.add(Rectangle::new(1., 1.))),
        MeshMaterial2d(materials.add(Color::srgb(0.5, 1., 0.5))),
        Transform::from_translation(Vec3::ZERO),
        Speed(if mods.run { 2 } else { 1 }),
        AttackSpeed(mods.weapon_speed),
        AttackRange(mods.weapon_range),
        MovementType::CardinalFirst,
        Size(1),
    ));
}

fn despawn_player(mut commands: Commands, mut query: Query<Entity, With<Player>>) {
    let entity = query.single_mut().expect("SHOULD BE ONE PLAYER");

    commands.entity(entity).despawn();
}

fn spawn_destination(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        DestinationMarker,
        Mesh2d(meshes.add(Rectangle::new(1., 1.))),
        MeshMaterial2d(materials.add(Color::srgb(0.5, 0.5, 0.5))),
        Transform::from_translation(Vec3::new(1., 1., 0.)),
        Visibility::Hidden,
    ));
}

fn update_action(
    mut commands: Commands,
    mut player_action_evr: EventReader<PlayerActionEvent>,
    mut query: Query<(Entity, &Transform), With<Player>>,
) {
    for player_action_event in player_action_evr.read() {
        let (entity, transform) = query.single_mut().expect("SHOULD BE ONE PLAYER");

        // Overwrite the current action with the most recent one
        match &player_action_event.action {
            PlayerAction::Move(dest) => {
                if transform.translation.truncate() != *dest {
                    commands.entity(entity).insert(Destination(*dest));
                    commands.entity(entity).try_remove::<Target>();
                } else {
                    // Already at the target destination
                    commands.entity(entity).try_remove::<Destination>();
                    commands.entity(entity).try_remove::<Target>();
                }
            }
            PlayerAction::Attack(target) => {
                commands.entity(entity).insert(Target(*target));
                commands.entity(entity).try_remove::<Destination>();
            }
            PlayerAction::Idle => {
                commands.entity(entity).try_remove::<Destination>();
                commands.entity(entity).try_remove::<Target>();
            }
        };
    }
}

fn update_modifiers(
    player_modifiers: Res<PlayerModifiers>,
    mut query: Query<(&mut Speed, &mut AttackSpeed, &mut AttackRange), With<Player>>,
) {
    let (mut speed, mut attack_speed, mut attack_range) =
        query.single_mut().expect("SHOULD BE ONE PLAYER");

    speed.0 = if player_modifiers.run { 2 } else { 1 };
    attack_speed.0 = player_modifiers.weapon_speed;
    attack_range.0 = player_modifiers.weapon_range;
}

fn highlight_destination(
    mut marker_query: Query<(&mut Transform, &mut Visibility), With<DestinationMarker>>,
    player_query: Query<&Destination, With<Player>>,
) {
    let (mut destination_marker_transform, mut destination_marker_visibility) = marker_query
        .single_mut()
        .expect("SHOULD BE ONE DESTINATION MARKER");

    // Only highlight the destination if there is one
    let Ok(Destination(location)) = player_query.single() else {
        *destination_marker_visibility = Visibility::Hidden;
        return;
    };
    destination_marker_transform.translation.x = location.x;
    destination_marker_transform.translation.y = location.y;

    *destination_marker_visibility = Visibility::Visible;
}
