use bevy::{platform::collections::HashMap, prelude::*};

/// Default base tile color
const DEFAULT_COLOR: Color = Color::srgb(0.3, 0.3, 0.3);

/// Coordinate, mostly created for hashing purposes
#[derive(Debug, Hash, PartialEq, Eq, Default, Clone, Copy)]
struct Coord {
    x: isize,
    y: isize,
}

/// Set of all created tile entities
#[derive(Resource, Debug, Default)]
struct WorldTiles {
    tiles: HashMap<Coord, Entity>,
    extents: (Coord, Coord),
}

/// Component defining that a tile entity is marked
#[derive(Component, Debug)]
struct MarkedTile {
    color: Color,
    label: Option<String>,
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(0., 0., 0.)))
            .insert_resource(WorldTiles::default())
            .add_systems(PostStartup, create_grid)
            .add_systems(Update, update_grid);
    }
}

fn create_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut world_tiles: ResMut<WorldTiles>,
    query: Query<(&Projection, &Transform)>,
) -> Result {
    let (Projection::Orthographic(ortho), transform) = query.single()? else {
        panic!("NOT ORTHO???");
    };

    let (neg_extent, pos_extent) = prv_get_extents(&ortho.area, transform);

    for y in neg_extent.y..=pos_extent.y {
        for x in neg_extent.x..=pos_extent.x {
            let id = prv_add_tile(
                &mut commands,
                &mut meshes,
                &mut materials,
                DEFAULT_COLOR,
                Coord { x, y },
            );

            world_tiles.tiles.insert(Coord { x, y }, id);
        }
    }

    world_tiles.extents = (neg_extent, pos_extent);

    Ok(())
}

fn update_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut world_tiles: ResMut<WorldTiles>,
    query: Query<(&Projection, &Transform), Changed<Projection>>,
) {
    let Ok((Projection::Orthographic(ortho), transform)) = query.single() else {
        return;
    };

    let (neg_extent, pos_extent) = prv_get_extents(&ortho.area, transform);

    for y in neg_extent.y..=pos_extent.y {
        for x in neg_extent.x..=pos_extent.x {
            if world_tiles.tiles.contains_key(&Coord { x, y }) {
                continue;
            }

            let id = prv_add_tile(
                &mut commands,
                &mut meshes,
                &mut materials,
                DEFAULT_COLOR,
                Coord { x, y },
            );

            world_tiles.tiles.insert(Coord { x, y }, id);
        }
    }

    let neg_extent = Coord {
        x: isize::min(neg_extent.x, world_tiles.extents.0.x),
        y: isize::min(neg_extent.y, world_tiles.extents.0.y),
    };
    let pos_extent = Coord {
        x: isize::max(pos_extent.x, world_tiles.extents.1.x),
        y: isize::max(pos_extent.y, world_tiles.extents.1.y),
    };

    world_tiles.extents = (neg_extent, pos_extent);
}

fn prv_add_tile(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    base_color: Color,
    coord: Coord,
) -> Entity {
    let offset = rand::random_range(-0.025..=0.025);

    let mut color = base_color.to_srgba();
    color.red += offset;
    color.green += offset;
    color.blue += offset;

    let color = Color::srgb(color.red, color.green, color.blue);
    commands
        .spawn((
            Mesh2d(meshes.add(Rectangle::new(0.95, 0.95))),
            MeshMaterial2d(materials.add(color)),
            Transform::from_translation(Vec3::new(coord.x as f32, coord.y as f32, -0.1)),
        ))
        .with_child(())
        .id()
}

fn prv_get_extents(area: &Rect, transform: &Transform) -> (Coord, Coord) {
    let neg_extent = Coord {
        x: (area.min.x + transform.translation.x - 0.5).floor() as isize,
        y: (area.min.y + transform.translation.y - 0.5).floor() as isize,
    };

    let pos_extent = Coord {
        x: (area.max.x + transform.translation.x + 0.5).ceil() as isize,
        y: (area.max.y + transform.translation.y + 0.5).ceil() as isize,
    };

    (neg_extent, pos_extent)
}
