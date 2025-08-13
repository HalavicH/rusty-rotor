use bevy::prelude::*;
use bevy_drone_sim::free_camera_plugin::FreeCameraPlugin;
use bevy_drone_sim::rotating_cube_plugin::RotatingCubePlugin;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((EguiPlugin::default(), WorldInspectorPlugin::new()))
        .add_plugins((RotatingCubePlugin, FreeCameraPlugin))
        .run();
}
