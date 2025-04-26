use bevy::prelude::*;

fn main() {
    info!("Starting...");

    let exit = App::new()
        // Bevy Plugins
        .add_plugins(DefaultPlugins)
        .run();
    info!("App exited with: {:?}", exit);
}
