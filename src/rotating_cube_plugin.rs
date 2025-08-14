use crate::save_system_plugin::CanSaveLoad;
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiContexts;
use bevy_inspector_egui::egui;
/// Used for demo purposes, this plugin creates a rotating cube in a Bevy application.
pub struct RotatingCubePlugin;

impl Plugin for RotatingCubePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, rotate_cube)
            .add_systems(Update, render_egui_slider)
            .register_type::<Cube>();
    }
}

fn render_egui_slider(mut contexts: EguiContexts, mut rotation_speed_query: Query<&mut Cube>) {
    let Ok(mut slider_value) = rotation_speed_query.single_mut() else {
        debug!("No cube entity found to update the slider value.");
        return;
    };
    //  thread 'Compute Task Pool (3)' panicked at /Users/oleksandrkholiavko/.cargo/registry/src/index.crates.io-
    //  1949cf8c6b5b557f/egui-0.31.1/src/context.rs:1026:22:
    //  No fonts available until first call to Context::run()
    //  note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
    //  Encountered a panic in system `bevy_drone_sim::rotating_cube_plugin::render_egui_slider`!
    //  Encountered a panic in system `bevy_app::main_schedule::Main::run_main`!
    if true {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        info!("No Egui context found to render the slider.");
        return;
    };
    egui::Window::new("Cube Controls").show(ctx, |ui| {
        ui.label("Adjust the value:");

        // Slider that updates SliderValue
        let slider = egui::Slider::new(&mut slider_value.rotation_speed_coef, 0.0..=1.0);
        ui.add(slider.text("Value"));

        // Show the current value
        ui.label(format!("Current: {:.2}", slider_value.rotation_speed_coef));
    });
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
#[require(CanSaveLoad)] // For save/load functionality, this component is required.
pub struct Cube {
    /// Coeficient for the rotation speed. At 1.0, the cube rotates at 90 degrees per second.
    pub rotation_speed_coef: f32,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    trace!("Setting up the scene...");
    // Spawn a cube to rotate.
    commands.spawn((
        Name::new("Rotating Cube"),
        Cube {
            rotation_speed_coef: 1.0,
        },
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
            color: Color::Srgba(Srgba::new(1.0, 1.0, 0.0, 1.0)),
            illuminance: 2000.0,
            shadows_enabled: false,
            affects_lightmapped_mesh_diffuse: false,
            shadow_depth_bias: 0.0,
            shadow_normal_bias: 0.0,
        },
        Transform::from_xyz(3.0, 3.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn rotate_cube(mut cube_query: Query<(&mut Transform, &Cube)>, time: Res<Time>) {
    let Ok((mut transform, cube)) = cube_query.single_mut() else {
        debug!("No cube entity found.");
        return;
    };

    // Rotate the cube around the Y-axis.
    let delta = time.delta_secs();
    transform.rotate(Quat::from_euler(
        EulerRot::YXZ,
        cube.rotation_speed_coef * delta * std::f32::consts::PI / 2.0, // Rotate at 90 degrees per second
        cube.rotation_speed_coef * delta * std::f32::consts::PI / 2.0, // Rotate at 90 degrees per second
        cube.rotation_speed_coef * delta * std::f32::consts::PI / 2.0, // Rotate at 90 degrees per second
    ))
}
