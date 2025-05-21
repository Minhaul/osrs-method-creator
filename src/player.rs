use bevy::prelude::*;

use crate::npc::Npc;
use crate::schedule::RunningSet;
use crate::movement::{Destination, Speed};

#[derive(Component, Default, Debug)]
struct Player;

#[derive(Default, Debug, Clone)]
pub enum Action {
    #[default]
    Idle,
    Move(Vec2),
    Attack(Npc),
}

#[derive(Event, Default, Debug)]
pub struct PlayerActionEvent {
    pub action: Action,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerActionEvent>()
            .add_systems(Startup, spawn_player)
            .add_systems(Update, update_action.in_set(RunningSet::EntityUpdates));
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

fn update_action(
    mut commands: Commands,
    mut player_action_event_reader: EventReader<PlayerActionEvent>,
    mut query: Query<Entity, With<Player>>,
) {
    for player_action_event in player_action_event_reader.read() {
        let Ok(entity) = query.single_mut() else {
            panic!("MORE THAN ONE PLAYER????");
        };

        // Overwrite the current action with the most recent one
        match &player_action_event.action {
            Action::Move(destination) => commands.entity(entity).insert(Destination(*destination)),
            Action::Attack(_) => commands.entity(entity).insert(Destination(Vec2::ZERO)), // TODO
            Action::Idle => commands.entity(entity).try_remove::<Destination>(),
        };
    }
}
