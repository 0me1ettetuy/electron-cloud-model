//! Rendering code lives here.
//!
//! This module will eventually hold:
//! - 3D scene setup
//! - particle cloud drawing
//! - camera controls
//! - UI overlays

use bevy::{prelude::*, window::WindowResolution};

use crate::physics;

pub struct RenderPlugin;

#[derive(Resource)]
pub struct CloudNeedsRegeneration(pub bool);

#[derive(Component)]
struct ElectronParticle;

#[derive(Component)]
struct HudText;

#[derive(Resource)]
struct ParticleRenderAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CloudNeedsRegeneration(true))
            .add_systems(Startup, (setup_scene, setup_hud))
            .add_systems(Update, (regenerate_cloud, update_hud));
    }
}

pub fn window_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: "Electron Cloud Model".into(),
            resolution: WindowResolution::new(1280, 720),
            ..default()
        }),
        ..default()
    }
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

fn setup_hud(mut commands: Commands, params: Res<physics::OrbitalParams>) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        IsDefaultUiCamera,
    ));

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(16.0),
                left: px(16.0),
                padding: UiRect::axes(px(14.0), px(10.0)),
                max_width: px(320.0),
                border_radius: BorderRadius::all(px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.05, 0.1, 0.82)),
        ))
        .with_child((
            Text::new(build_hud_text(&params)),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.95, 1.0)),
            HudText,
        ));
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

fn update_hud(params: Res<physics::OrbitalParams>, mut hud_text: Single<&mut Text, With<HudText>>) {
    if !params.is_changed() {
        return;
    }

    hud_text.0 = build_hud_text(&params);
}

fn build_hud_text(params: &physics::OrbitalParams) -> String {
    format!(
        "Electron Cloud Model\nn: {}   l: {}   m: {}\nparticles: {}\n\nControls\nR: regenerate cloud\nUp: add 250 particles\nDown: remove 250 particles",
        params.n, params.l, params.m, params.particle_count
    )
}
