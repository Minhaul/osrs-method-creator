use bevy::prelude::*;

use crate::{
    attack::{AttackRange, Target},
    movement::{Destination, MovementType, Speed},
    player::Player,
};

#[derive(Component, Debug, Clone, PartialEq)]
#[require(Transform, Speed)]
pub struct Npc {
    pub name: String,
    // TODO: encode attacks w/ range and cds, maybe list with conditions based on range
}

/// Size of the entity (always a square so 1 dimension)
#[derive(Component, Debug, Clone, PartialEq)]
pub struct Size(pub u8);

impl Default for Size {
    fn default() -> Self {
        Self(1)
    }
}

#[derive(Component, Default, Debug)]
#[allow(unused)]
struct NpcDestinationMarker;

pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_npc, spawn_npc_destination))
            .add_systems(PostStartup, target_player);
        #[cfg(feature = "debug")]
        app.add_systems(Update, highlight_npc_destination);
    }
}

fn spawn_npc(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let size = 5;
    commands.spawn((
        Npc {
            name: String::from("test"),
        },
        Transform::from_translation(Vec3::new(1., 1., 0.)),
        Visibility::Visible,
        Size(size),
        AttackRange(1),
        MovementType::DiagonalFirst,
        children![(
            Mesh2d(meshes.add(Rectangle::new(size as f32, size as f32))),
            MeshMaterial2d(materials.add(Color::srgb(1., 0.5, 0.5))),
            Transform::from_translation(Vec3::new(
                (size as f32 - 1.) / 2.,
                (size as f32 - 1.) / 2.,
                0.
            )),
        )],
    ));
}

fn spawn_npc_destination(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        NpcDestinationMarker,
        Mesh2d(meshes.add(Rectangle::new(1., 1.))),
        MeshMaterial2d(materials.add(Color::srgb(0.7, 0.5, 0.5))),
        Transform::from_translation(Vec3::new(1., 1., 0.)),
        Visibility::Hidden,
    ));
}

fn target_player(
    mut commands: Commands,
    npc_query: Query<Entity, With<Npc>>,
    player_query: Query<Entity, With<Player>>,
) {
    let player = player_query.single().expect("SHOULD BE ONE PLAYER");

    for npc in npc_query.iter() {
        commands.entity(npc).insert(Target(player));
    }
}

#[allow(unused)]
fn highlight_npc_destination(
    mut marker_query: Query<(&mut Transform, &mut Visibility), With<NpcDestinationMarker>>,
    npc_query: Query<&Destination, With<Npc>>,
) {
    let (mut destination_marker_transform, mut destination_marker_visibility) = marker_query
        .single_mut()
        .expect("SHOULD BE ONE DESTINATION MARKER");

    // Only highlight the destination if there is one
    let Ok(Destination(location)) = npc_query.single() else {
        *destination_marker_visibility = Visibility::Hidden;
        return;
    };
    destination_marker_transform.translation.x = location.x;
    destination_marker_transform.translation.y = location.y;

    *destination_marker_visibility = Visibility::Visible;
}
