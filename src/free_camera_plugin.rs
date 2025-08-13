use bevy::prelude::*;

/// Used to fly around a scene with a free camera.
pub struct FreeCameraPlugin;

impl Plugin for FreeCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_input)
            .add_systems(Startup, setup_ui)
            .add_systems(Update, (toggle_controls, update_ui))
            .init_resource::<FreeCameraMode>()
            .register_type::<FreeCameraMode>();
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

#[derive(Component, Reflect)]
#[reflect(Component)]
struct FreeCameraUiSwitchMarker;

fn setup_ui(
    mut commands: Commands,
) {
    commands.spawn((
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            align_items: AlignItems::End, // Right
            justify_content: JustifyContent::Start, // Top
            flex_direction: FlexDirection::Column,
           ..default()
        },
        Text::new(""),
        children![
            (
                Node {
                    width: Val::Px(300.),
                    height: Val::Px(50.),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                Text::new("Free Camera Controls: Press 'F' to toggle controls"),
            ),
            (
                FreeCameraUiSwitchMarker,
                Node {
                    width: Val::Px(300.),
                    height: Val::Px(50.),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                Text::new("Free Camera: Disabled"),
            )
        ]
    ));
    info!("Free camera UI marker spawned. Press 'F' to toggle controls.");
}

fn update_ui(
    free_camera_mode: Res<FreeCameraMode>,
    mut query: Query<&mut Text, With<FreeCameraUiSwitchMarker>>,
) {
    if let Ok(mut text) = query.single_mut() {
        text.0 = format!(
            "Free Camera: {}",
            if free_camera_mode.enabled {
                "Enabled"
            } else {
                "Disabled"
            }
        );
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
    free_camera_mode: ResMut<FreeCameraMode>,
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

    let movement = {
        let mut movement = Vec3::ZERO;

        // Forward / backward
        if keyboard.pressed(KeyCode::KeyW) {
            movement.z += 1.0;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            movement.z -= 1.0;
        }

        // Left / right
        if keyboard.pressed(KeyCode::KeyA) {
            movement.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            movement.x += 1.0;
        }

        // Ascend / descend
        if keyboard.pressed(KeyCode::KeyQ) {
            movement.y -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyE) {
            movement.y += 1.0;
        }

        movement
    };

    if movement != Vec3::ZERO {
        debug!("Moving camera with free camera plugin: {:?}", movement);
        // Normalize to keep speed consistent
        let movement = movement.normalize();

        // Rotate local movement vector into world space, ignoring vertical for forward/strafe
        let forward = transform.forward();
        let right = transform.right();
        let up = Vec3::Y;

        let world_movement = movement.z * forward + movement.x * right + movement.y * up;

        transform.translation += world_movement * free_camera_mode.speed_mps * time.delta_secs();
    }
}
