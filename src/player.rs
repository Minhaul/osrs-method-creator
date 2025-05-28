use bevy::prelude::*;

use crate::movement::{Destination, Speed};
use crate::npc::Npc;
use crate::schedule::{EditingSet, FreeRoamSet};
use crate::state::ToolState;

#[derive(Component, Default, Debug)]
pub struct Player;

#[derive(Component, Default, Debug)]
struct DestinationMarker;

#[derive(Default, Debug, Clone)]
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
                (despawn_player, spawn_player).chain(),
            )
            .add_systems(
                Update,
                (update_action, update_modifiers, highlight_destination)
                    .in_set(FreeRoamSet::EntityUpdates),
            )
            .add_systems(
                Update,
                (update_action, update_modifiers, highlight_destination)
                    .in_set(EditingSet::EntityUpdates),
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
    ));
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

fn despawn_player(mut commands: Commands, mut query: Query<Entity, With<Player>>) {
    let Ok(entity) = query.single_mut() else {
        panic!("MORE THAN ONE PLAYER????");
    };

    commands.entity(entity).despawn();
}

fn update_action(
    mut commands: Commands,
    mut player_action_evr: EventReader<PlayerActionEvent>,
    mut query: Query<Entity, With<Player>>,
) {
    for player_action_event in player_action_evr.read() {
        let Ok(entity) = query.single_mut() else {
            panic!("MORE THAN ONE PLAYER????");
        };

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
    let Ok(mut speed) = query.single_mut() else {
        panic!("PLAYER HAS NO SPEED???");
    };

    speed.0 = if player_modifiers.run { 2 } else { 1 };
}

fn highlight_destination(
    mut marker_query: Query<(&mut Transform, &mut Visibility), With<DestinationMarker>>,
    player_query: Query<&Destination, With<Player>>,
) {
    let Ok((mut destination_marker_transform, mut destination_marker_visibility)) =
        marker_query.single_mut()
    else {
        panic!("NO DESTINATION MARKER ENTITY???");
    };

    // Only highlight the destination if there is one
    let Ok(Destination(location)) = player_query.single() else {
        *destination_marker_visibility = Visibility::Hidden;
        return;
    };
    destination_marker_transform.translation.x = location.x;
    destination_marker_transform.translation.y = location.y;

    *destination_marker_visibility = Visibility::Visible;
}
