use bevy::prelude::*;

use crate::movement::{Destination, Speed};
use crate::npc::Npc;
use crate::schedule::{EditingCatchupSet, EditingSet, FreeRoamSet};
use crate::state::ToolState;

/// Player marker component
#[derive(Component, Default, Debug)]
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
    Attack(Npc),
}

impl std::fmt::Display for PlayerAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayerAction::Idle => write!(f, "Idle"),
            PlayerAction::Move(dest) => write!(f, "Move: {dest}"),
            PlayerAction::Attack(Npc { name, .. }) => write!(f, "Attack: {name}"),
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
                OnExit(ToolState::FreeRoam),
                (despawn_player, spawn_player).chain(),
            )
            .add_systems(
                OnEnter(ToolState::EditingCatchup),
                (despawn_player, spawn_player, hide_player).chain(),
            )
            .add_systems(OnExit(ToolState::EditingCatchup), show_player)
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
                (update_action, update_modifiers).in_set(EditingCatchupSet::EntityUpdates),
            );
    }
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(1., 1.))),
        MeshMaterial2d(materials.add(Color::srgb(0.5, 1., 0.5))),
        Transform::from_translation(Vec3::ZERO),
        Player,
        Speed(2),
        Visibility::Visible,
    ));
}

fn despawn_player(mut commands: Commands, mut query: Query<Entity, With<Player>>) {
    let entity = query.single_mut().expect("SHOULD BE ONE PLAYER");

    commands.entity(entity).despawn();
}

fn show_player(mut query: Query<&mut Visibility, With<Player>>) {
    let mut vis = query.single_mut().expect("SHOULD BE ONE PLAYER");
    *vis = Visibility::Visible;
}

fn hide_player(mut query: Query<&mut Visibility, With<Player>>) {
    let mut vis = query.single_mut().expect("SHOULD BE ONE PLAYER");
    *vis = Visibility::Hidden;
}

fn spawn_destination(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(1., 1.))),
        MeshMaterial2d(materials.add(Color::srgb(0.5, 0.5, 0.5))),
        Transform::from_translation(Vec3::new(1., 1., 0.)),
        DestinationMarker,
        Visibility::Hidden,
    ));
}

fn update_action(
    mut commands: Commands,
    mut player_action_evr: EventReader<PlayerActionEvent>,
    mut query: Query<Entity, With<Player>>,
) {
    for player_action_event in player_action_evr.read() {
        let entity = query.single_mut().expect("SHOULD BE ONE PLAYER");

        // Overwrite the current action with the most recent one
        match &player_action_event.action {
            PlayerAction::Move(dest) => commands.entity(entity).insert(Destination(*dest)),
            // TODO
            PlayerAction::Attack(_) => commands.entity(entity).insert(Destination(Vec2::ZERO)),
            PlayerAction::Idle => commands.entity(entity).try_remove::<Destination>(),
        };
    }
}

fn update_modifiers(
    player_modifiers: Res<PlayerModifiers>,
    mut query: Query<&mut Speed, With<Player>>,
) {
    let mut speed = query.single_mut().expect("SHOULD BE ONE PLAYER WITH SPEED");

    speed.0 = if player_modifiers.run { 2 } else { 1 };
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
