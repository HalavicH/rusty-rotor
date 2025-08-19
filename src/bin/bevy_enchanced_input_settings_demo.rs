//! This demo evaluates the `bevy_enhanced_input` crate.
//!
//! # Demo Description
//! This demo showcases menu and settings management for keyboard and joystick inputs using the `bevy_enhanced_input` crate.

use bevy::ecs::relationship::RelatedSpawnerCommands;
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
        // Register types
        .register_type::<KeyboardInputSettings>()
        // Debug plugins
        .add_plugins((EguiPlugin::default(), WorldInspectorPlugin::new()))
        // Camera plugin for free flight
        .add_plugins(FreeCameraPlugin)
        // Bevy enhanced input plugin
        .init_resource::<KeyboardInputSettings>()
        .init_resource::<JoyStickInputSettings>()
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
        .add_systems(
            Update,
            (
                update_debug_ui,
                menu_button_coloring_system,
                focus_on_click,
                listen_for_key,
            ),
        )
        .add_systems(OnEnter(GameState::InMenu), spawn_menu)
        .add_systems(OnExit(GameState::InMenu), despawn_menu)
        .run();
}

///// UI and Menu Management /////
#[derive(Default, States, Debug, Eq, PartialEq, Hash, Clone, Copy)]
enum GameState {
    #[default]
    InGame,
    InMenu,
}

const BUTTON_BACKGROUND_COLOR: Color = Color::srgba(0.2, 0.2, 0.2, 0.8);
const REBIND_BACKGROUND_COLOR: Color = Color::srgba(0.3, 0.3, 0.3, 0.8);

#[derive(Component)]
#[require(Button)]
struct MenuButton;
#[derive(Component)]
struct MenuRootNode;
fn spawn_menu(
    mut commands: Commands,
    keyboard_bindings: Res<KeyboardInputSettings>,
    joystick_bindings: Res<JoyStickInputSettings>,
) {
    info!("Spawning menu UI");
    // Spawn a menu UI
    commands
        .spawn((
            MenuRootNode,
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
            // Back to Game button
            parent
                .spawn((
                    Name::new("Back to Game Button"),
                    Node {
                        border: UiRect::all(Val::Px(2.0)),
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    MenuButton,
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

            // Display input mapping
            parent
                .spawn((
                    Name::new("Keyboard Input Mapping"),
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(5.0),
                        ..default()
                    },
                ))
                .with_children(|parent: &mut RelatedSpawnerCommands<'_, ChildOf>| {
                    spawn_keyboard_settings(keyboard_bindings, parent);
                });

            parent
                .spawn((
                    Name::new("Joystick Input Mapping"),
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(5.0),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    spawn_joystick_settings(joystick_bindings, parent);
                });

            // Exit button
            parent
                .spawn((
                    Name::new("Exit Button"),
                    Node {
                        border: UiRect::all(Val::Px(2.0)),
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    MenuButton,
                    BackgroundColor(BUTTON_BACKGROUND_COLOR),
                    BorderRadius::all(Val::Px(2.0)),
                    BorderColor(Srgba::new(1.0, 0.0, 0.0, 0.5).into()),
                    Text::new("Exit Game"),
                ))
                .observe(|mut trigger: Trigger<Pointer<Click>>| {
                    info!("'Exit Button' was clicked (entity: {:?})", trigger.target());
                    // Stop the event from bubbling up the entity hierarchy
                    trigger.propagate(false);

                    // Exit the application
                    std::process::exit(0);
                });
        });
}

fn spawn_joystick_settings(
    joystick_bindings: Res<JoyStickInputSettings>,
    parent: &mut RelatedSpawnerCommands<ChildOf>,
) {
    // Hardcoded joystick settings
    // TODO: Generalize
    parent
        .spawn((
            Name::new("Forward/Backward Axis Binding Row"),
            Node::DEFAULT,
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("Forward/Backward Axis Label"),
                Text::new("Forward/Backward Axis:"),
                Node::DEFAULT,
            ));
            parent.spawn((
                RebindBox,
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(30.0),
                    border: UiRect::all(Val::Px(1.0)),
                    padding: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                Button,
                BackgroundColor(REBIND_BACKGROUND_COLOR),
                // Text::new(format!("{:?}", joystick_bindings.movement.y))
            ));
            parent
                .spawn((Name::new("Invert Axis Row"), Node::DEFAULT))
                .with_children(|parent| {
                    parent.spawn((
                        Name::new("Invert Axis Label"),
                        Text::new("Invert Axis:"),
                        Node::DEFAULT,
                    ));
                    let _ec = spawn_checkbox(parent);
                    // Do additional setup for the checkbox
                });
        });
}

fn spawn_checkbox<'a>(parent: &'a mut RelatedSpawnerCommands<ChildOf>) -> EntityCommands<'a> {
    let is_checked = false;
    let symbol = if is_checked { "x" } else { "" };
    let mut entity_commands = parent.spawn((
        Name::new("Checkbox"),
        Checkbox { is_checked },
        Node {
            width: Val::Px(20.0),
            height: Val::Px(20.0),
            border: UiRect::all(Val::Px(1.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Button,
        BackgroundColor(REBIND_BACKGROUND_COLOR),
        children![(Node::DEFAULT, Text::new(symbol))],
    ));

    entity_commands.observe(
        |mut trigger: Trigger<Pointer<Click>>,
         mut checkbox: Query<&mut Checkbox>,
         children: Query<&Children>,
         mut commands: Commands| {
            info!("Checkbox clicked: {:?}", trigger.target());
            trigger.propagate(false);

            let mut checkbox = checkbox
                .single_mut()
                .expect("Expected single Checkbox entity");
            checkbox.is_checked = !checkbox.is_checked;
            let symbol = if checkbox.is_checked { "x" } else { "" };

            let children = children
                .get(trigger.target())
                .expect("Expected entity to have children");

            let text_entity = children
                .first()
                .expect("Expected entity to have text child");

            // Update the text of the checkbox
            commands
                .entity(*text_entity)
                .insert(Text::new(symbol.to_string()));
        },
    );
    entity_commands
}

#[derive(Component)]
struct Checkbox {
    is_checked: bool,
}

/// Helper type to describe a single binding row
struct BindingRow<'a> {
    label: &'a str,
    getter: fn(&KeyboardInputSettings) -> KeyCode,
    setter: fn(&mut KeyboardInputSettings, KeyCode),
}

fn spawn_binding_row<A: InputAction + InsertBindings>(
    parent: &mut RelatedSpawnerCommands<'_, ChildOf>,
    row: BindingRow,
    keyboard_bindings: &KeyboardInputSettings,
) {
    parent
        .spawn((Name::new(format!("{} Row", row.label)), Node::DEFAULT))
        .with_children(|parent| {
            parent.spawn((
                Name::new(format!("{} Label", row.label)),
                Text::new(format!("{}:", row.label)),
                Node::DEFAULT,
            ));
            parent
                .spawn((
                    RebindBoxMeta {
                        label: row.label.to_string(),
                        setter: row.setter,
                    },
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(30.0),
                        border: UiRect::all(Val::Px(1.0)),
                        padding: UiRect::all(Val::Px(5.0)),
                        ..default()
                    },
                    Button,
                    BackgroundColor(REBIND_BACKGROUND_COLOR),
                    Text::new(format!("{:?}", (row.getter)(keyboard_bindings))),
                ))
                .observe(handle_binding_change::<A>);
        });
}

fn spawn_keyboard_settings(
    keyboard_bindings: Res<KeyboardInputSettings>,
    parent: &mut RelatedSpawnerCommands<ChildOf>,
) {
    let rows = [
        BindingRow {
            label: "Move Forward",
            getter: |k| k.movement.north,
            setter: |k, key| k.movement.north = key,
        },
        BindingRow {
            label: "Move Backward",
            getter: |k| k.movement.south,
            setter: |k, key| k.movement.south = key,
        },
        BindingRow {
            label: "Strafe Right",
            getter: |k| k.movement.east,
            setter: |k, key| k.movement.east = key,
        },
        BindingRow {
            label: "Strafe Left",
            getter: |k| k.movement.west,
            setter: |k, key| k.movement.west = key,
        },
    ];

    for row in rows {
        spawn_binding_row::<MovePlayer>(parent, row, &keyboard_bindings);
    }

    // Add a row for the menu action
    let esc = BindingRow {
        label: "To pause menu",
        getter: |k| k.menu,
        setter: |k, key| k.menu = key,
    };
    // Doesn't work, as (CardinalBindings, AxialBindings) is not a valid binding group for IntoMenuContext
    spawn_binding_row::<IntoMenuContext>(parent, esc, &keyboard_bindings);
}

fn handle_binding_change<A: InputAction + InsertBindings>(
    mut trigger: Trigger<NewBinding>,
    mut keyboard_input_settings: ResMut<KeyboardInputSettings>,
    joy_bindings: Res<JoyStickInputSettings>,
    move_action: Query<Entity, With<Action<A>>>,
    meta: Query<&RebindBoxMeta>,
    mut commands: Commands,
) {
    trigger.propagate(false);

    let ui_entity = trigger.target();
    let meta = meta
        .get(ui_entity)
        .expect("Expected RebindBoxMeta component");

    info!("New binding for {} received: {:?}", meta.label, trigger);

    // Update the keyboard input settings with the new binding
    (meta.setter)(&mut keyboard_input_settings, trigger.key);

    // Update the text in the rebind box
    commands
        .entity(ui_entity)
        .insert(Text::new(format!("{:?}", trigger.key)));

    // Update the MovePlayer action bindings (replace)
    if let Ok(entity) = move_action.single() {
        let mut entity_commands: EntityCommands<'_> = commands.entity(entity);
        entity_commands.remove::<Bindings>();
        A::insert_bindings(
            &mut entity_commands,
            &keyboard_input_settings,
            &joy_bindings,
        );
    }
}

#[allow(clippy::type_complexity)]
fn menu_button_coloring_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<MenuButton>),
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

fn despawn_menu(mut commands: Commands, menu_query: Query<Entity, With<MenuRootNode>>) {
    info!("Despawning menu UI");
    // Despawn the menu UI
    if let Ok(menu_entity) = menu_query.single() {
        commands.entity(menu_entity).despawn();
    } else {
        warn!("No menu entity found to despawn.");
    }
}

fn setup(
    mut commands: Commands,
    keyboard_bindings: Res<KeyboardInputSettings>,
    joy_bindings: Res<JoyStickInputSettings>,
    state: Res<State<GameState>>,
) {
    // Spawn a 2D camera
    commands.spawn(Camera2d);

    let is_in_menu = state.deref() == &GameState::InMenu;

    // Spawn a controller entity
    commands.spawn((
        Controller,
        InGameContext,
        ContextActivity::<InGameContext>::new(!is_in_menu),
        InMenuContext,
        ContextActivity::<InMenuContext>::new(is_in_menu),
        Name::new("Controller"),
        // In game context actions
        actions!(
            InGameContext[
                (
                    Action::<MovePlayer>::new(),
                    DeltaScale,
                    Bindings::spawn((
                        CardinalBindings::from(keyboard_bindings.movement),
                        joy_bindings.movement,
                    )),
                ),
                (
                    Action::<IntoMenuContext>::new(),
                    ActionSettings {
                            require_reset: true,
                            ..Default::default()
                    },
                    Bindings::spawn((
                        Spawn(Binding::from(keyboard_bindings.menu)),
                        Spawn(joy_bindings.menu)
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

trait InsertBindings {
    /// Inserts the bindings into the entity commands.
    fn insert_bindings(
        entity_commands: &mut EntityCommands<'_>,
        keyboard_input_settings: &KeyboardInputSettings,
        joy_bindings: &JoyStickInputSettings,
    );
}

impl InsertBindings for MovePlayer {
    fn insert_bindings(
        entity_commands: &mut EntityCommands<'_>,
        keyboard_input_settings: &KeyboardInputSettings,
        joy_bindings: &JoyStickInputSettings,
    ) {
        entity_commands.insert(Bindings::spawn((
            CardinalBindings::from(keyboard_input_settings.movement),
            joy_bindings.movement,
        )));
    }
}

impl InsertBindings for IntoMenuContext {
    fn insert_bindings(
        entity_commands: &mut EntityCommands<'_>,
        keyboard_input_settings: &KeyboardInputSettings,
        joy_bindings: &JoyStickInputSettings,
    ) {
        entity_commands.insert(Bindings::spawn((
            Spawn(Binding::from(keyboard_input_settings.menu)),
            Spawn(joy_bindings.menu),
        )));
    }
}

///// Input bindings for the actions /////
type BindingSetter = fn(&mut KeyboardInputSettings, KeyCode);

#[derive(Component)]
#[require(RebindBox)]
struct RebindBoxMeta {
    label: String,
    setter: BindingSetter,
}
#[derive(Debug, Copy, Clone, Reflect)]
struct CardinalKeys {
    pub north: KeyCode,
    pub east: KeyCode,
    pub south: KeyCode,
    pub west: KeyCode,
}

type CardinalBindings = Cardinal<Binding, Binding, Binding, Binding>;
#[derive(Resource, Debug, Reflect)]
#[reflect(Resource)]
struct KeyboardInputSettings {
    movement: CardinalKeys,
    menu: KeyCode,
}
impl Default for KeyboardInputSettings {
    fn default() -> Self {
        Self {
            movement: CardinalKeys {
                north: KeyCode::KeyW,
                south: KeyCode::KeyS,
                west: KeyCode::KeyA,
                east: KeyCode::KeyD,
            },
            menu: KeyCode::Escape,
        }
    }
}
impl From<CardinalKeys> for CardinalBindings {
    fn from(keys: CardinalKeys) -> Self {
        Cardinal {
            north: Binding::from(keys.north),
            south: Binding::from(keys.south),
            west: Binding::from(keys.west),
            east: Binding::from(keys.east),
        }
    }
}
type AxialBindings = Axial<Binding, Binding>;
#[derive(Resource, Debug)]
struct JoyStickInputSettings {
    movement: AxialBindings,
    menu: Binding,
}

impl Default for JoyStickInputSettings {
    fn default() -> Self {
        Self {
            movement: Axial::left_stick(),
            menu: Binding::from(GamepadButton::South),
        }
    }
}

///// Rebinding logic /////
/// Marker for our input box
#[derive(Component, Default)]
struct RebindBox;

/// Marker for "currently focused" rebind box
#[derive(Component)]
struct Focused;

#[derive(Component, Debug, Event)]
struct NewBinding {
    key: KeyCode,
}

/// When the button is clicked, mark it as Focused
#[allow(clippy::type_complexity)]
fn focus_on_click(
    mut commands: Commands,
    mut q: Query<(Entity, &Interaction), (Changed<Interaction>, With<RebindBox>)>,
    focused: Query<Entity, With<Focused>>,
) {
    for (entity, interaction) in &mut q {
        if interaction == &Interaction::Pressed {
            // Remove focus from any other entity
            for old in &focused {
                commands.entity(old).remove::<Focused>();
            }
            // Add focus to this one
            commands.entity(entity).insert(Focused);
            info!("Rebind box focused. Waiting for key...");
        }
    }
}
/// If a box is focused, listen for any key press
fn listen_for_key(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    focused: Query<Entity, With<Focused>>,
) {
    if focused.is_empty() {
        return;
    }

    let entity = focused
        .single()
        .expect("Expected exactly one focused entity");

    // detect any pressed key
    let keys_pressed = keys.get_just_pressed().collect::<Vec<_>>();
    if keys_pressed.is_empty() {
        // No key pressed, do nothing
        return;
    }
    info!("You pressed: {:?}", keys_pressed);

    let Some(&&key) = keys_pressed.first() else {
        return;
    };
    // Remove the Focused marker from the entity
    commands.entity(entity).remove::<Focused>();
    // Emit a NewBinding event with the pressed key to trigger the rebinding
    commands.trigger_targets(NewBinding { key }, entity);
}
