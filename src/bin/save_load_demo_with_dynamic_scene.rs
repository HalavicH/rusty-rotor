use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use std::fs::File;
use std::io::Write;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .init_resource::<AppTypeRegistry>()
        .add_systems(Startup, setup)
        .add_systems(Update, (rotate_cube, move_cube))
        // .register_type::>()
        // Save specific stuff
        .register_type::<Cube>()
        .register_type::<Transform>()
        .register_type::<Mesh3d>()
        .add_systems(Update, (save_on_input, load_on_input))
        .run();
}

// The initial scene file will be loaded below and not change when the scene is saved
// const SCENE_FILE_PATH: &str = "scenes/load_scene_example.scn.ron";

// The new, updated scene data will be saved here so that you can see the changes
const NEW_SCENE_FILE_PATH: &str = "scenes/load_scene_example-new.scn.ron";

fn save_on_input(
    type_registry: Res<AppTypeRegistry>,
    keyboard: Res<ButtonInput<KeyCode>>,
    persistent_entities: Query<(&Cube, &Persistent, &Transform, &Mesh3d)>,
) {
    if !(keyboard.just_pressed(KeyCode::KeyS)) {
        return;
    }

    info!("Saving world...");
    // Save cube
    let mut scene_world = World::new();
    scene_world.insert_resource(type_registry.clone());
    // let mut persistent_entities = world.query::<(&Cube, &Persistent, &Transform, &Mesh3d)>();

    for (_, _, t, m) in persistent_entities {
        // Spawn the cube in the new world
        scene_world.spawn((
            Cube, Persistent, *t,
            // m.clone(), // Mesh3d component
        ));
    }
    info!("Spawned entities in the new world");

    // The TypeRegistry resource contains information about all registered types (including
    // components). This is used to construct assets.
    // let type_registry = world.resource::<AppTypeRegistry>();
    let scene = DynamicScene::from_world(&scene_world);
    info!("Created dynamic scene from world");

    // Scenes can be serialized like this:
    let read_guard = type_registry.read();
    let serialized_scene = scene
        .serialize(&read_guard)
        .inspect_err(|e| {
            error!("Error while serializing scene: {e}");
        })
        .unwrap();

    info!("Serialized scene to RON format");

    // Showing the scene in the console
    info!("{}", serialized_scene);

    // Writing the scene to a new file. Using a task to avoid calling the filesystem APIs in a system
    // as they are blocking
    // This can't work in WASM as there is no filesystem access
    #[cfg(not(target_arch = "wasm32"))]
    IoTaskPool::get()
        .spawn(async move {
            // Write the scene RON data to file
            File::create(format!("assets/{NEW_SCENE_FILE_PATH}"))
                .and_then(|mut file| file.write(serialized_scene.as_bytes()))
                .inspect_err(|e| {
                    error!("Error while writing scene to file: {e}");
                })
                .ok()
        })
        .detach();
    info!("World saved!");
}

fn load_on_input(world: &mut World) {
    let keyboard = world.resource::<ButtonInput<KeyCode>>();
    if keyboard.just_pressed(KeyCode::KeyL) {
        info!("World loaded!");
        // commands.spawn(Scene);
    }
}

/// A component that marks an entity as persistent, meaning it can be saved and loaded
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Persistent;

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Cube;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    trace!("Setting up the scene...");
    // Spawn a cube to rotate.
    commands.spawn((
        Persistent,
        Cube,
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_translation(Vec3::ZERO),
    ));

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
