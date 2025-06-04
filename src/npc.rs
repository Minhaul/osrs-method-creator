use bevy::prelude::*;

use crate::movement::Speed;

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

pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_npc);
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
