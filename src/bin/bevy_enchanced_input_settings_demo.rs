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
        .add_input_context::<InGameContext>()
        .add_input_context::<InMenuContext>()
        .add_observer(move_square)
        .add_observer(log_move_in_menu)
        // Switching contexts
        .add_observer(switch_to_menu)
        .add_observer(switch_to_game)
        // Startup systems
        .add_systems(Startup, (spawn_ui, setup))
        .add_systems(Update, update_ui)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn a 2D camera
    commands.spawn(Camera2d);

    // Spawn a controller entity
    commands.spawn((
        Controller,
        InGameContext,
        ContextActivity::<InGameContext>::ACTIVE,
        InMenuContext,
        ContextActivity::<InMenuContext>::INACTIVE,
        Name::new("Controller"),
        // In game context actions
        actions!(
            InGameContext[
                (
                    Action::<MovePlayer>::new(),
                    DeltaScale,
                    Bindings::spawn((
                        Cardinal::wasd_keys(),
                        Axial::left_stick(),
                    )),
                ),
                (
                    Action::<IntoMenuContext>::new(),
                    ActionSettings {
                            require_reset: true,
                            ..Default::default()
                    },
                    Bindings::spawn((
                        Spawn(Binding::from(KeyCode::Escape)),
                        Spawn(Binding::from(GamepadButton::South)),
                    )),
                ),
            ]
        ),
        // In menu context actions
        actions!(
            InMenuContext[
                (
                    Action::<MoveInMenu>::new(),
                    DeltaScale,
                    Bindings::spawn((
                        Cardinal::wasd_keys(),
                        Axial::left_stick(),
                    )),
                ),
                 (
                    Action::<IntoGameContext>::new(),
                    ActionSettings {
                            require_reset: true,
                            ..Default::default()
                    },
                    Bindings::spawn((
                        Spawn(Binding::from(KeyCode::Escape)),
                        Spawn(Binding::from(GamepadButton::South)),
                    )),
                ),
            ]
        ),
    ));

    // Spawn a square entity
    commands.spawn((
        Square,
        Name::new("Square"),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::splat(100.0)),
            ..default()
        },
    ));
}

fn move_square(
    trigger: Trigger<Fired<MovePlayer>>,
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

fn switch_to_menu(trigger: Trigger<Started<IntoMenuContext>>, mut commands: Commands) {
    info!("Switching to menu context: {:#?}", trigger);
    commands.entity(trigger.target()).insert((
        ContextActivity::<InGameContext>::INACTIVE,
        ContextActivity::<InMenuContext>::ACTIVE,
    ));
}

fn log_move_in_menu(trigger: Trigger<Fired<MoveInMenu>>) {
    info!("Moving in menu: {:#?}", trigger);
}

fn switch_to_game(trigger: Trigger<Started<IntoGameContext>>, mut commands: Commands) {
    info!("Switching to Game context: {:#?}", trigger);
    commands.entity(trigger.target()).insert((
        ContextActivity::<InMenuContext>::INACTIVE,
        ContextActivity::<InGameContext>::ACTIVE,
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
                ActiveContextUiMarker,
                Text::new("Active context: SquareMode or RectangleMode")
            )
        ],
    ));
}

/// Prints active context
fn update_ui(
    mut text_to_change: Query<&mut Text, With<ActiveContextUiMarker>>,
    contexts: Query<
        (
            &ContextActivity<InMenuContext>,
            &ContextActivity<InGameContext>,
        ),
        With<Controller>,
    >,
) {
    if let Ok((menu, game)) = contexts.single() {
        let active_context = if **menu {
            "Menu"
        } else if **game {
            "Game"
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
#[derive(Component)]
struct Controller;

#[derive(Component)]
struct ActiveContextUiMarker;

///// Modes (contexts) for input actions /////
#[derive(Component)]
struct InGameContext;

#[derive(Component)]
struct InMenuContext;

///// Maker components for shapes /////
#[derive(Component)]
struct Square;

///// Actions for the Game context /////
#[derive(InputAction)]
#[action_output(Vec2)]
struct MovePlayer;

#[derive(InputAction)]
#[action_output(Vec2)]
struct IntoMenuContext;

///// Actions for Menu context /////
#[derive(InputAction)]
#[action_output(Vec2)]
struct MoveInMenu;

#[derive(InputAction)]
#[action_output(bool)]
struct IntoGameContext;
