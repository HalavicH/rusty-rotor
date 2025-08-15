use avian3d::PhysicsPlugins;
use avian3d::prelude::PhysicsDebugPlugin;
use bevy::prelude::*;
use bevy_drone_sim::avian_falling_cubes_plugin::FallingCubesPlugin;
use bevy_drone_sim::free_camera_plugin::FreeCameraPlugin;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((EguiPlugin::default(), WorldInspectorPlugin::new()))
        // Avian’s physics group + Draw colliders, contacts, etc.
        .add_plugins((PhysicsPlugins::default(), PhysicsDebugPlugin::default()))
        .add_plugins((FreeCameraPlugin, FallingCubesPlugin))
        .run();
}
