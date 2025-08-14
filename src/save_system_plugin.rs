use bevy::prelude::*;
use bevy_save::format::JSONFormat;
use bevy_save::prelude::*;

pub struct SaveSystemPlugin;

impl bevy::prelude::Plugin for SaveSystemPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // bevy_save plugins
            .add_plugins(SavePlugins)
            // Flows
            // .add_flows(CaptureFlow, save_cube)
            // .add_flows(ApplyFlow, load_cube)
            .register_type::<CanSaveLoad>()
            .add_systems(
                bevy::prelude::Update,
                handle_save_input, // .run_if(bevy::prelude::in_state(bevy::prelude::AppState::Running)),
            );
    }
}

///// Save and load functionality /////
/// Marks any entity to participate in save/load operations.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CanSaveLoad;

// Pathways look very similar to pipelines, but there are a few key differences
pub struct BasicSaveLoadPathway;

impl Pathway for BasicSaveLoadPathway {
    // The capture type allows you to save anything you want to disk, even without using reflection
    type Capture = Snapshot;

    type Backend = DefaultDebugBackend;
    type Format = JSONFormat;
    type Key<'a> = String;

    fn key(&self) -> Self::Key<'_> {
        // TODO: Parametrize this key generation
        "saves/save_load_demo_with_bevy_save".to_string()
    }

    // Instead of capturing and applying directly, now these methods just return labels to user-defined flows
    // This allows for better dependency injection and reduces code complexity
    fn capture(&self, _world: &World) -> impl FlowLabel {
        CaptureFlow
    }

    fn apply(&self, _world: &World) -> impl FlowLabel {
        ApplyFlow
    }
}

// Flow labels don't encode any behavior by themselves, only point to flows
#[derive(Hash, Debug, PartialEq, Eq, Clone, Copy, FlowLabel)]
pub struct CaptureFlow;

#[derive(Hash, Debug, PartialEq, Eq, Clone, Copy, FlowLabel)]
pub struct ApplyFlow;

fn handle_save_input(world: &mut World) {
    let keys = world.resource::<ButtonInput<KeyCode>>();

    if keys.just_released(KeyCode::Enter) {
        info!("Saving data");
        world.save(&BasicSaveLoadPathway).expect("Failed to save");
    } else if keys.just_released(KeyCode::Backspace) {
        info!("Loading data");
        world.load(&BasicSaveLoadPathway).expect("Failed to load");
    }
}
