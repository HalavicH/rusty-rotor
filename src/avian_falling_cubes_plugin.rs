use avian3d::prelude::*;
use bevy::prelude::*;

pub struct FallingCubesPlugin;

impl Plugin for FallingCubesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Gravity(Vec3::new(0.0, 0.0, 0.0)))
            .add_systems(Startup, (setup_scene, spawn_cubes))
            // .add_systems(Update, log_collisions)
            .add_systems(Update, (handle_gravity_type, apply_gravity))
            .register_type::<WorldGravity>();
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
struct WorldGravity {
    is_enabled: bool,
    is_reversed: bool,
    gravity_force: Vec3,
    gravity_type: GravityType,
}

impl Default for WorldGravity {
    fn default() -> Self {
        Self {
            is_enabled: false,
            is_reversed: false,
            gravity_type: GravityType::Space,
            gravity_force: GravityType::Space.gravity_force(),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Reflect)]
enum GravityType {
    Space,
    Moon,
    Earth,
}

impl GravityType {
    pub fn gravity_force(self) -> Vec3 {
        match self {
            GravityType::Space => Vec3::ZERO,
            GravityType::Moon => Vec3::NEG_Y * 1.62,
            GravityType::Earth => Vec3::NEG_Y * 9.81,
        }
    }
    pub fn gravity_reverse(self) -> Vec3 {
        match self {
            GravityType::Space => Vec3::ZERO,
            GravityType::Moon => Vec3::Y * 1.62,
            GravityType::Earth => Vec3::Y * 9.81,
        }
    }
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(WorldGravity::default());

    // Ground
    commands.spawn((
        Name::new("Ground"),
        RigidBody::Static,
        Collider::cuboid(20.0, 0.2, 20.0),
        Transform::from_xyz(0.0, -0.1, 0.0),
        Mesh3d(meshes.add(Mesh::from(Cuboid::new(20.0, 0.2, 20.0)))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.5, 0.2))),
        // Avian only sends/observes collision events for entities
        // with this tag:
        CollisionEventsEnabled,
    ));

    // Add a light source so we can see clearly.
    commands.spawn((
        DirectionalLight {
            color: Color::Srgba(Srgba::new(1.0, 1.0, 0.0, 1.0)),
            illuminance: 2000.0,
            shadows_enabled: false,
            affects_lightmapped_mesh_diffuse: false,
            shadow_depth_bias: 0.0,
            shadow_normal_bias: 0.0,
        },
        Transform::from_xyz(3.0, 3.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-10.0, 8.0, 14.0).looking_at(Vec3::Y * 2.0, Dir3::Y),
    ));
}
fn spawn_cubes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for y in 0..5 {
        for x in 0..5 {
            let pos = Vec3::new(x as f32 - 3.0, 3.0 + y as f32 * 1.2, 0.0);
            commands.spawn((
                Name::new(format!("Cube_{x}_{y}")),
                RigidBody::Dynamic,
                Collider::cuboid(1.0, 1.0, 1.0),
                // Avian friction/restitution are on the collider’s material:
                Friction::new(0.8),
                Restitution::new(0.4),
                Transform::from_translation(pos),
                Mesh3d(meshes.add(Mesh::from(Cuboid::new(1.0, 1.0, 1.0)))),
                MeshMaterial3d(materials.add(Color::srgb(0.6, 0.7, 1.0))),
                // enable events for these entities:
                CollisionEventsEnabled,
            ));
        }
    }
}

// Avian’s buffered event type:
#[allow(dead_code)]
fn log_collisions(mut started: EventReader<CollisionStarted>) {
    for ev in started.read() {
        info!("Avian: collision START between {:?} and {:?}", ev.0, ev.1);
    }
}

fn apply_gravity(mut gravity: ResMut<Gravity>, world_gravity: ResMut<WorldGravity>) {
    if !world_gravity.is_enabled {
        gravity.0 = Vec3::ZERO;
    } else if world_gravity.gravity_force != gravity.0 {
        info!(
            "Gravity was changed from {:?} to {:?}",
            gravity.0, world_gravity.gravity_force
        );
        gravity.0 = world_gravity.gravity_force;
    }
}

fn handle_gravity_type(keys: Res<ButtonInput<KeyCode>>, mut world_gravity: ResMut<WorldGravity>) {
    if keys.just_pressed(KeyCode::Space) {
        world_gravity.is_enabled = !world_gravity.is_enabled;
        info!(
            "Gravity is {}",
            if world_gravity.is_enabled {
                "enabled"
            } else {
                "disabled"
            }
        );
        return;
    }

    if keys.just_pressed(KeyCode::Digit1) {
        world_gravity.gravity_type = GravityType::Space;
        info!("Gravity set to Space");
    } else if keys.just_pressed(KeyCode::Digit2) {
        world_gravity.gravity_type = GravityType::Moon;
        info!("Gravity set to Moon");
    } else if keys.just_pressed(KeyCode::Digit3) {
        world_gravity.gravity_type = GravityType::Earth;
        info!("Gravity set to Earth");
    } else if keys.just_pressed(KeyCode::KeyR) {
        world_gravity.is_reversed = !world_gravity.is_reversed;
        info!(
            "Gravity set to {}",
            if world_gravity.is_reversed {
                "reversed"
            } else {
                "normal"
            }
        );
    }

    if world_gravity.is_reversed {
        world_gravity.gravity_force = world_gravity.gravity_type.gravity_reverse();
    } else {
        world_gravity.gravity_force = world_gravity.gravity_type.gravity_force();
    }
}
