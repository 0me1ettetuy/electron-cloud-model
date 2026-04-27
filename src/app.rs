use bevy::{prelude::*, window::WindowResolution};

use crate::physics;

#[derive(Resource)]
struct CloudNeedsRegeneration(bool);

#[derive(Component)]
struct ElectronParticle;

#[derive(Resource)]
struct ParticleRenderAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Electron Cloud Model".into(),
                resolution: WindowResolution::new(1280, 720),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(physics::OrbitalParams::default())
        .insert_resource(CloudNeedsRegeneration(true))
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (handle_cloud_input, regenerate_cloud))
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 80.0,
        affects_lightmapped_meshes: true,
    });

    commands.insert_resource(ParticleRenderAssets {
        mesh: meshes.add(Sphere::new(0.06)),
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.5, 1.0),
            emissive: LinearRgba::rgb(0.05, 0.15, 0.35),
            ..default()
        }),
    });

    let nucleus_mesh = meshes.add(Sphere::new(0.18));
    let nucleus_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.4, 0.3),
        emissive: LinearRgba::rgb(0.4, 0.08, 0.04),
        ..default()
    });

    commands.spawn((
        Mesh3d(nucleus_mesh),
        MeshMaterial3d(nucleus_material),
        Transform::default(),
    ));

    commands.spawn((
        PointLight {
            intensity: 4_000_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(8.0, 12.0, 8.0),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-5.0, 4.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn handle_cloud_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut params: ResMut<physics::OrbitalParams>,
    mut needs_regeneration: ResMut<CloudNeedsRegeneration>,
) {
    let mut changed = false;

    if keyboard.just_pressed(KeyCode::KeyR) {
        changed = true;
    }

    if keyboard.just_pressed(KeyCode::ArrowUp) {
        params.particle_count += 250;
        changed = true;
    }

    if keyboard.just_pressed(KeyCode::ArrowDown) && params.particle_count > 250 {
        params.particle_count -= 250;
        changed = true;
    }

    if changed {
        needs_regeneration.0 = true;
        info!(
            "Regenerating 1s cloud with {} particles",
            params.particle_count
        );
    }
}

fn regenerate_cloud(
    mut commands: Commands,
    params: Res<physics::OrbitalParams>,
    mut needs_regeneration: ResMut<CloudNeedsRegeneration>,
    particles: Query<Entity, With<ElectronParticle>>,
    particle_assets: Res<ParticleRenderAssets>,
) {
    if !needs_regeneration.0 {
        return;
    }

    for entity in &particles {
        commands.entity(entity).despawn();
    }

    let cloud = physics::generate_cloud(params.as_ref());

    for point in cloud {
        commands.spawn((
            ElectronParticle,
            Mesh3d(particle_assets.mesh.clone()),
            MeshMaterial3d(particle_assets.material.clone()),
            Transform::from_translation(point.position),
        ));
    }

    needs_regeneration.0 = false;
}
