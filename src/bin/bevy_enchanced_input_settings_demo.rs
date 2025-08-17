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
use std::ops::Deref;

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
        // State management
        .init_state::<GameState>()
        // Startup systems
        .add_systems(Startup, (spawn_debug_ui, setup))
        .add_systems(Update, (update_debug_ui, button_coloring_system))
        .add_systems(OnEnter(GameState::InMenu), spawn_menu)
        .add_systems(OnExit(GameState::InMenu), despawn_menu)
        .run();
}

#[derive(Default, States, Debug, Eq, PartialEq, Hash, Clone, Copy)]
enum GameState {
    #[default]
    InGame,
    InMenu,
}

#[derive(Component)]
struct Menu;

const BUTTON_BACKGROUND_COLOR: Color = Color::srgba(0.2, 0.2, 0.2, 0.8);

fn spawn_menu(mut commands: Commands) {
    info!("Spawning menu UI");
    // Spawn a menu UI
    commands
        .spawn((
            Menu,
            Name::new("Menu"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Srgba::new(0.0, 0.0, 0.0, 0.7).into()),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Name::new("Back to Game Button"),
                    Node {
                        border: UiRect::all(Val::Px(2.0)),
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    Button,
                    BackgroundColor(BUTTON_BACKGROUND_COLOR),
                    BorderRadius::all(Val::Px(2.0)),
                    BorderColor(Srgba::new(1.0, 0.0, 0.0, 0.5).into()),
                    Text::new("Back to Game"),
                ))
                .observe(
                    |mut trigger: Trigger<Pointer<Click>>,
                     mut commands: Commands,
                     controller: Query<Entity, With<Controller>>| {
                        info!(
                            "'Back to Game' button was clicked (entity: {:?})",
                            trigger.target()
                        );
                        // Stop the event from bubbling up the entity hierarchy
                        trigger.propagate(false);

                        commands.trigger_targets(
                            Started::<IntoGameContext> {
                                value: true,
                                state: ActionState::Fired,
                            },
                            controller.single().unwrap(),
                        );
                    },
                );
        });
}

fn button_coloring_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                color.0 = Srgba::new(0.8, 0.8, 0.8, 0.8).into();
            }
            Interaction::Hovered => {
                color.0 = Srgba::new(0.5, 0.5, 0.5, 0.8).into();
            }
            Interaction::None => {
                color.0 = BUTTON_BACKGROUND_COLOR;
            }
        }
    }
}

fn despawn_menu(mut commands: Commands, menu_query: Query<Entity, With<Menu>>) {
    info!("Despawning menu UI");
    // Despawn the menu UI
    if let Ok(menu_entity) = menu_query.single() {
        commands.entity(menu_entity).despawn();
    } else {
        warn!("No menu entity found to despawn.");
    }
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

fn switch_to_menu(
    trigger: Trigger<Started<IntoMenuContext>>,
    mut commands: Commands,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    info!("Switching to menu context: {:#?}", trigger);
    commands.entity(trigger.target()).insert((
        ContextActivity::<InGameContext>::INACTIVE,
        ContextActivity::<InMenuContext>::ACTIVE,
    ));
    if state.deref() == &GameState::InGame {
        next_state.set(GameState::InMenu);
        info!("Changed state to InMenu");
    } else {
        warn!("Tried to switch to Menu context, but current state is not InGame");
    }
}

fn log_move_in_menu(trigger: Trigger<Fired<MoveInMenu>>) {
    info!("Moving in menu: {:#?}", trigger);
}

// Change state to InGame when IntoGameContext action is triggered
fn switch_to_game(
    trigger: Trigger<Started<IntoGameContext>>,
    mut commands: Commands,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    info!("Switching to Game context: {:#?}", trigger);
    commands.entity(trigger.target()).insert((
        ContextActivity::<InMenuContext>::INACTIVE,
        ContextActivity::<InGameContext>::ACTIVE,
    ));

    if state.deref() == &GameState::InMenu {
        next_state.set(GameState::InGame);
        info!("Changed state to InGame");
    } else {
        warn!("Tried to switch to Game context, but current state is not InMenu");
    }
}

fn spawn_debug_ui(mut commands: Commands) {
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
fn update_debug_ui(
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
