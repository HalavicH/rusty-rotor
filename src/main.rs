use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    info!("Starting...");

    let exit = App::new()
        // Bevy Plugins
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(Startup, |mut commands: Commands| {
            // Spawn a 2D camera
            commands.spawn(Camera2d);
        })
        .add_systems(Update, list_gamepads)
        .run();
    info!("App exited with: {:?}", exit);
}

fn list_gamepads(gamepad: Query<(&Gamepad, &GamepadSettings, &Name)>) {
    let Ok(gp) = gamepad.single() else {
        println!("No gamepads connected.");
        return;
    };
    let (g, gs, name) = gp;
    println!("Currently connected gamepad: {name}");
    println!("Gamepad: {g:#?}, Settings: {gs:#?}");
}
