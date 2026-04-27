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

pub struct OrbitalPoint {
    pub position: Vec3,
}

pub fn generate_cloud(params: &OrbitalParams) -> Vec<OrbitalPoint> {
    // We only support 1s right now, so unsupported values fall back to 1s
    // while still letting the app adopt the future parameter shape now.
    if params.n != 1 || params.l != 0 || params.m != 0 {
        return generate_1s_cloud(params.particle_count);
    }

    generate_1s_cloud(params.particle_count)
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

fn sample_1s_position(rng: &mut impl Rng) -> Vec3 {
    // The 1s radial distribution behaves like r^2 * e^(-2r/a0),
    // which we can sample with the sum of three exponential variables.
    let radius = sample_1s_radius(rng);
    let z = rng.random_range(-1.0..=1.0);
    let phi = rng.random_range(0.0..std::f32::consts::TAU);
    let xy = (1.0_f32 - z * z).sqrt();

    Vec3::new(radius * xy * phi.cos(), radius * z, radius * xy * phi.sin())
}

fn sample_1s_radius(rng: &mut impl Rng) -> f32 {
    let u1 = rng.random::<f32>().max(f32::EPSILON);
    let u2 = rng.random::<f32>().max(f32::EPSILON);
    let u3 = rng.random::<f32>().max(f32::EPSILON);

    -0.5 * BOHR_RADIUS_SCALE * (u1 * u2 * u3).ln()
}
