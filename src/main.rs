use std::ops::Deref;
use bevy::input::gamepad::GamepadInput;
use bevy::{prelude::*};
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    info!("Starting...");

    let exit = App::new()
        // Bevy Plugins
        .add_plugins(DefaultPlugins)
        // Egui and World Inspector Plugins
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        // Game resources
        .insert_resource(DroneControls::default())
        // Game systems
        .add_systems(Update, update_drone_controls)
        .add_systems(Startup, setup)
        // .add_systems(Update, list_gamepads)
        .add_systems(Update, rotate_drone_system)
        .run();
    info!("App exited with: {:?}", exit);
}

#[derive(Debug, Resource, Default)]
pub struct DroneControls {
    pub thrust: f32,
    pub pitch: f32,
    pub roll: f32,
    pub yaw: f32,
}

fn update_drone_controls(
    gamepad: Query<&Gamepad>,
    mut controls: ResMut<DroneControls>,
) {
    let Ok(gamepad) = gamepad.single() else {
        *controls = DroneControls::default();
        return;
    };

    let left_stick = gamepad.left_stick();
    let right_stick = gamepad.right_stick();
    let right_trigger2 = gamepad.get(GamepadInput::Button(GamepadButton::RightTrigger2)).unwrap_or(0.);

    controls.thrust = right_trigger2;
    controls.pitch = left_stick.y;
    controls.roll = left_stick.x;
    controls.yaw = right_stick.x;
}

fn rotate_drone_system(
    mut drone: Query<&mut Transform, With<Drone>>,
    drone_controls: Res<DroneControls>,
) {
    println!("{:?}", drone_controls);
    // Get the gamepad's left stick input.
    let DroneControls {
        thrust,
        pitch,
        roll,
        yaw,
    } = drone_controls.deref();

    let Ok(mut transform) = drone.single_mut() else {
        println!("No drone entity found.");
        return;
    };

    let coef = 1.0;
    // Apply rotation based on the gamepad input.
    transform.rotation = Quat::from_euler(
        EulerRot::YXZ,
        yaw * coef,
        pitch * coef,
        roll * coef,
    );
    transform.translation.y = *thrust;
}

#[derive(Component)]
struct Drone;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn a cube to rotate.
    commands.spawn((
        Name::new("Drone"),
        Drone,
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_translation(Vec3::ZERO),
    ));

    // Spawn a camera looking at the entities to show what's happening in this example.
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(2.0, 1.0, 5.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
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

// #[allow(dead_code)]
// fn list_gamepads(gamepad: Query<(&Gamepad, &GamepadSettings, &Name)>) {
//     let Ok(gp) = gamepad.single() else {
//         println!("No gamepads connected.");
//         return;
//     };
//     let (g, gs, name) = gp;
//     println!("Currently connected gamepad: {name}");
//     println!("Gamepad: {g:#?},");
// }

