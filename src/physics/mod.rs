//! Physics and orbital math live here.
//!
//! This module now follows the same high-level approach as the C++ `Atoms`
//! reference: sample generic hydrogen orbitals from quantum numbers `n`, `l`,
//! and `m` instead of branching on a hardcoded list of named orbitals.

use bevy::prelude::{Color, Resource, Vec3};
use rand::Rng;
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

const BOHR_RADIUS_SCALE: f64 = 1.0;
const RADIAL_CDF_SAMPLES: usize = 4096;
const THETA_CDF_SAMPLES: usize = 2048;
const PROBABILITY_FLOW_REFERENCE_DT: f32 = 0.5;
const PROBABILITY_FLOW_RADIUS_SOFTENING: f64 = 0.75;
const PROBABILITY_FLOW_SIN_THETA_FLOOR: f64 = 0.12;
const PROBABILITY_FLOW_MAX_PHI_STEP_RADIANS: f64 = 0.12;
const COLOR_DISTRIBUTION_LOWER_PERCENTILE: f32 = 0.05;
const COLOR_DISTRIBUTION_UPPER_PERCENTILE: f32 = 0.95;

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
            // Match the C++ realtime demo's starting orbital more closely.
            n: 2,
            l: 1,
            m: 0,
            // The web viewer ships precomputed clouds with 100k particles.
            particle_count: 100_000,
        }
    }
}

#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug)]
pub enum DisplayMode {
    SphericalDensity,
    RealOrbitalBasis,
}

impl Default for DisplayMode {
    fn default() -> Self {
        Self::SphericalDensity
    }
}

impl DisplayMode {
    pub fn toggle(&mut self) {
        *self = match self {
            Self::SphericalDensity => Self::RealOrbitalBasis,
            Self::RealOrbitalBasis => Self::SphericalDensity,
        };
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::SphericalDensity => "spherical density",
            Self::RealOrbitalBasis => "real orbital basis",
        }
    }
}

#[derive(Resource, Clone, Copy, Default)]
pub struct FlowAnimation;

impl FlowAnimation {
    pub fn flow_step_dt(self) -> f32 {
        PROBABILITY_FLOW_REFERENCE_DT
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
    pub color: Color,
    intensity: f32,
}

#[derive(Clone)]
struct CdfSampler {
    cumulative: Vec<f64>,
    step_size: f64,
}

type RadialCache = HashMap<(u32, u32), CdfSampler>;
type ThetaCache = HashMap<(u32, u32), CdfSampler>;

pub fn generate_cloud(params: &OrbitalParams, display_mode: DisplayMode) -> Vec<OrbitalPoint> {
    let mut points = if matches!(display_mode, DisplayMode::RealOrbitalBasis) {
        if let Some(points) = generate_real_basis_cloud(params) {
            points
        } else {
            generate_spherical_density_cloud(params)
        }
    } else {
        generate_spherical_density_cloud(params)
    };

    apply_dynamic_color_mapping(&mut points);
    points
}

pub fn rendered_cloud_label(params: &OrbitalParams, display_mode: DisplayMode) -> String {
    if matches!(display_mode, DisplayMode::RealOrbitalBasis) {
        if let Some(label) = real_orbital_label(params) {
            return label.to_string();
        }
    }

    spherical_density_label(params)
}

pub fn advance_probability_flow(
    position: Vec3,
    params: &OrbitalParams,
    flow_step_dt: f32,
) -> Vec3 {
    let radius = position.length() as f64;

    if radius <= f64::EPSILON || params.m == 0 || flow_step_dt <= f32::EPSILON {
        return position;
    }

    let theta = ((position.y as f64) / radius).clamp(-1.0, 1.0).acos();
    let phi = (position.z as f64).atan2(position.x as f64);
    let velocity = probability_flow_velocity(position, params.m);
    let temp_position = position + velocity * flow_step_dt;
    let proposed_phi = (temp_position.z as f64).atan2(temp_position.x as f64);
    let phi_delta =
        wrapped_angle_delta(proposed_phi - phi).clamp(
            -PROBABILITY_FLOW_MAX_PHI_STEP_RADIANS,
            PROBABILITY_FLOW_MAX_PHI_STEP_RADIANS,
        );
    let new_phi = phi + phi_delta;

    spherical_to_cartesian(radius, theta, new_phi)
}

pub fn probability_flow_omega(position: Vec3, params: &OrbitalParams, flow_step_dt: f32) -> f32 {
    let radius = position.length() as f64;

    if radius <= f64::EPSILON || params.m == 0 || flow_step_dt <= f32::EPSILON {
        return 0.0;
    }

    let current_phi = (position.z as f64).atan2(position.x as f64);
    let next_position = advance_probability_flow(position, params, flow_step_dt);
    let next_phi = (next_position.z as f64).atan2(next_position.x as f64);
    let phi_delta = wrapped_angle_delta(next_phi - current_phi);

    (phi_delta as f32) / flow_step_dt
}

fn generate_spherical_density_cloud(params: &OrbitalParams) -> Vec<OrbitalPoint> {
    let abs_m = params.m.unsigned_abs().min(params.l);
    let radial_sampler = cached_radial_sampler(params.n, params.l);
    let theta_sampler = cached_theta_sampler(params.l, abs_m);
    let mut rng = rand::rng();
    let mut points = Vec::with_capacity(params.particle_count);

    for _ in 0..params.particle_count {
        let radius = sample_from_cdf(&radial_sampler, &mut rng);
        let theta = sample_from_cdf(&theta_sampler, &mut rng);
        // In the probability density |Y_l^m|^2, the phi part is uniform.
        let phi = rng.random_range(0.0..std::f64::consts::TAU);
        let position = spherical_to_cartesian(radius, theta, phi);
        let intensity = orbital_intensity(radius, theta, params.n, params.l, params.m) as f32;

        points.push(OrbitalPoint {
            color: Color::BLACK,
            intensity,
            position,
        });
    }

    points
}

fn spherical_density_label(params: &OrbitalParams) -> String {
    let shell = orbital_shell_label(params.l);

    if let Some(shell) = shell {
        format!("{}{} (m={})", params.n, shell, params.m)
    } else {
        format!("n={} l={} m={}", params.n, params.l, params.m)
    }
}

fn generate_real_basis_cloud(params: &OrbitalParams) -> Option<Vec<OrbitalPoint>> {
    let orbital = real_orbital_kind(params)?;
    let radial_sampler = cached_radial_sampler(params.n, params.l);
    let mut rng = rand::rng();
    let mut points = Vec::with_capacity(params.particle_count);

    for _ in 0..params.particle_count {
        let radius = sample_from_cdf(&radial_sampler, &mut rng);
        let direction = sample_real_basis_direction(&mut rng, orbital);
        let position = direction * radius as f32;
        let (theta, _phi) = cartesian_to_angles(position);
        let intensity = orbital_intensity(radius, theta, params.n, params.l, params.m) as f32;
        points.push(OrbitalPoint {
            color: Color::BLACK,
            intensity,
            position,
        });
    }

    Some(points)
}

fn real_orbital_label(params: &OrbitalParams) -> Option<&'static str> {
    match real_orbital_kind(params)? {
        RealOrbitalKind::Px => Some("p_x"),
        RealOrbitalKind::Py => Some("p_y"),
        RealOrbitalKind::Pz => Some("p_z"),
        RealOrbitalKind::Dxy => Some("d_xy"),
        RealOrbitalKind::Dyz => Some("d_yz"),
        RealOrbitalKind::Dz2 => Some("d_z2"),
        RealOrbitalKind::Dxz => Some("d_xz"),
        RealOrbitalKind::Dx2MinusY2 => Some("d_x2-y2"),
    }
}

fn real_orbital_kind(params: &OrbitalParams) -> Option<RealOrbitalKind> {
    match (params.l, params.m) {
        (1, -1) => Some(RealOrbitalKind::Px),
        (1, 0) => Some(RealOrbitalKind::Pz),
        (1, 1) => Some(RealOrbitalKind::Py),
        (2, -2) => Some(RealOrbitalKind::Dxy),
        (2, -1) => Some(RealOrbitalKind::Dyz),
        (2, 0) => Some(RealOrbitalKind::Dz2),
        (2, 1) => Some(RealOrbitalKind::Dxz),
        (2, 2) => Some(RealOrbitalKind::Dx2MinusY2),
        _ => None,
    }
}

fn orbital_shell_label(l: u32) -> Option<&'static str> {
    match l {
        0 => Some("s"),
        1 => Some("p"),
        2 => Some("d"),
        3 => Some("f"),
        4 => Some("g"),
        5 => Some("h"),
        6 => Some("i"),
        _ => None,
    }
}

fn build_radial_sampler(n: u32, l: u32) -> CdfSampler {
    let r_max = 10.0 * n as f64 * n as f64 * BOHR_RADIUS_SCALE;
    let step_size = r_max / (RADIAL_CDF_SAMPLES - 1) as f64;
    let mut cumulative = Vec::with_capacity(RADIAL_CDF_SAMPLES);
    let mut sum = 0.0;

    for index in 0..RADIAL_CDF_SAMPLES {
        let radius = index as f64 * step_size;
        let radial = hydrogen_radial_wavefunction(n, l, radius);
        let pdf = radius * radius * radial * radial;
        sum += pdf;
        cumulative.push(sum);
    }

    normalize_cdf(&mut cumulative, sum);

    CdfSampler {
        cumulative,
        step_size,
    }
}

fn build_theta_sampler(l: u32, abs_m: u32) -> CdfSampler {
    let step_size = std::f64::consts::PI / (THETA_CDF_SAMPLES - 1) as f64;
    let mut cumulative = Vec::with_capacity(THETA_CDF_SAMPLES);
    let mut sum = 0.0;

    for index in 0..THETA_CDF_SAMPLES {
        let theta = index as f64 * step_size;
        let x = theta.cos();
        let associated_legendre = associated_legendre(l, abs_m, x);
        let pdf = theta.sin() * associated_legendre * associated_legendre;
        sum += pdf;
        cumulative.push(sum);
    }

    normalize_cdf(&mut cumulative, sum);

    CdfSampler {
        cumulative,
        step_size,
    }
}

fn cached_radial_sampler(n: u32, l: u32) -> CdfSampler {
    static CACHE: OnceLock<Mutex<RadialCache>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut cache = cache.lock().expect("radial sampler cache poisoned");

    cache
        .entry((n, l))
        .or_insert_with(|| build_radial_sampler(n, l))
        .clone()
}

fn cached_theta_sampler(l: u32, abs_m: u32) -> CdfSampler {
    static CACHE: OnceLock<Mutex<ThetaCache>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut cache = cache.lock().expect("theta sampler cache poisoned");

    cache
        .entry((l, abs_m))
        .or_insert_with(|| build_theta_sampler(l, abs_m))
        .clone()
}

fn normalize_cdf(cumulative: &mut [f64], sum: f64) {
    if sum <= f64::EPSILON {
        let last_index = cumulative.len().saturating_sub(1).max(1);

        for (index, value) in cumulative.iter_mut().enumerate() {
            *value = index as f64 / last_index as f64;
        }

        return;
    }

    for value in cumulative.iter_mut() {
        *value /= sum;
    }
}

fn sample_from_cdf(sampler: &CdfSampler, rng: &mut impl Rng) -> f64 {
    let target = rng.random::<f64>();
    let index = sampler.cumulative.partition_point(|value| *value < target);
    let clamped_index = index.min(sampler.cumulative.len().saturating_sub(1));

    if clamped_index == 0 {
        return 0.0;
    }

    let previous_cdf = sampler.cumulative[clamped_index - 1];
    let current_cdf = sampler.cumulative[clamped_index];
    let span = (current_cdf - previous_cdf).max(f64::EPSILON);
    let local_t = ((target - previous_cdf) / span).clamp(0.0, 1.0);

    ((clamped_index - 1) as f64 + local_t) * sampler.step_size
}

fn sample_real_basis_direction(rng: &mut impl Rng, orbital: RealOrbitalKind) -> Vec3 {
    loop {
        let direction = sample_unit_direction(rng);
        let weight = real_basis_weight(direction, orbital);

        if rng.random::<f32>() <= weight * weight {
            return direction;
        }
    }
}

fn probability_flow_velocity(position: Vec3, magnetic_quantum_number: i32) -> Vec3 {
    let radius = position.length() as f64;

    if radius <= f64::EPSILON {
        return Vec3::ZERO;
    }

    let theta = ((position.y as f64) / radius).clamp(-1.0, 1.0).acos();
    let phi = (position.z as f64).atan2(position.x as f64);
    let sin_theta = theta.sin().abs().max(PROBABILITY_FLOW_SIN_THETA_FLOOR);
    let softened_radius = radius.max(PROBABILITY_FLOW_RADIUS_SOFTENING);
    let velocity_magnitude = magnetic_quantum_number as f64 / (softened_radius * sin_theta);
    let vx = -velocity_magnitude * phi.sin();
    let vz = velocity_magnitude * phi.cos();

    Vec3::new(vx as f32, 0.0, vz as f32)
}

fn wrapped_angle_delta(delta: f64) -> f64 {
    let tau = std::f64::consts::TAU;
    (delta + std::f64::consts::PI).rem_euclid(tau) - std::f64::consts::PI
}

fn spherical_to_cartesian(radius: f64, theta: f64, phi: f64) -> Vec3 {
    let sin_theta = theta.sin();
    let x = radius * sin_theta * phi.cos();
    let y = radius * theta.cos();
    let z = radius * sin_theta * phi.sin();

    Vec3::new(x as f32, y as f32, z as f32)
}

fn cartesian_to_angles(position: Vec3) -> (f64, f64) {
    let radius = position.length() as f64;

    if radius <= f64::EPSILON {
        return (0.0, 0.0);
    }

    let theta = ((position.y as f64) / radius).clamp(-1.0, 1.0).acos();
    let phi = (position.z as f64).atan2(position.x as f64);

    (theta, phi)
}

fn sample_unit_direction(rng: &mut impl Rng) -> Vec3 {
    let z = rng.random_range(-1.0_f32..=1.0_f32);
    let phi = rng.random_range(0.0_f32..std::f32::consts::TAU);
    let xy = (1.0_f32 - z * z).sqrt();

    Vec3::new(xy * phi.cos(), z, xy * phi.sin())
}

fn hydrogen_radial_wavefunction(n: u32, l: u32, radius: f64) -> f64 {
    let rho = 2.0 * radius / (n as f64 * BOHR_RADIUS_SCALE);
    let k = n - l - 1;
    let alpha = 2 * l + 1;
    let laguerre = associated_laguerre(k, alpha, rho);
    let normalization = ((2.0 / (n as f64 * BOHR_RADIUS_SCALE)).powi(3) * gamma_integer(n - l)
        / (2.0 * n as f64 * gamma_integer(n + l + 1)))
    .sqrt();

    normalization * (-rho / 2.0).exp() * rho.powi(l as i32) * laguerre
}

fn associated_laguerre(k: u32, alpha: u32, rho: f64) -> f64 {
    if k == 0 {
        return 1.0;
    }

    let mut current = 1.0 + alpha as f64 - rho;

    if k == 1 {
        return current;
    }

    let mut previous = 1.0;

    for j in 2..=k {
        let next = ((2 * j - 1 + alpha) as f64 - rho) * current - (j - 1 + alpha) as f64 * previous;
        let next = next / j as f64;

        previous = current;
        current = next;
    }

    current
}

fn associated_legendre(l: u32, m: u32, x: f64) -> f64 {
    let mut p_mm = 1.0;

    if m > 0 {
        let one_minus_x2 = ((1.0 - x) * (1.0 + x)).max(0.0).sqrt();
        let mut factor = 1.0;

        for _ in 1..=m {
            p_mm *= -factor * one_minus_x2;
            factor += 2.0;
        }
    }

    if l == m {
        return p_mm;
    }

    let mut p_m1m = x * (2 * m + 1) as f64 * p_mm;

    if l == m + 1 {
        return p_m1m;
    }

    for current_l in (m + 2)..=l {
        let p_ll = ((2 * current_l - 1) as f64 * x * p_m1m - (current_l + m - 1) as f64 * p_mm)
            / (current_l - m) as f64;

        p_mm = p_m1m;
        p_m1m = p_ll;
    }

    p_m1m
}

fn gamma_integer(value: u32) -> f64 {
    if value <= 1 {
        return 1.0;
    }

    let mut product = 1.0;

    for term in 1..value {
        product *= term as f64;
    }

    product
}

fn orbital_intensity(radius: f64, theta: f64, n: u32, l: u32, m: i32) -> f64 {
    let radial = hydrogen_radial_wavefunction(n, l, radius).powi(2);
    let angular = associated_legendre(l, m.unsigned_abs().min(l), theta.cos()).powi(2);
    radial * angular
}

fn apply_dynamic_color_mapping(points: &mut [OrbitalPoint]) {
    if points.is_empty() {
        return;
    }

    let mut intensities: Vec<f32> = points.iter().map(|point| point.intensity.max(0.0)).collect();
    intensities.sort_by(|left, right| left.total_cmp(right));

    let lower = quantile_value(&intensities, COLOR_DISTRIBUTION_LOWER_PERCENTILE);
    let upper = quantile_value(&intensities, COLOR_DISTRIBUTION_UPPER_PERCENTILE);
    let span = (upper - lower).max(f32::EPSILON);

    for point in points {
        let normalized = ((point.intensity - lower) / span).clamp(0.0, 1.0);
        point.color = heatmap_fire(normalized);
    }
}

fn quantile_value(sorted_values: &[f32], quantile: f32) -> f32 {
    let last_index = sorted_values.len().saturating_sub(1);
    let scaled_index = (last_index as f32 * quantile.clamp(0.0, 1.0)).round() as usize;
    sorted_values[scaled_index.min(last_index)]
}


fn heatmap_fire(value: f32) -> Color {
    let value = value.clamp(0.0, 1.0);
    // Rust-tuned version:
    // let value = contrast_curve(value);
    // let stops = [
    //     (0.00_f32, 0.18_f32, 0.03_f32, 0.30_f32), // deep purple
    //     (0.20_f32, 0.55_f32, 0.00_f32, 0.75_f32), // purple
    //     (0.40_f32, 0.95_f32, 0.15_f32, 0.85_f32), // pink
    //     (0.55_f32, 0.90_f32, 0.05_f32, 0.05_f32), // red
    //     (0.66_f32, 1.00_f32, 0.45_f32, 0.00_f32), // orange
    //     (0.76_f32, 1.00_f32, 0.95_f32, 0.10_f32), // yellow
    //     (1.00_f32, 1.00_f32, 1.00_f32, 1.00_f32), // white
    // ];

    // C++ `atom_realtime.cpp` version:
    let stops = [
        (0.0_f32, 0.00_f32, 0.00_f32, 0.00_f32), // black
        (0.13_f32, 0.25_f32, 0.00_f32, 0.495_f32), // black -> dark purple midpoint
        (0.23_f32, 0.50_f32, 0.00_f32, 0.99_f32), // dark purple
        (0.33_f32, 0.65_f32, 0.00_f32, 0.495_f32), // dark purple -> deep red midpoint
        (0.43_f32, 0.80_f32, 0.00_f32, 0.00_f32), // deep red
        (0.53_f32, 0.90_f32, 0.25_f32, 0.00_f32), // deep red -> orange midpoint
        (0.63_f32, 1.00_f32, 0.50_f32, 0.00_f32), // orange
        (0.73_f32, 1.00_f32, 0.75_f32, 0.00_f32), // orange -> yellow midpoint
        (0.83_f32, 1.00_f32, 1.00_f32, 0.00_f32), // yellow
        (0.93_f32, 1.00_f32, 1.00_f32, 0.50_f32), // yellow -> white midpoint
        (1.0_f32, 1.00_f32, 1.00_f32, 1.00_f32), // white
    ];

    let mut index = 0;
    while index + 1 < stops.len() && value > stops[index + 1].0 {
        index += 1;
    }

    let next = (index + 1).min(stops.len() - 1);
    let (p1, r1, g1, b1) = stops[index];
    let (p2, r2, g2, b2) = stops[next];
    let span = (p2 - p1).max(f32::EPSILON);
    let local_t = ((value - p1) / span).clamp(0.0, 1.0);

    Color::srgb(
        r1 + local_t * (r2 - r1),
        g1 + local_t * (g2 - g1),
        b1 + local_t * (b2 - b1),
    )
}

#[derive(Clone, Copy)]
enum RealOrbitalKind {
    Px,
    Py,
    Pz,
    Dxy,
    Dyz,
    Dz2,
    Dxz,
    Dx2MinusY2,
}

fn real_basis_weight(direction: Vec3, orbital: RealOrbitalKind) -> f32 {
    match orbital {
        RealOrbitalKind::Px => direction.x,
        RealOrbitalKind::Py => direction.y,
        RealOrbitalKind::Pz => direction.z,
        RealOrbitalKind::Dxy => 2.0 * direction.x * direction.y,
        RealOrbitalKind::Dyz => 2.0 * direction.y * direction.z,
        RealOrbitalKind::Dz2 => 0.5 * (3.0 * direction.z * direction.z - 1.0),
        RealOrbitalKind::Dxz => 2.0 * direction.x * direction.z,
        RealOrbitalKind::Dx2MinusY2 => direction.x * direction.x - direction.y * direction.y,
    }
}
