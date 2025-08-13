use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(Startup, setup)
        .add_systems(Update, (rotate_cube, move_cube))
        // Save specific stuff
        .register_type::<Cube>()
        .register_type::<Transform>()
        .add_systems(Update, (save_on_input, load_on_input))
        .run();
}


fn save_on_input(world: &mut World) {
    let keyboard = world.resource::<ButtonInput<KeyCode>>();
    if keyboard.just_pressed(KeyCode::KeyS) {
        info!("World saved!");
    }
}

fn load_on_input(world: &mut World) {
    let keyboard = world.resource::<ButtonInput<KeyCode>>();
    if keyboard.just_pressed(KeyCode::KeyL) {
        info!("World loaded!");
    }
}

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
