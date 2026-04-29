//! Rendering code lives here.
//!
//! This module will eventually hold:
//! - 3D scene setup
//! - particle cloud drawing
//! - camera controls
//! - UI overlays

use bevy::{
    asset::RenderAssetUsages,
    camera::visibility::NoFrustumCulling,
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
    prelude::*,
    window::WindowResolution,
};

use crate::physics;

pub struct RenderPlugin;
const FLOW_SIMULATION_HZ: f32 = 60.0;
const MAX_FLOW_STEPS_PER_FRAME: usize = 8;
const REFERENCE_GUIDE_SIZE: f32 = 100.0;
const DEFAULT_CAMERA_TARGET: Vec3 = Vec3::ZERO;
const DEFAULT_CAMERA_POSITION: Vec3 = Vec3::new(50.0, 70.0, 50.0);
const WEB_CAMERA_FOV_DEGREES: f32 = 55.0;
const WEB_CAMERA_FAR: f32 = 1000.0;
const PARTICLE_TEMPLATE_RADIUS: f32 = 0.20;

#[derive(Clone, Default)]
pub struct WindowHostConfig {
    pub canvas_selector: Option<String>,
}

#[derive(Resource)]
pub struct CloudNeedsRegeneration(pub bool);

#[derive(Resource)]
pub struct RenderUiConfig {
    pub show_hud: bool,
}

impl Default for RenderUiConfig {
    fn default() -> Self {
        Self { show_hud: false }
    }
}

#[derive(Resource, Default)]
pub struct PendingCameraReset(pub bool);

#[derive(Resource, Default)]
struct FlowSimulationClock {
    accumulated_seconds: f32,
}

#[derive(Component)]
struct HudText;

#[derive(Resource)]
struct ParticleRenderAssets {
    template: ParticleMeshTemplate,
    cloud_mesh: Handle<Mesh>,
}

#[derive(Resource, Default)]
struct CloudParticleState {
    positions: Vec<Vec3>,
    colors: Vec<[f32; 4]>,
    scales: Vec<f32>,
    particle_scale: f32,
}

#[derive(Clone)]
struct ParticleMeshTemplate {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    indices: Vec<u32>,
}

#[derive(Component)]
struct ElectronCloud;

#[derive(Component)]
struct ReferenceGuides;

impl Default for ParticleMeshTemplate {
    fn default() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            indices: Vec::new(),
        }
    }
}

#[derive(Component)]
struct OrbitCamera {
    target: Vec3,
    radius: f32,
    azimuth: f32,
    elevation: f32,
    orbit_speed: f32,
    zoom_speed: f32,
}

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CloudNeedsRegeneration(true))
            .insert_resource(RenderUiConfig::default())
            .insert_resource(PendingCameraReset::default())
            .insert_resource(FlowSimulationClock::default())
            .add_systems(Startup, (setup_scene, setup_hud))
            .add_systems(
                Update,
                (
                    animate_cloud,
                    regenerate_cloud,
                    reset_orbit_camera,
                    orbit_camera_controls,
                    refresh_cloud_cutaway,
                    update_hud,
                ),
            );
    }
}

pub fn window_plugin(window_host: &WindowHostConfig) -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: "Electron Cloud Model".into(),
            resolution: WindowResolution::new(1280, 720),
            canvas: window_host.canvas_selector.clone(),
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
    commands.insert_resource(ClearColor(Color::BLACK));

    let template = build_particle_mesh_template();
    let default_camera = default_camera_transform();
    let cloud_mesh = meshes.add(build_cloud_mesh(
        &template,
        &CloudParticleState::default(),
        &default_camera,
    ));
    let reference_guides_mesh = meshes.add(build_reference_guides_mesh(REFERENCE_GUIDE_SIZE));
    let cloud_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        // Previous stronger blur/glow version:
        // emissive: LinearRgba::rgb(2.0, 2.0, 2.0),
        // Slightly reduced blur version:
        // emissive: LinearRgba::rgb(1.2, 1.2, 1.2),
        // Crisp version:
        emissive: LinearRgba::BLACK,
        unlit: true,
        // Blur version:
        // alpha_mode: AlphaMode::Blend,
        // Crisp version:
        alpha_mode: AlphaMode::Mask(0.5),
        // Earlier glow test:
        // alpha_mode: AlphaMode::Add,
        ..default()
    });
    let reference_guides_material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.35),
        emissive: LinearRgba::rgb(0.12, 0.12, 0.12),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.insert_resource(ParticleRenderAssets {
        template,
        cloud_mesh: cloud_mesh.clone(),
    });
    commands.insert_resource(CloudParticleState::default());

    commands.spawn((
        ElectronCloud,
        Mesh3d(cloud_mesh),
        MeshMaterial3d(cloud_material),
        Transform::default(),
        NoFrustumCulling,
    ));

    commands.spawn((
        ReferenceGuides,
        Mesh3d(reference_guides_mesh),
        MeshMaterial3d(reference_guides_material),
        Transform::default(),
    ));

    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: WEB_CAMERA_FOV_DEGREES.to_radians(),
            near: 0.1,
            far: WEB_CAMERA_FAR,
            ..default()
        }),
        default_camera_transform(),
        default_orbit_camera(),
    ));
}

fn setup_hud(
    mut commands: Commands,
    ui_config: Res<RenderUiConfig>,
    params: Res<physics::OrbitalParams>,
    display_mode: Res<physics::DisplayMode>,
) {
    if !ui_config.show_hud {
        return;
    }

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
            Text::new(build_hud_text(&params, *display_mode)),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.95, 1.0)),
            HudText,
        ));
}

fn animate_cloud(
    time: Res<Time>,
    params: Res<physics::OrbitalParams>,
    flow_animation: Res<physics::FlowAnimation>,
    needs_regeneration: Res<CloudNeedsRegeneration>,
    mut flow_clock: ResMut<FlowSimulationClock>,
    particle_assets: Res<ParticleRenderAssets>,
    mut cloud_state: ResMut<CloudParticleState>,
    mut meshes: ResMut<Assets<Mesh>>,
    camera: Single<&Transform, With<Camera3d>>,
) {
    if needs_regeneration.0 {
        return;
    }

    let fixed_step_seconds = 1.0 / FLOW_SIMULATION_HZ;
    flow_clock.accumulated_seconds += time.delta_secs().min(0.25);

    let mut steps = 0;
    while flow_clock.accumulated_seconds >= fixed_step_seconds && steps < MAX_FLOW_STEPS_PER_FRAME {
        flow_clock.accumulated_seconds -= fixed_step_seconds;
        steps += 1;

        for position in &mut cloud_state.positions {
            *position = physics::advance_probability_flow(
                *position,
                params.as_ref(),
                flow_animation.flow_step_dt(),
            );
        }
    }

    if steps > 0
        && let Some(mesh) = meshes.get_mut(&particle_assets.cloud_mesh)
    {
        update_cloud_mesh_positions(mesh, &particle_assets.template, &cloud_state, &camera);
    }
}

fn regenerate_cloud(
    params: Res<physics::OrbitalParams>,
    display_mode: Res<physics::DisplayMode>,
    mut needs_regeneration: ResMut<CloudNeedsRegeneration>,
    mut flow_clock: ResMut<FlowSimulationClock>,
    particle_assets: Res<ParticleRenderAssets>,
    mut cloud_state: ResMut<CloudParticleState>,
    mut meshes: ResMut<Assets<Mesh>>,
    camera: Single<&Transform, With<Camera3d>>,
) {
    if !needs_regeneration.0 {
        return;
    }

    let cloud = physics::generate_cloud(params.as_ref(), *display_mode);
    cloud_state.positions.clear();
    cloud_state.colors.clear();
    cloud_state.scales.clear();
    cloud_state.positions.reserve(cloud.len());
    cloud_state.colors.reserve(cloud.len());
    cloud_state.scales.reserve(cloud.len());
    cloud_state.particle_scale = params.n.max(1) as f32 / 3.0;
    let base_particle_scale = cloud_state.particle_scale;
    let mut radii = Vec::with_capacity(cloud.len());
    let mut max_radius = 0.0_f32;

    for point in cloud {
        let radius = point.position.length();
        let srgba = point.color.to_srgba();
        cloud_state.positions.push(point.position);
        cloud_state
            .colors
            .push([srgba.red, srgba.green, srgba.blue, srgba.alpha]);
        radii.push(radius);
        max_radius = max_radius.max(radius);
    }

    for radius in radii {
        cloud_state.scales.push(particle_scale_for_radius(
            radius,
            max_radius,
            base_particle_scale,
        ));
    }

    flow_clock.accumulated_seconds = 0.0;

    if let Some(mesh) = meshes.get_mut(&particle_assets.cloud_mesh) {
        *mesh = build_cloud_mesh(&particle_assets.template, &cloud_state, &camera);
    }

    needs_regeneration.0 = false;
}

fn refresh_cloud_cutaway(
    camera: Option<Single<&Transform, (With<Camera3d>, Changed<Transform>)>>,
    particle_assets: Res<ParticleRenderAssets>,
    cloud_state: Res<CloudParticleState>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Some(camera) = camera else {
        return;
    };

    if let Some(mesh) = meshes.get_mut(&particle_assets.cloud_mesh) {
        update_cloud_mesh_positions(mesh, &particle_assets.template, &cloud_state, &camera);
    }
}

fn reset_orbit_camera(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut pending_camera_reset: ResMut<PendingCameraReset>,
    camera: Option<Single<(&mut Transform, &mut OrbitCamera)>>,
) {
    if keyboard.just_pressed(KeyCode::KeyC) {
        pending_camera_reset.0 = true;
    }

    if !pending_camera_reset.0 {
        return;
    }

    let Some(camera) = camera else {
        return;
    };
    let (mut transform, mut orbit_camera) = camera.into_inner();

    *orbit_camera = default_orbit_camera();
    *transform = default_camera_transform();
    pending_camera_reset.0 = false;
}

fn orbit_camera_controls(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    camera: Single<(&mut Transform, &mut OrbitCamera)>,
) {
    let (mut transform, mut orbit_camera) = camera.into_inner();

    if mouse_buttons.pressed(MouseButton::Left) || mouse_buttons.pressed(MouseButton::Middle) {
        orbit_camera.azimuth += mouse_motion.delta.x * orbit_camera.orbit_speed;
        orbit_camera.elevation -= mouse_motion.delta.y * orbit_camera.orbit_speed;
        orbit_camera.elevation = orbit_camera
            .elevation
            .clamp(0.01, std::f32::consts::PI - 0.01);
    }

    if mouse_scroll.delta.y != 0.0 {
        orbit_camera.radius -= mouse_scroll.delta.y * orbit_camera.zoom_speed;
        orbit_camera.radius = orbit_camera.radius.max(1.0);
    }

    let translation = Vec3::new(
        orbit_camera.radius * orbit_camera.elevation.sin() * orbit_camera.azimuth.cos(),
        orbit_camera.radius * orbit_camera.elevation.cos(),
        orbit_camera.radius * orbit_camera.elevation.sin() * orbit_camera.azimuth.sin(),
    );

    transform.translation = translation;
    transform.look_at(orbit_camera.target, Vec3::Y);
}

fn update_hud(
    ui_config: Res<RenderUiConfig>,
    params: Res<physics::OrbitalParams>,
    display_mode: Res<physics::DisplayMode>,
    hud_text: Option<Single<&mut Text, With<HudText>>>,
) {
    if !ui_config.show_hud {
        return;
    }

    let Some(mut hud_text) = hud_text else {
        return;
    };

    if !params.is_changed() && !display_mode.is_changed() {
        return;
    }

    hud_text.0 = build_hud_text(&params, *display_mode);
}

fn build_hud_text(
    params: &physics::OrbitalParams,
    display_mode: physics::DisplayMode,
) -> String {
    let rendered_cloud = physics::rendered_cloud_label(params, display_mode);

    format!(
        "Electron Cloud Model\nn: {}   l: {}   m: {}\nparticles: {}\nrendered cloud: {}\ndisplay mode: {}\nreference flow dt: {:.2}\nrender path: combined cloud mesh\nvisual style: black background + octant cutaway + 3-plane guides\nsampler: generic hydrogen CDF\n\nDebug Controls\nW / S: change n\nE / D: change l\nR / F: change m\nT / G: change particle count\nQ: regenerate cloud\nV: toggle display mode\nC: reset camera\nMouse drag: orbit camera\nMouse wheel: zoom",
        params.n,
        params.l,
        params.m,
        params.particle_count,
        rendered_cloud,
        display_mode.label(),
        physics::FlowAnimation.flow_step_dt()
    )
}

fn default_orbit_camera() -> OrbitCamera {
    let radius = DEFAULT_CAMERA_POSITION.length();
    let azimuth = DEFAULT_CAMERA_POSITION.z.atan2(DEFAULT_CAMERA_POSITION.x);
    let elevation = (DEFAULT_CAMERA_POSITION.y / radius).clamp(-1.0, 1.0).acos();

    OrbitCamera {
        target: DEFAULT_CAMERA_TARGET,
        radius,
        azimuth,
        elevation,
        orbit_speed: 0.01,
        zoom_speed: 10.0,
    }
}

fn default_camera_transform() -> Transform {
    Transform::from_translation(DEFAULT_CAMERA_POSITION).looking_at(DEFAULT_CAMERA_TARGET, Vec3::Y)
}

fn build_particle_mesh_template() -> ParticleMeshTemplate {
    let template_mesh = Sphere::new(PARTICLE_TEMPLATE_RADIUS)
        .mesh()
        .ico(1)
        .expect("particle template sphere should build");

    let positions = match template_mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(values)) => values.clone(),
        _ => Vec::new(),
    };
    let normals = match template_mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
        Some(VertexAttributeValues::Float32x3(values)) => values.clone(),
        _ => Vec::new(),
    };
    let indices = match template_mesh.indices() {
        Some(Indices::U16(values)) => values.iter().map(|value| *value as u32).collect(),
        Some(Indices::U32(values)) => values.clone(),
        None => Vec::new(),
    };

    ParticleMeshTemplate {
        positions,
        normals,
        indices,
    }
}

fn particle_scale_for_radius(radius: f32, max_radius: f32, base_scale: f32) -> f32 {
    let normalized_radius = if max_radius <= f32::EPSILON {
        0.0
    } else {
        (radius / max_radius).clamp(0.0, 1.0)
    };

    let radius_multiplier = 0.65 + normalized_radius * 1.35;
    base_scale * radius_multiplier
}

fn build_reference_guides_mesh(size: f32) -> Mesh {
    let half = size * 0.5;
    let mut positions = Vec::with_capacity(24);

    append_square_outline(
        &mut positions,
        [
            [-half, -half, 0.0],
            [half, -half, 0.0],
            [half, half, 0.0],
            [-half, half, 0.0],
        ],
    );
    append_square_outline(
        &mut positions,
        [
            [-half, 0.0, -half],
            [half, 0.0, -half],
            [half, 0.0, half],
            [-half, 0.0, half],
        ],
    );
    append_square_outline(
        &mut positions,
        [
            [0.0, -half, -half],
            [0.0, half, -half],
            [0.0, half, half],
            [0.0, -half, half],
        ],
    );

    Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
}

fn append_square_outline(positions: &mut Vec<[f32; 3]>, corners: [[f32; 3]; 4]) {
    positions.push(corners[0]);
    positions.push(corners[1]);
    positions.push(corners[1]);
    positions.push(corners[2]);
    positions.push(corners[2]);
    positions.push(corners[3]);
    positions.push(corners[3]);
    positions.push(corners[0]);
}

fn build_cloud_mesh(
    template: &ParticleMeshTemplate,
    cloud_state: &CloudParticleState,
    camera_transform: &Transform,
) -> Mesh {
    let vertex_count = cloud_state.positions.len() * template.positions.len();
    let mut positions = Vec::with_capacity(vertex_count);
    let mut normals = Vec::with_capacity(vertex_count);
    let mut colors = Vec::with_capacity(vertex_count);
    let mut indices = Vec::with_capacity(cloud_state.positions.len() * template.indices.len());
    let vertices_per_particle = template.positions.len() as u32;

    for (particle_index, position) in cloud_state.positions.iter().enumerate() {
        let particle_scale = cloud_state
            .scales
            .get(particle_index)
            .copied()
            .unwrap_or(cloud_state.particle_scale);
        let mut color = cloud_state
            .colors
            .get(particle_index)
            .copied()
            .unwrap_or([1.0, 1.0, 1.0, 1.0]);
        color[3] = particle_opacity(color, *position, camera_transform);
        let base_vertex = particle_index as u32 * vertices_per_particle;

        for template_position in &template.positions {
            positions.push([
                position.x + template_position[0] * particle_scale,
                position.y + template_position[1] * particle_scale,
                position.z + template_position[2] * particle_scale,
            ]);
            colors.push(color);
        }

        normals.extend_from_slice(&template.normals);
        indices.extend(template.indices.iter().map(|index| base_vertex + index));
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors);

    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn update_cloud_mesh_positions(
    mesh: &mut Mesh,
    template: &ParticleMeshTemplate,
    cloud_state: &CloudParticleState,
    camera_transform: &Transform,
) {
    let vertices_per_particle = template.positions.len();
    let expected_vertex_count = cloud_state.positions.len() * vertices_per_particle;

    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
    else {
        return;
    };

    if positions.len() != expected_vertex_count {
        return;
    }

    for (particle_index, particle_position) in cloud_state.positions.iter().enumerate() {
        let start = particle_index * vertices_per_particle;
        let end = start + vertices_per_particle;
        let particle_scale = cloud_state
            .scales
            .get(particle_index)
            .copied()
            .unwrap_or(cloud_state.particle_scale);

        for (vertex, template_position) in positions[start..end]
            .iter_mut()
            .zip(template.positions.iter())
        {
            *vertex = [
                particle_position.x + template_position[0] * particle_scale,
                particle_position.y + template_position[1] * particle_scale,
                particle_position.z + template_position[2] * particle_scale,
            ];
        }
    }

    let Some(VertexAttributeValues::Float32x4(colors)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
    else {
        return;
    };

    if colors.len() != expected_vertex_count {
        return;
    }

    for (particle_index, particle_position) in cloud_state.positions.iter().enumerate() {
        let start = particle_index * vertices_per_particle;
        let end = start + vertices_per_particle;
        let mut color = cloud_state
            .colors
            .get(particle_index)
            .copied()
            .unwrap_or([1.0, 1.0, 1.0, 1.0]);
        color[3] = particle_opacity(color, *particle_position, camera_transform);

        for vertex_color in &mut colors[start..end] {
            *vertex_color = color;
        }
    }
}

fn particle_opacity(color: [f32; 4], position: Vec3, camera_transform: &Transform) -> f32 {
    let camera_facing = (-camera_transform.forward()).as_vec3();

    if position.dot(camera_facing) > 0.0 {
        return 0.0;
    }

    let max_channel = color[0].max(color[1]).max(color[2]);
    let luminance = 0.2126 * color[0] + 0.7152 * color[1] + 0.0722 * color[2];
    let _brightness = max_channel.max(luminance).clamp(0.0, 1.0);
    // Previous stronger blur version:
    // let contrast_opacity = _brightness.powf(2.6);
    // Slightly reduced blur version:
    // let contrast_opacity = _brightness.powf(0.8);

    // Crisp version:
    1.0
    // Previous stronger blur version:
    // 0.002 + contrast_opacity * 0.998
    // Slightly reduced blur version:
    // 0.001 + contrast_opacity * 0.92
}
