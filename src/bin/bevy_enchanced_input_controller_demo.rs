//! This demo evaluates the `bevy_enhanced_input` crate.
//!
//! # Demo Description
//! Two contexts (`SquareContext`, `RectangleContext`) define identical actions (MoveX, MoveY).
//! Press **R** to toggle the active context.
//! Use **WASD** to move the currently active shape.
//!
//! Limitations:
//! - actions and contexts are tied to one entity (`Controller`).
//! - all contexts are active by default (todo: research if it's possible to have them inactive by default)

use bevy::prelude::*;
use bevy_drone_sim::free_camera_plugin::FreeCameraPlugin;
use bevy_enhanced_input::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    App::new()
        // Standard Bevy plugins
        .add_plugins(DefaultPlugins)
        // Debug plugins
        .add_plugins((EguiPlugin::default(), WorldInspectorPlugin::new()))
        // Camera plugin for free flight
        .add_plugins(FreeCameraPlugin)
        // Bevy enhanced input plugin
        .add_plugins(EnhancedInputPlugin)
        .add_input_context::<SquareMode>()
        .add_input_context::<RectangleMode>()
        .add_observer(move_square)
        .add_observer(rotate_square)
        .add_observer(switch_to_rectangle)
        .add_observer(move_rectangle)
        .add_observer(rotate_rectangle)
        .add_observer(switch_to_square)
        // Startup systems
        .add_systems(Startup, (spawn_ui, setup))
        .add_systems(Update, update_ui)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn a 2D camera
    commands.spawn(Camera2d);

    // Spawn a controller entity
    commands
        .spawn((
            Controller,
            Name::new("Controller"),
            SquareMode,
            RectangleMode,
            // Square actions
            actions!(
                SquareMode[
                    (
                        Action::<MoveSquare>::new(),
                        DeltaScale,
                        Bindings::spawn((
                            Cardinal::wasd_keys(),
                            Axial::left_stick(),
                        )),
                    ),
                    (
                        Action::<RotateSquare>::new(),
                        DeltaScale,
                        Bindings::spawn((
                            Bidirectional {
                                positive: Binding::from(KeyCode::KeyE),
                                negative: Binding::from(KeyCode::KeyQ),
                            },
                            Axial::right_stick(),
                        )),
                    ),
                    (
                        Action::<SwitchToRectangle>::new(),
                        // We set `require_reset` to `true` because `SwitchToSquare` action uses the same input,
                        // and we want it to be triggerable only after the button is released.
                        ActionSettings {
                            require_reset: true,
                            ..Default::default()
                        },
                        Bindings::spawn(Spawn(Binding::from(KeyCode::KeyR)))
                    )
                ]
            ),
            // Rectangle actions (same as Square, different marker, can be generalized)
            actions!(
                RectangleMode[
                    (
                        Action::<MoveRectangle>::new(),
                        DeltaScale,
                        Bindings::spawn((
                            Cardinal::wasd_keys(),
                            Axial::left_stick(),
                        )),
                    ),
                    (
                        Action::<RotateRectangle>::new(),
                        DeltaScale,
                        Bindings::spawn((
                            Bidirectional {
                                positive: Binding::from(KeyCode::KeyE),
                                negative: Binding::from(KeyCode::KeyQ),
                            },
                            Axial::right_stick(),
                        )),
                    ),
                    (
                        Action::<SwitchToSquare>::new(),
                        ActionSettings {
                            require_reset: true,
                            ..Default::default()
                        },
                        Bindings::spawn(Spawn(Binding::from(KeyCode::KeyR)))
                    )
                ]
            ),
        ))
        // By default, all contexts are active. Inactivating one of them
        .insert(ContextActivity::<RectangleMode>::INACTIVE);

    // Spawn a square
    commands.spawn((
        Square,
        Name::new("Square"),
        Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(100.0, 100.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(-200.0, 0.0, 0.0)),
    ));

    // Spawn a rectangle
    commands.spawn((
        Rectangle,
        Name::new("Rectangle"),
        Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(100.0, 200.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(200.0, 0.0, 0.0)),
    ));
}

fn move_square(
    trigger: Trigger<Fired<MoveSquare>>,
    mut transforms: Query<&mut Transform, With<Square>>,
) {
    info!("Applying movement: {:#?}", trigger);
    let mut transform = transforms.single_mut().unwrap();

    // Move to the camera direction.
    let rotation = transform.rotation;

    // Converting the Vec2 to Vec3 by adding a Z component of 0.0
    // and multiplying by a scale factor of 100.0 for better visibility.
    let movement = trigger.value.extend(0.0).xyz() * Vec3::splat(100.0);
    info!("Movement before negation: {:#?}", movement);

    transform.translation += rotation * movement
}

fn rotate_square(
    trigger: Trigger<Fired<RotateSquare>>,
    mut transforms: Query<&mut Transform, With<Square>>,
) {
    info!("Applying rotation: {:#?}", trigger);
    let mut transform = transforms.single_mut().unwrap();

    // Rotate around the Z axis.
    let rotation = Quat::from_rotation_z(trigger.value.x);

    // Apply the rotation to the current transform.
    transform.rotation *= rotation;
}

fn switch_to_rectangle(trigger: Trigger<Started<SwitchToRectangle>>, mut commands: Commands) {
    info!("Switching to Rectangle context: {:#?}", trigger);
    commands.entity(trigger.target()).insert((
        ContextActivity::<SquareMode>::INACTIVE,
        ContextActivity::<RectangleMode>::ACTIVE,
    ));
}

fn move_rectangle(
    trigger: Trigger<Fired<MoveRectangle>>,
    mut transforms: Query<&mut Transform, With<Rectangle>>,
) {
    info!("Applying movement: {:#?}", trigger);
    let mut transform = transforms.single_mut().unwrap();

    // Move to the camera direction.
    let rotation = transform.rotation;

    // Converting the Vec2 to Vec3 by adding a Z component of 0.0
    // and multiplying by a scale factor of 100.0 for better visibility.
    let movement = trigger.value.extend(0.0).xyz() * Vec3::splat(100.0);
    info!("Movement before negation: {:#?}", movement);

    transform.translation += rotation * movement
}

fn rotate_rectangle(
    trigger: Trigger<Fired<RotateRectangle>>,
    mut transforms: Query<&mut Transform, With<Rectangle>>,
) {
    info!("Applying rotation: {:#?}", trigger);
    let mut transform = transforms.single_mut().unwrap();

    // Rotate around the Z axis.
    let rotation = Quat::from_rotation_z(trigger.value.x);

    // Apply the rotation to the current transform.
    transform.rotation *= rotation;
}

fn switch_to_square(trigger: Trigger<Started<SwitchToSquare>>, mut commands: Commands) {
    info!("Switching to Square context: {:#?}", trigger);
    commands.entity(trigger.target()).insert((
        ContextActivity::<RectangleMode>::INACTIVE,
        ContextActivity::<SquareMode>::ACTIVE,
    ));
}

fn spawn_ui(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::End,
            align_items: AlignItems::Start,
            ..default()
        },
        children![
            (Node::default(), Text::new("Bevy Enhanced Input Demo")),
            (
                Node::default(),
                Text::new("Press 'R' to toggle input contexts")
            ),
            (
                ContextActivityMarker,
                Text::new("Active context: SquareMode or RectangleMode")
            )
        ],
    ));
}

/// Prints active context
fn update_ui(
    mut text_to_change: Query<&mut Text, With<ContextActivityMarker>>,
    contexts: Query<
        (
            &ContextActivity<RectangleMode>,
            &ContextActivity<SquareMode>,
        ),
        With<Controller>,
    >,
) {
    if let Ok((rectangle_context, square_context)) = contexts.single() {
        let active_context = if **rectangle_context {
            "RectangleMode"
        } else if **square_context {
            "SquareMode"
        } else {
            "None"
        };

        // Update the UI text with the active context
        if let Ok(mut text) = text_to_change.single_mut() {
            text.0 = format!("Active context: {active_context}");
        }
    } else {
        warn!("No controller found or multiple controllers exist.");
    }
}

/// Controller marker component.
/// This entity holds the input actions and their bindings.
/// It can be switched between different contexts (SquareMode, RectangleMode).
/// SquareMode and RectangleMode are input contexts.
/// They define different sets of actions that can be triggered.
#[derive(Component)]
struct Controller;

#[derive(Component)]
struct ContextActivityMarker;

///// Modes (contexts) for input actions /////
#[derive(Component)]
struct SquareMode;

#[derive(Component)]
struct RectangleMode;

///// Maker components for shapes /////
#[derive(Component)]
struct Square;

#[derive(Component)]
struct Rectangle;

///// Actions for the Square context /////
#[derive(InputAction)]
#[action_output(Vec2)]
struct MoveSquare;

#[derive(InputAction)]
#[action_output(Vec2)]
struct RotateSquare;

#[derive(InputAction)]
#[action_output(Vec2)]
struct MoveRectangle;

///// Actions for the Rectangle context /////
#[derive(InputAction)]
#[action_output(Vec2)]
struct RotateRectangle;

#[derive(InputAction)]
#[action_output(bool)]
struct SwitchToSquare;

#[derive(InputAction)]
#[action_output(bool)]
struct SwitchToRectangle;
