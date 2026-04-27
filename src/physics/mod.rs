//! Physics and orbital math live here.
//!
//! This module will eventually hold:
//! - quantum number validation
//! - hydrogen orbital sampling
//! - probability density helpers
//! - probability-flow animation logic

use bevy::prelude::{Resource, Vec3};
use rand::Rng;

const BOHR_RADIUS_SCALE: f32 = 1.5;

#[derive(Resource, Clone)]
pub struct OrbitalParams {
    pub n: u32,
    pub l: u32,
    pub m: i32,
    pub particle_count: usize,
}

impl Default for OrbitalParams {
    fn default() -> Self {
        Self {
            n: 1,
            l: 0,
            m: 0,
            particle_count: 1_500,
        }
    }
}

impl OrbitalParams {
    pub fn normalize_quantum_numbers(&mut self) {
        self.n = self.n.max(1);
        self.l = self.l.min(self.n.saturating_sub(1));

        let max_m = self.l as i32;
        self.m = self.m.clamp(-max_m, max_m);
    }
}

pub struct OrbitalPoint {
    pub position: Vec3,
}

pub fn generate_cloud(params: &OrbitalParams) -> Vec<OrbitalPoint> {
    match (params.n, params.l, params.m) {
        (1, 0, 0) => generate_1s_cloud(params.particle_count),
        (2, 1, -1) => generate_2p_cloud(params.particle_count, Axis::X),
        (2, 1, 0) => generate_2p_cloud(params.particle_count, Axis::Z),
        (2, 1, 1) => generate_2p_cloud(params.particle_count, Axis::Y),
        _ => generate_1s_cloud(params.particle_count),
    }
}

pub fn rendered_cloud_label(params: &OrbitalParams) -> &'static str {
    match (params.n, params.l, params.m) {
        (1, 0, 0) => "1s",
        // We use a real-orbital basis here so each m value has a distinct visible lobe axis.
        (2, 1, -1) => "2p_x",
        (2, 1, 0) => "2p_z",
        (2, 1, 1) => "2p_y",
        _ => "1s fallback",
    }
}

#[derive(Clone, Copy)]
enum Axis {
    X,
    Y,
    Z,
}

fn generate_1s_cloud(count: usize) -> Vec<OrbitalPoint> {
    let mut rng = rand::rng();
    let mut points = Vec::with_capacity(count);

    for _ in 0..count {
        let position = sample_1s_position(&mut rng);
        points.push(OrbitalPoint { position });
    }

    points
}

fn generate_2p_cloud(count: usize, axis: Axis) -> Vec<OrbitalPoint> {
    let mut rng = rand::rng();
    let mut points = Vec::with_capacity(count);

    for _ in 0..count {
        let radius = sample_2p_radius(&mut rng);
        let direction = sample_2p_direction(&mut rng, axis);
        points.push(OrbitalPoint {
            position: direction * radius,
        });
    }

    points
}

fn sample_1s_position(rng: &mut impl Rng) -> Vec3 {
    // The 1s radial distribution behaves like r^2 * e^(-2r/a0),
    // which we can sample with the sum of three exponential variables.
    let radius = sample_1s_radius(rng);
    sample_unit_direction(rng) * radius
}

fn sample_1s_radius(rng: &mut impl Rng) -> f32 {
    let u1 = rng.random::<f32>().max(f32::EPSILON);
    let u2 = rng.random::<f32>().max(f32::EPSILON);
    let u3 = rng.random::<f32>().max(f32::EPSILON);

    -0.5 * BOHR_RADIUS_SCALE * (u1 * u2 * u3).ln()
}

fn sample_2p_radius(rng: &mut impl Rng) -> f32 {
    let mut product = 1.0_f32;

    // The 2p radial probability behaves like r^4 * e^(-r/a0),
    // which is a gamma-like distribution with five exponential factors.
    for _ in 0..5 {
        product *= rng.random::<f32>().max(f32::EPSILON);
    }

    -BOHR_RADIUS_SCALE * product.ln()
}

fn sample_2p_direction(rng: &mut impl Rng, axis: Axis) -> Vec3 {
    loop {
        let direction = sample_unit_direction(rng);
        let axis_component = match axis {
            Axis::X => direction.x,
            Axis::Y => direction.y,
            Axis::Z => direction.z,
        };

        // Rejection sampling with axis_component^2 gives the familiar two-lobe 2p shape.
        if rng.random::<f32>() <= axis_component * axis_component {
            return direction;
        }
    }
}

fn sample_unit_direction(rng: &mut impl Rng) -> Vec3 {
    let z = rng.random_range(-1.0..=1.0);
    let phi = rng.random_range(0.0..std::f32::consts::TAU);
    let xy = (1.0_f32 - z * z).sqrt();

    Vec3::new(xy * phi.cos(), z, xy * phi.sin())
}
