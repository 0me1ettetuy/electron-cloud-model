use bevy::{prelude::*, window::WindowResolution};

use crate::physics;

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
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 80.0,
        affects_lightmapped_meshes: true,
    });

    let cloud = physics::generate_1s_cloud(1_500);
    let particle_mesh = meshes.add(Sphere::new(0.06));
    let particle_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.5, 1.0),
        emissive: LinearRgba::rgb(0.05, 0.15, 0.35),
        ..default()
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

    for point in cloud {
        commands.spawn((
            Mesh3d(particle_mesh.clone()),
            MeshMaterial3d(particle_material.clone()),
            Transform::from_translation(point.position),
        ));
    }

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
