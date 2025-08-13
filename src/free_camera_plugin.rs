use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;

/// Used to fly around a scene with a free camera.
pub struct FreeCameraPlugin;

impl Plugin for FreeCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_input)
            .add_systems(Startup, setup_ui)
            .add_systems(Update, (toggle_controls, update_ui))
            // Initialize resources
            .init_resource::<FreeCameraMode>()
            .init_resource::<CameraRotation>()
            // Register types for reflection
            .register_type::<FreeCameraMode>()
            .register_type::<CameraRotation>();
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
struct FreeCameraMode {
    /// Whether the free camera controls are enabled.
    enabled: bool,
    /// Speed of the camera movement in meters per second.
    speed_mps: f32,
}

impl Default for FreeCameraMode {
    fn default() -> Self {
        Self {
            enabled: false,
            speed_mps: 5.0,
        }
    }
}

/// Track yaw/pitch for mouse look
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
struct CameraRotation {
    yaw: f32,
    pitch: f32,
    sensitivity: f32,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct FreeCameraUiSwitchMarker;

fn setup_ui(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            align_items: AlignItems::End,           // Right
            justify_content: JustifyContent::Start, // Top
            flex_direction: FlexDirection::Column,
            ..default()
        },
        Text::new(""),
        children![
            (
                Node {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                Text::new("Free Camera Controls: Press 'F' to toggle controls"),
            ),
            (
                FreeCameraUiSwitchMarker,
                Node {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                Text::new("Free Camera: Disabled"),
            )
        ],
    ));
    info!("Free camera UI marker spawned. Press 'F' to toggle controls.");
}

fn update_ui(
    free_camera_mode: Res<FreeCameraMode>,
    mut query: Query<&mut Text, With<FreeCameraUiSwitchMarker>>,
    camera: Query<&Transform, With<Camera>>,
) {
    let Ok(transform) = camera.single() else {
        warn!("No camera found to control with free camera plugin.");
        return;
    };

    if let Ok(mut text) = query.single_mut() {
        let status = format!(
            "Free Camera: {}",
            if free_camera_mode.enabled {
                "Enabled"
            } else {
                "Disabled"
            }
        );
        let position = format!(
            "Camera Position: ({:.2}, {:.2}, {:.2})",
            transform.translation.x, transform.translation.y, transform.translation.z
        );

        let rotation = format!(
            "Camera Looking at: ({:.2}, {:.2}, {:.2})",
            transform.forward().x,
            transform.forward().y,
            transform.forward().z
        );

        text.0 = format!("{status}\n{position}\n{rotation}");
    } else {
        warn!("No UI marker found to update free camera status.");
    }
}

fn toggle_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut free_camera_mode: ResMut<FreeCameraMode>,
) {
    if keyboard.just_pressed(KeyCode::KeyF) {
        free_camera_mode.enabled = !free_camera_mode.enabled;
        info!(
            "Free camera controls are now {}",
            if free_camera_mode.enabled {
                "enabled"
            } else {
                "disabled"
            }
        );
    }
}

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_motion_events: EventReader<MouseMotion>,
    free_camera_mode: ResMut<FreeCameraMode>,
    cam_rot: ResMut<CameraRotation>,
    mut query: Query<&mut Transform, With<Camera>>,
    time: Res<Time>,
) {
    if !free_camera_mode.enabled {
        return;
    }

    let Ok(mut transform) = query.single_mut() else {
        warn!("No camera found to control with free camera plugin.");
        return;
    };

    // --- 1. Mouse look ---
    // for ev in mouse_motion_events.read() {
    //     info!("Free camera controls: Mouse motion detected: {:?}", ev.delta);
    //     cam_rot.yaw -= ev.delta.x * cam_rot.sensitivity * time.delta_secs();
    //     cam_rot.pitch -= ev.delta.y * cam_rot.sensitivity * time.delta_secs();
    //     cam_rot.pitch = cam_rot.pitch.clamp(-1.54, 1.54); // avoid flipping (±~89°)
    // }
    //
    // transform.rotation = Quat::from_axis_angle(Vec3::Y, cam_rot.yaw)
    //     * Quat::from_axis_angle(Vec3::X, cam_rot.pitch);

    // --- 2. Movement ---
    let mut movement = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        movement.z += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        movement.z -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        movement.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        movement.x += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyQ) {
        movement.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyE) {
        movement.y += 1.0;
    }

    if movement != Vec3::ZERO {
        info!(
            "Free camera controls: Moving {:?} at speed {} m/s",
            movement, free_camera_mode.speed_mps
        );
        let movement = movement.normalize();
        let forward = transform.forward();
        let right = transform.right();
        let up = Vec3::Y;
        let world_movement = movement.z * forward + movement.x * right + movement.y * up;
        transform.translation += world_movement * free_camera_mode.speed_mps * time.delta_secs();
    }
}
