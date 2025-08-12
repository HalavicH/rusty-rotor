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
        // Game types
        .register_type::<PlayerDrone>()
        .register_type::<DronePosition>()
        // Game resources
        // Game systems
        .add_systems(Startup, (setup, setup_ui, spawn_stick_position_ui))
        .add_systems(
            Update,
            (
                update_drone_controls,
                update_drone_controls_ui,
                update_stick_position,
            ),
        )
        .add_systems(Update, rotate_drone_system)
        // .add_systems(Update, list_gamepads)
        .run();
    info!("App exited with: {:?}", exit);
}

#[derive(Debug, Clone, Copy, Component)]
enum StickSideUi {
    Left,
    Right,
}

/// This system spawns a UI node/2d sprites to display the stick position.
fn spawn_stick_position_ui(
    mut commands: Commands,
    // Add other resources if needed
) {
    commands.spawn((
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::End,
            column_gap: Val::Px(5.),
            ..Default::default()
        },
        Text::new("Stick Position".to_string()),
        Name::new("Gamepad Stick Position UI"),
        children![
            build_stick_ui(StickSideUi::Left),
            build_stick_ui(StickSideUi::Right),
        ],
    ));
}

fn build_stick_ui(side: StickSideUi) -> impl Bundle {
    (
        Name::new(format!("{side:?}: Stick: Box")),
        Text::new(format!("{side:?}: Stick: Box")),
        Node {
            width: Val::Px(200.),
            height: Val::Px(200.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..Default::default()
        },
        BackgroundColor(Color::Srgba(Srgba::new(0.1, 0.1, 0.1, 0.5))),
        children![(
            Name::new(format!("{side:?}: Stick: Dot")),
            side,
            Node {
                width: Val::Percent(5.),
                height: Val::Percent(5.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            BackgroundColor(Color::Srgba(Srgba::new(0.5, 0.5, 0.0, 0.9))),
            BorderRadius {
                top_left: Val::Percent(50.),
                top_right: Val::Percent(50.),
                bottom_left: Val::Percent(50.),
                bottom_right: Val::Percent(50.),
            },
        )],
    )
}

fn update_stick_position(
    controls: Query<&DronePosition, With<PlayerDrone>>,
    mut query: Query<(&mut Node, &StickSideUi)>,
) {
    let Some(controls) = controls.single().ok() else {
        info!("No drone controls found.");
        return;
    };

    for (mut node, side) in query.iter_mut() {
        match side {
            StickSideUi::Left => {
                // x-axis
                node.left = Val::Percent(100. * controls.yaw / 2.);
                // y-axis
                node.bottom = Val::Percent((100. * controls.throttle) - 50.);
            }
            StickSideUi::Right => {
                // x-axis
                node.left = Val::Percent(100. * controls.roll / 2.);
                // y-axis
                node.bottom = Val::Percent(100. * controls.pitch / 2.);
            }
        }
    }
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
            Name::new("Drone Controls Text"),
            Text::new("Drone Controls: "),
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
        controls.throttle, controls.pitch, controls.roll, controls.yaw
    );
}
#[derive(Debug, Default, Component, Reflect)]
pub struct DronePosition {
    pub throttle: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
}

fn update_drone_controls(
    gamepad: Query<&Gamepad>,
    mut controls: Query<&mut DronePosition, With<PlayerDrone>>,
) {
    let Ok(gamepad) = gamepad.single() else {
        debug!("No gamepad found for drone controls.");
        return;
    };
    let Ok(mut controls) = controls.single_mut() else {
        debug!("No drone controls found.");
        return;
    };

    let left_stick = gamepad.left_stick();
    let right_stick = gamepad.right_stick();
    let right_trigger2 = gamepad
        .get(GamepadInput::Button(GamepadButton::RightTrigger2))
        .unwrap_or(0.);

    controls.throttle = right_trigger2;
    controls.yaw = right_stick.x;
    controls.pitch = -left_stick.y;
    controls.roll = left_stick.x;
}

fn rotate_drone_system(mut drone: Query<(&mut Transform, &DronePosition), With<PlayerDrone>>) {
    let Ok((mut transform, drone_controls)) = drone.single_mut() else {
        debug!("No drone entity found.");
        return;
    };

    trace!("{drone_controls:?}");
    // Get the gamepad's left stick input.
    let DronePosition {
        throttle: thrust,
        pitch,
        roll,
        yaw,
    } = drone_controls;

    let coef = 1.0;
    // Apply rotation based on the gamepad input.
    transform.rotation = Quat::from_euler(EulerRot::YXZ, -yaw * coef, pitch * coef, roll * coef);
    transform.translation.y = *thrust;
}

#[derive(Component, Reflect)]
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
