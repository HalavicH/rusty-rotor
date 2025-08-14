use bevy::prelude::*;
use bevy_drone_sim::rotating_cube_plugin::{Cube, RotatingCubePlugin};
use bevy_drone_sim::save_system_plugin::SaveSystemPlugin;
use bevy_drone_sim::save_system_plugin::{ApplyFlow, CanSaveLoad, CaptureFlow};
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_save::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // bevy inspector plugins
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        // Save system
        .add_plugins(SaveSystemPlugin)
        .add_flows(CaptureFlow, save_cube)
        .add_flows(ApplyFlow, load_cube)
        .add_plugins(RotatingCubePlugin)
        // types
        .register_type::<Cube>()
        .register_type::<CubePrefab>()
        .run();
}

fn save_cube(In(cap): In<Builder>, world: &World) -> Builder {
    cap.scope(world, |b| {
        b.extract_all_prefabs::<CubePrefab>().clear_empty()
    })
}

fn load_cube(In(apply): In<Applier<'static>>, world: &mut World) -> Applier<'static> {
    apply.scope(world, |a| {
        a.prefab::<CubePrefab>()
            // Despawn previous entities with the `CanSaveLoad` component
            .despawn::<With<CanSaveLoad>>()
    })
}

#[derive(Reflect, Default)]
struct CubePrefab {
    color: Color,
    transform: Transform,
    rotation_speed_coef: f32,
}

impl Prefab for CubePrefab {
    type Marker = Cube;

    /// What components will be added to the entity when it's loaded from the save file.
    fn spawn(self, target: Entity, world: &mut World) {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let handle = meshes.add(Cuboid::default()); // TODO: Looks like bad idea to create a new mesh every time. How to reuse it?

        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let mat_handle = materials.add(self.color); // Looks like a bad idea to create a new material every time. TODO: How to reuse it?

        world.entity_mut(target).insert((
            Cube {
                rotation_speed_coef: self.rotation_speed_coef,
            },
            Name::new("Cube"),
            Mesh3d(handle),
            MeshMaterial3d(mat_handle),
            self.transform,
        ));
    }

    /// Extracts serializable data from the entity to be saved, as we can't save assets directly.
    fn extract(builder: BuilderRef) -> BuilderRef {
        let world = builder.world();
        builder.extract_prefab(|entity| {
            let materials_asset_server = world.resource::<Assets<StandardMaterial>>();

            let material_handle = entity.get::<MeshMaterial3d<StandardMaterial>>()?.0.clone();
            let material = materials_asset_server.get(&material_handle)?;
            let transform = entity.get::<Transform>().cloned().unwrap_or_default();
            Some(CubePrefab {
                color: material.base_color,
                transform,
                rotation_speed_coef: entity
                    .get::<Cube>()
                    .map_or(1.0, |cube| cube.rotation_speed_coef),
            })
        })
    }
}
