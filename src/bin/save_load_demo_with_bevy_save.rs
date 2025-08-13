use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_save::format::JSONFormat;
use bevy_save::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // bevy_save plugins
        .add_plugins(SavePlugins)
        // Flows
        .add_flows(CaptureFlow, save_cube)
        .add_flows(ApplyFlow, load_cube)
        // bevy inspector plugins
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        // resources
        .init_resource::<AppTypeRegistry>()
        // systems
        .add_systems(Startup, setup)
        .add_systems(Update, (rotate_cube, move_cube))
        .add_systems(Update, handle_save_input)
        // .add_systems(Update, inflate_from_asset_props)
        // types
        .register_type::<Cube>()
        .register_type::<CanSaveLoad>()
        .register_type::<CubePrefab>()
        .run();
}

///// Save and load functionality /////
/// Marks any entity to participate in save/load operations.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CanSaveLoad;

// Pathways look very similar to pipelines, but there are a few key differences
pub struct RONPathway;

impl Pathway for RONPathway {
    // The capture type allows you to save anything you want to disk, even without using reflection
    type Capture = Snapshot;

    type Backend = DefaultDebugBackend;
    type Format = JSONFormat;
    type Key<'a> = &'a str;

    fn key(&self) -> Self::Key<'_> {
        "saves/save_load_demo_with_bevy_save"
    }

    // Instead of capturing and applying directly, now these methods just return labels to user-defined flows
    // This allows for better dependency injection and reduces code complexity
    fn capture(&self, _world: &World) -> impl FlowLabel {
        CaptureFlow
    }

    fn apply(&self, _world: &World) -> impl FlowLabel {
        ApplyFlow
    }
}

// Flow labels don't encode any behavior by themselves, only point to flows
#[derive(Hash, Debug, PartialEq, Eq, Clone, Copy, FlowLabel)]
pub struct CaptureFlow;

#[derive(Hash, Debug, PartialEq, Eq, Clone, Copy, FlowLabel)]
pub struct ApplyFlow;

fn save_cube(In(cap): In<Builder>, world: &World) -> Builder {
    cap.scope(world, |b| {
        b.allow::<CanSaveLoad>()
            .allow::<Name>()
            .allow::<Transform>()
            .allow::<Cube>()
            .allow::<DirectionalLight>()
            .allow::<Camera3d>()
            .extract_entities_matching(|e| {
                // Only entities with the `Persistent` component will be captured
                e.contains::<CanSaveLoad>()
            })
            .extract_all_prefabs::<CubePrefab>()
            .clear_empty()
    })
}

fn load_cube(In(apply): In<Applier<'static>>, world: &mut World) -> Applier<'static> {
    apply.scope(world, |a| {
        // Apply is handled automatically for us

        a.prefab::<CubePrefab>().despawn::<With<CanSaveLoad>>()
    })
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Asset3dProps {
    pub mesh: String,
    pub color: Color,
}

// fn inflate_from_asset_props(
//     mut commands: Commands,
//     meshes: Res<Assets<Mesh>>,
//     materials: Res<Assets<StandardMaterial>>,
//     query: Query<(Entity, &Asset3dProps)>,
// ) {
//     for (entity, props) in query.iter() {
//         if props.mesh == "Cuboid" {
//             commands.entity(entity).insert(Mesh3d(&meshes).add(Cuboid::default()));
//         }
//         if let Some(material) = materials.get(&props.color.into()) {
//             commands.entity(entity).insert(MeshMaterial3d(material.clone()));
//         }
//
//         // delete the Asset3dProps component after inflating
//         commands.entity(entity).remove::<Asset3dProps>();
//     }
// }

// Idea of reusable prefab, which can be used to spawn entities of different types (with different set of components)
// struct MeshPrefab<T> {
//     /// Path to the mesh asset
//     mesh: String,
//     /// Material color
//     color: Color,
//
//     /// Rest of components of generic entity
//     bundle: T,
// }

#[derive(Reflect, Default)]
struct CubePrefab {
    color: Color,
    transform: Transform,
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
            })
        })
    }
}

fn handle_save_input(world: &mut World) {
    let keys = world.resource::<ButtonInput<KeyCode>>();

    if keys.just_released(KeyCode::Enter) {
        info!("Saving data");
        world.save(&RONPathway).expect("Failed to save");
    } else if keys.just_released(KeyCode::Backspace) {
        info!("Loading data");
        world.load(&RONPathway).expect("Failed to load");
    }
}
///// Game components and systems /////
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(CanSaveLoad)]
struct Cube;

fn setup(mut commands: Commands) {
    trace!("Setting up the scene...");
    // Spawn a cube to rotate.
    commands.spawn_prefab(CubePrefab::default());

    // Spawn a camera looking at the entities to show what's happening in this example.
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.0, 2.0, -5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Add a light source so we can see clearly.
    commands.spawn((
        DirectionalLight {
            color: Color::Srgba(Srgba::new(1.0, 0.0, 0.0, 1.0)),
            illuminance: 200.0,
            shadows_enabled: false,
            affects_lightmapped_mesh_diffuse: false,
            shadow_depth_bias: 0.0,
            shadow_normal_bias: 0.0,
        },
        Transform::from_xyz(3.0, 3.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn rotate_cube(
    mut cube: Query<&mut Transform, With<Cube>>,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
) {
    let Ok(mut transform) = cube.single_mut() else {
        debug!("No cube entity found.");
        return;
    };

    let speed = 2.0;
    if input.pressed(KeyCode::KeyX) {
        // Rotate the cube around the X-axis
        transform.rotate(Quat::from_rotation_x(time.delta_secs() * speed));
    }
    if input.pressed(KeyCode::KeyY) {
        // Rotate the cube around the Y-axis
        transform.rotate(Quat::from_rotation_y(time.delta_secs() * speed));
    }
    if input.pressed(KeyCode::KeyZ) {
        // Rotate the cube around the Z-axis
        transform.rotate(Quat::from_rotation_z(time.delta_secs() * speed));
    }
}

fn move_cube(
    mut cube: Query<&mut Transform, With<Cube>>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let Ok(mut transform) = cube.single_mut() else {
        debug!("No cube entity found.");
        return;
    };

    let speed = 2.0; // Speed of movement

    if input.pressed(KeyCode::KeyW) {
        // Move forward
        transform.translation += Vec3::Z * speed * time.delta_secs();
    }
    if input.pressed(KeyCode::KeyS) {
        // Move backward
        transform.translation -= Vec3::Z * speed * time.delta_secs();
    }
    if input.pressed(KeyCode::KeyA) {
        // Move left
        transform.translation -= Vec3::X * speed * time.delta_secs();
    }
    if input.pressed(KeyCode::KeyD) {
        // Move right
        transform.translation += Vec3::X * speed * time.delta_secs();
    }
}
