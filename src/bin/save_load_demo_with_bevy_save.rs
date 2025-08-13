use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_save::format::JSONFormat;
use bevy_save::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // bevy_save plugins
        .add_plugins(SavePlugins)
        // Flows
        .add_flows(CaptureFlow, save_cube)
        .add_flows(ApplyFlow, load_cube)
        // bevy inspector plugins
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        // resources
        .init_resource::<AppTypeRegistry>()
        // systems
        .add_systems(Startup, setup)
        .add_systems(Update, (rotate_cube, move_cube))
        .add_systems(Update, handle_save_input)
        // types
        .register_type::<Cube>()
        .run();
}

///// Save and load functionality /////
/// Marks any entity to participate in save/load operations.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CanSaveLoad;

// Pathways look very similar to pipelines, but there are a few key differences
pub struct RONPathway;

impl Pathway for RONPathway {
    // The capture type allows you to save anything you want to disk, even without using reflection
    type Capture = Snapshot;

    type Backend = DefaultDebugBackend;
    type Format = JSONFormat;
    type Key<'a> = &'a str;

    fn key(&self) -> Self::Key<'_> {
        "saves/save_load_demo_with_bevy_save"
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

fn save_cube(In(cap): In<Builder>, world: &World) -> Builder {
    cap.scope(world, |b| {
        b
            .allow::<CanSaveLoad>()
            .allow::<Name>()
            .allow::<Transform>()
            .allow::<Cube>()
            .allow::<DirectionalLight>()
            .allow::<Camera3d>()
            .extract_entities_matching(|e| {
                // Only entities with the `Persistent` component will be captured
                e.contains::<CanSaveLoad>()
            })
            .clear_empty()
    })
}

fn load_cube(In(apply): In<Applier<'static>>, world: &mut World) -> Applier<'static> {
    apply.scope(world, |a| {
        // Apply is handled automatically for us
        // a.apply().expect("Failed to apply")

        a.despawn::<With<CanSaveLoad>>()
    })
}

fn handle_save_input(world: &mut World) {
    let keys = world.resource::<ButtonInput<KeyCode>>();

    if keys.just_released(KeyCode::Enter) {
        info!("Saving data");
        world.save(&RONPathway).expect("Failed to save");
    } else if keys.just_released(KeyCode::Backspace) {
        info!("Loading data");
        world.load(&RONPathway).expect("Failed to load");
    }
}
///// Game components and systems /////
#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(CanSaveLoad)]
struct Cube;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    trace!("Setting up the scene...");
    // Spawn a cube to rotate.
    commands.spawn((
        Cube,
        Name::new("Cube"),
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_translation(Vec3::ZERO),
    ));

    // Spawn a camera looking at the entities to show what's happening in this example.
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.0, 2.0, -5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Add a light source so we can see clearly.
    commands.spawn((
        DirectionalLight {
            color: Color::Srgba(Srgba::new(1.0, 0.0, 0.0, 1.0)),
            illuminance: 200.0,
            shadows_enabled: false,
            affects_lightmapped_mesh_diffuse: false,
            shadow_depth_bias: 0.0,
            shadow_normal_bias: 0.0,
        },
        Transform::from_xyz(3.0, 3.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn rotate_cube(
    mut cube: Query<&mut Transform, With<Cube>>,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
) {
    let Ok(mut transform) = cube.single_mut() else {
        debug!("No cube entity found.");
        return;
    };

    let speed = 2.0;
    if input.pressed(KeyCode::KeyX) {
        // Rotate the cube around the X-axis
        transform.rotate(Quat::from_rotation_x(time.delta_secs() * speed));
    }
    if input.pressed(KeyCode::KeyY) {
        // Rotate the cube around the Y-axis
        transform.rotate(Quat::from_rotation_y(time.delta_secs() * speed));
    }
    if input.pressed(KeyCode::KeyZ) {
        // Rotate the cube around the Z-axis
        transform.rotate(Quat::from_rotation_z(time.delta_secs() * speed));
    }
}

fn move_cube(
    mut cube: Query<&mut Transform, With<Cube>>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let Ok(mut transform) = cube.single_mut() else {
        debug!("No cube entity found.");
        return;
    };

    let speed = 2.0; // Speed of movement

    if input.pressed(KeyCode::KeyW) {
        // Move forward
        transform.translation += Vec3::Z * speed * time.delta_secs();
    }
    if input.pressed(KeyCode::KeyS) {
        // Move backward
        transform.translation -= Vec3::Z * speed * time.delta_secs();
    }
    if input.pressed(KeyCode::KeyA) {
        // Move left
        transform.translation -= Vec3::X * speed * time.delta_secs();
    }
    if input.pressed(KeyCode::KeyD) {
        // Move right
        transform.translation += Vec3::X * speed * time.delta_secs();
    }
}
