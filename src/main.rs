// NOTE: ECS
// System - The actual code
// Entities - The "objects" (ID)
// Components - The data

use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;

// Window Size
pub const WIDTH: f32 = 1280.0;
pub const HEIGHT: f32 = 720.0;

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Tower {
    shooting_timer: Timer,
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Lifetime {
    timer: Timer,
}

pub struct GameAssets {
    bullet_scene: Handle<Scene>,
}

fn main() {
    App::new()
        // NOTE: Set the background of our app
        .insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
        // NOTE: Describe the information needed for creating a window
        .insert_resource(WindowDescriptor {
            width: WIDTH,
            height: HEIGHT,
            title: "Bevy Tower Defense".to_string(),
            resizable: false,
            // NOTE: Two dots here means the rest of the function arguments
            // equivalent to just default()
            ..Default::default()
        })
        // Our systems
        .add_startup_system(spawn_basic_scene)
        .add_startup_system(spawn_camera)
        .add_startup_system(asset_loading)
        .add_system(tower_shooting)
        .add_system(bullet_despawn)
        /*
        Plugins
        */
        .add_plugins(DefaultPlugins)
        // Inspector plugins
        .add_plugin(WorldInspectorPlugin::new())
        .register_type::<Tower>()
        .run();
}

// NOTE: AssetServer allow you to load the assets asynchronously
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
    mut commands: Commands, // NOTE: commands are actual manipulations to the engine
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

    // Spawn a cube
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.67, 0.84, 0.92).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        })
        .insert(Tower {
            shooting_timer: Timer::from_seconds(1.0, true),
        })
        .insert(Name::new("Tower"));

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
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
    // NOTE: FINDs the <Tower>s in the scene and iterate them
    // Can use tuple as parameter to iterate to get a tuple of components, which is the entity associated with these components(Only query entities that have all these components)
    // Remember the entity is just a ID
    mut towers: Query<&mut Tower>,
    bullet_assets: Res<GameAssets>,
) {
    for mut tower in &mut towers {
        tower.shooting_timer.tick(time.delta());
        if tower.shooting_timer.just_finished() {
            let spawn_transform =
                Transform::from_xyz(0.0, 0.7, 0.6).with_rotation(Quat::from_rotation_y(-PI / 2.0));

            commands
                .spawn_bundle(SceneBundle {
                    scene: bullet_assets.bullet_scene.clone(),
                    transform: spawn_transform,
                    ..default()
                })
                .insert(Lifetime {
                    timer: Timer::from_seconds(0.5, false),
                })
                .insert(Name::new("Bullet"));
        }
    }
}

fn bullet_despawn(
    mut commands: Commands,
    mut bullets: Query<(Entity, &mut Lifetime)>,
    time: Res<Time>,
) {
    for (entity, mut lifetime) in &mut bullets {
        lifetime.timer.tick(time.delta());
        if lifetime.timer.just_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}


