use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::physics::{self, DisplayMode, OrbitalParams};

#[derive(Clone, Debug, Serialize)]
pub struct WebOrbitalData {
    pub n: u32,
    pub l: u32,
    pub m: i32,
    pub particle_count: usize,
    pub display_mode: &'static str,
    pub rendered_cloud_label: String,
    pub positions: Vec<f32>,
    pub colors: Vec<f32>,
    pub omegas: Vec<f32>,
}

impl WebOrbitalData {
    pub fn file_stem(&self) -> String {
        format!("n{}_l{}_m{}", self.n, self.l, self.m)
    }
}

pub fn build_web_orbital_data(
    params: &OrbitalParams,
    display_mode: DisplayMode,
) -> WebOrbitalData {
    let cloud = physics::generate_cloud(params, display_mode);
    let mut positions = Vec::with_capacity(cloud.len() * 3);
    let mut colors = Vec::with_capacity(cloud.len() * 3);
    let mut omegas = Vec::with_capacity(cloud.len());

    for point in cloud {
        let srgb = point.color.to_srgba();
        positions.extend_from_slice(&[point.position.x, point.position.y, point.position.z]);
        colors.extend_from_slice(&[srgb.red, srgb.green, srgb.blue]);
        omegas.push(physics::probability_flow_omega(
            point.position,
            params,
            physics::FlowAnimation.flow_step_dt(),
        ));
    }

    WebOrbitalData {
        n: params.n,
        l: params.l,
        m: params.m,
        particle_count: params.particle_count,
        display_mode: display_mode.label(),
        rendered_cloud_label: physics::rendered_cloud_label(params, display_mode),
        positions,
        colors,
        omegas,
    }
}

pub fn write_web_orbital_data(
    output_dir: impl AsRef<Path>,
    data: &WebOrbitalData,
) -> std::io::Result<PathBuf> {
    let output_dir = output_dir.as_ref();
    fs::create_dir_all(output_dir)?;

    let path = output_dir.join(format!("{}.json", data.file_stem()));
    let bytes = serde_json::to_vec(data).expect("web orbital data should serialize");
    fs::write(&path, bytes)?;

    Ok(path)
}
