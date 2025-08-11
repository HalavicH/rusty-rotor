use bevy::input::gamepad::GamepadInput;
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    println!("Starting...");

    let exit = App::new()
        // Bevy Plugins
        .add_plugins(DefaultPlugins)
        // Egui and World Inspector Plugins
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        // Game resources
        // Game systems
        .add_systems(Startup, (setup, setup_ui))
        .add_systems(Update, (update_drone_controls, update_drone_controls_ui))
        .add_systems(Update, rotate_drone_system)
        // .add_systems(Update, list_gamepads)
        .run();
    info!("App exited with: {:?}", exit);
}

#[derive(Resource)]
struct DroneControlsText(Entity);

fn setup_ui(mut commands: Commands) {
    let text_entity = commands
        .spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Px(100.),
                align_self: AlignSelf::FlexEnd,
                justify_self: JustifySelf::End,
                ..Default::default()
            },
            (
                Name::new("Drone Controls Text"),
                Text::new("Drone Controls: "),
            ),
        ))
        .id();

    commands.insert_resource(DroneControlsText(text_entity));
}

fn update_drone_controls_ui(
    controls: Query<&DronePosition, With<PlayerDrone>>,
    text_res: Res<DroneControlsText>,
    mut query: Query<&mut Text>,
) {
    let Some(controls) = controls.single().ok() else {
        info!("No drone controls found.");
        return;
    };
    let Some(mut text) = query.get_mut(text_res.0).ok() else {
        info!("No text entity found for drone controls.");
        return;
    };
    text.0 = format!(
        "Thrust: {:.2}\nPitch: {:.2}\nRoll: {:.2}\nYaw: {:.2}",
        controls.thrust, controls.pitch, controls.roll, controls.yaw
    );
}
#[derive(Debug, Default, Component)]
pub struct DronePosition {
    pub thrust: f32,
    pub pitch: f32,
    pub roll: f32,
    pub yaw: f32,
}

fn update_drone_controls(
    gamepad: Query<&Gamepad>,
    mut controls: Query<&mut DronePosition, With<PlayerDrone>>,
) {
    let Ok(gamepad) = gamepad.single() else {
        info!("No gamepad found for drone controls.");
        return;
    };
    let Ok(mut controls) = controls.single_mut() else {
        info!("No drone controls found.");
        return;
    };

    let left_stick = gamepad.left_stick();
    let right_stick = gamepad.right_stick();
    let right_trigger2 = gamepad
        .get(GamepadInput::Button(GamepadButton::RightTrigger2))
        .unwrap_or(0.);

    controls.thrust = right_trigger2;
    controls.pitch = left_stick.y;
    controls.roll = -left_stick.x;
    controls.yaw = -right_stick.x;
}

fn rotate_drone_system(
    mut drone: Query<&mut Transform, With<PlayerDrone>>,
    controls: Query<&DronePosition, With<PlayerDrone>>,
) {
    let Some(drone_controls) = controls.single().ok() else {
        info!("No drone controls found.");
        return;
    };
    info!("{drone_controls:?}");
    // Get the gamepad's left stick input.
    let DronePosition {
        thrust,
        pitch,
        roll,
        yaw,
    } = drone_controls;

    let Ok(mut transform) = drone.single_mut() else {
        info!("No drone entity found.");
        return;
    };

    let coef = 1.0;
    // Apply rotation based on the gamepad input.
    transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw * coef, pitch * coef, roll * coef);
    transform.translation.y = *thrust;
}

#[derive(Component)]
struct PlayerDrone;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn a cube to rotate.
    commands.spawn((
        Name::new("Drone"),
        PlayerDrone,
        DronePosition::default(),
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_translation(Vec3::ZERO),
    ));

    // Spawn a camera looking at the entities to show what's happening in this example.
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(2.0, 1.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
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
