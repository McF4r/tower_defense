/*
NOTE: ECS
System - The actual code      =>   The Actual Behaviours
Entities - The "objects" (ID) =>   Spawned
Components - The data         =>   Inserted
*/

/*
NOTE: AssetServer 
Allows you to load the assets asynchronously
*/

/*
NOTE:
Commands - Are actual manipulations to the engine
*/

/*
NOTE: Query
Queries enable iteration over entities and their components as well as filtering them on certain conditions.
A query matches its parameters against the world to produce a series of results.
Each *query result* is a tuple of components (the same components defined in the query) that belong to the same entity.
Remember the entity is just a ID, so Entity in the Query parameter doesn't need an ampersand
WARNING:
Example: bullets: Query<(Entity, &GlobalTransform), With<Bullet>> ---> Get the global coordinates of spawned Entity which CONTAIN this component
*/

use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;

use bevy::utils::FloatOrd;

// Window Size
pub const WIDTH: f32 = 1280.0;
pub const HEIGHT: f32 = 720.0;
#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Tower {
    shooting_timer: Timer,
    bullet_offset: Vec3,
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Lifetime {
    timer: Timer,
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Target {
    speed: f32,
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Health{
    value: i32,
}

pub struct GameAssets {
    bullet_scene: Handle<Scene>,
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Bullet {
    direction: Vec3,
    speed: f32,
}

fn main() {
    App::new()
        // Set the background of our app
        .insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
        // Describe the information needed for creating a window
        .insert_resource(WindowDescriptor {
            width: WIDTH,
            height: HEIGHT,
            title: "Bevy Tower Defense".to_string(),
            resizable: false,
            // Two dots here means the rest of the function arguments
            // equivalent to just default()
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        // Inspector plugins
        .add_plugin(WorldInspectorPlugin::new())
        .register_type::<Tower>()
        .register_type::<Lifetime>()
        .register_type::<Target>()
        .register_type::<Bullet>()
        .register_type::<Health>()
        // Our systems
        .add_startup_system(spawn_basic_scene)
        .add_startup_system(spawn_camera)
        .add_startup_system(asset_loading)
        .add_system(tower_shooting)
        .add_system(move_bullets)
        .add_system(move_targets)
        .add_system(bullet_despawn)
        .add_system(target_death)
        .add_system(bullet_collison)
        .run();
}

fn asset_loading(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(GameAssets {
        bullet_scene: assets.load("Bullet.glb#Scene0"),
    });
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

// NOTE: PBR => Phycically basd rendering
// Check the official bevy 3D example for shading
fn spawn_basic_scene(
    mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn a plane
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..default()
        })
        // Name insert here, same below
        .insert(Name::new("Ground"));

    // Spawn a Tower cube
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.67, 0.84, 0.92).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        })
        .insert(Tower {
            shooting_timer: Timer::from_seconds(1.0, true),
            bullet_offset: Vec3::new(0.0, 0.2, 0.5),
        })
        .insert(Name::new("Tower"));

    // Spawn two target in the scene
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.4 })),
            material: materials.add(Color::rgb(0.67, 0.84, 0.92).into()),
            transform: Transform::from_xyz(-2.0, 0.2, 1.5),
            ..default()
        })
        .insert(Target { speed: 0.3 })
        .insert(Health { value: 3 })
        .insert(Name::new("Target"));
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.4 })),
            material: materials.add(Color::rgb(0.67, 0.84, 0.92).into()),
            transform: Transform::from_xyz(-4.0, 0.2, 1.5),
            ..default()
        })
        .insert(Target { speed: 0.3 })
        .insert(Health { value: 3 })
        .insert(Name::new("Target"));

    // Spawn a directional light
    commands
        .spawn_bundle(PointLightBundle {
            point_light: PointLight {
                intensity: 1500.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(4.0, 8.0, 4.0),
            ..default()
        })
        .insert(Name::new("Light"));
}

fn tower_shooting(
    mut commands: Commands,
    time: Res<Time>,

    mut towers: Query<(Entity, &mut Tower, &GlobalTransform)>,
    // Get the entity with Target Component
    targets: Query<&GlobalTransform, With<Target>>,
    // A container to hold GameAssets that load bullet model which is "bullet_scene"
    bullet_assets: Res<GameAssets>,
) {
    for (tower_ent, mut tower, transform) in &mut towers {
        // Start time ticking
        tower.shooting_timer.tick(time.delta());
        if tower.shooting_timer.just_finished() {
            let bullet_spawn = transform.translation() + tower.bullet_offset;

            let direction = targets
                .iter()
                .min_by_key(|target_transform| {
                    // NOTE: Floats in Rust aren't orderable by default
                    // FloatOrd = Float Orderable
                    FloatOrd(Vec3::distance(target_transform.translation(), bullet_spawn))
                })
                .map(|closest_target| closest_target.translation() - bullet_spawn);

            // Incase there is no target left(Maybe killed by something)
            if let Some(direction) = direction {

                commands.entity(tower_ent).with_children(|commands| {
                    commands
                        .spawn_bundle(SceneBundle {
                            scene: bullet_assets.bullet_scene.clone(),
                            transform: Transform::from_translation(tower.bullet_offset),
                            ..default()
                        })
                        .insert(Lifetime {
                            timer: Timer::from_seconds(1000.5, false),
                        })
                        .insert(Bullet {
                            direction,
                            speed: 2.5,
                        })
                        .insert(Name::new("Bullet"));

                });
            }
        }
    }
}

fn bullet_despawn(
    mut commands: Commands,
    mut bullets: Query<(Entity, &mut Lifetime)>,
    time: Res<Time>,
) {
    // For every bullet entity, when the time comes, it vanishes
    for (entity, mut lifetime) in &mut bullets {
        lifetime.timer.tick(time.delta());
        if lifetime.timer.just_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn move_targets(mut targets: Query<(&Target, &mut Transform)>, time: Res<Time>) {
    for (target, mut transform) in &mut targets {
        transform.translation.x += target.speed * time.delta_seconds();
    }
}

fn move_bullets(mut bullets: Query<(&Bullet, &mut Transform)>, time: Res<Time>) {
    for (bullet, mut transform) in &mut bullets {
        transform.translation += bullet.direction.normalize() * bullet.speed * time.delta_seconds();
    }
}

fn target_death(mut commands: Commands, targets: Query<(Entity, &Health)>) {
    for (ent, health) in &targets {
        if health.value <= 0 {
            commands.entity(ent).despawn_recursive(); // Also despawn its children
        }
    }
}

fn bullet_collison(
    mut commands: Commands,
    bullets: Query<(Entity, &GlobalTransform), With<Bullet>>,
    mut targets: Query<(&mut Health, &Transform), With<Target>>,
) {
    for (bullet, bullet_transform) in &bullets {
        for (mut health, target_transform) in &mut targets {
            if Vec3::distance(bullet_transform.translation(), target_transform.translation) < 0.2 {
                commands.entity(bullet).despawn_recursive();
                health.value -= 1;
                break; // NOTE: Only does damage once
            }
        }
    }
}
