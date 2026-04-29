use bevy::prelude::*;

use crate::{physics, render};

#[derive(Resource, Default)]
pub struct HostCommandQueue(pub Vec<HostCommand>);

#[derive(Resource, Clone, Debug)]
pub struct HostSnapshot {
    pub n: u32,
    pub l: u32,
    pub m: i32,
    pub particle_count: usize,
    pub display_mode: physics::DisplayMode,
    pub display_mode_label: &'static str,
    pub rendered_cloud_label: String,
}

impl Default for HostSnapshot {
    fn default() -> Self {
        let display_mode = physics::DisplayMode::default();
        let params = physics::OrbitalParams::default();

        Self {
            n: params.n,
            l: params.l,
            m: params.m,
            particle_count: params.particle_count,
            display_mode,
            display_mode_label: display_mode.label(),
            rendered_cloud_label: physics::rendered_cloud_label(&params, display_mode),
        }
    }
}

#[derive(Clone, Debug)]
pub enum HostCommand {
    SetQuantumNumbers { n: u32, l: u32, m: i32 },
    SetParticleCount(usize),
    SetDisplayMode(physics::DisplayMode),
    ToggleDisplayMode,
    Regenerate,
    ResetCamera,
}

pub fn enqueue_command(app: &mut App, command: HostCommand) {
    app.world_mut()
        .resource_mut::<HostCommandQueue>()
        .0
        .push(command);
}

pub fn snapshot(app: &App) -> HostSnapshot {
    app.world().resource::<HostSnapshot>().clone()
}

pub fn apply_host_commands(
    mut command_queue: ResMut<HostCommandQueue>,
    mut params: ResMut<physics::OrbitalParams>,
    mut display_mode: ResMut<physics::DisplayMode>,
    mut needs_regeneration: ResMut<render::CloudNeedsRegeneration>,
    mut pending_camera_reset: ResMut<render::PendingCameraReset>,
) {
    let commands = std::mem::take(&mut command_queue.0);
    let mut orbital_changed = false;

    for command in commands {
        match command {
            HostCommand::SetQuantumNumbers { n, l, m } => {
                params.n = n;
                params.l = l;
                params.m = m;
                orbital_changed = true;
            }
            HostCommand::SetParticleCount(particle_count) => {
                params.particle_count = particle_count.max(1);
                orbital_changed = true;
            }
            HostCommand::SetDisplayMode(next_mode) => {
                if *display_mode != next_mode {
                    *display_mode = next_mode;
                    orbital_changed = true;
                }
            }
            HostCommand::ToggleDisplayMode => {
                display_mode.toggle();
                orbital_changed = true;
            }
            HostCommand::Regenerate => {
                needs_regeneration.0 = true;
            }
            HostCommand::ResetCamera => {
                pending_camera_reset.0 = true;
            }
        }
    }

    if orbital_changed {
        params.normalize_quantum_numbers();
        needs_regeneration.0 = true;
    }
}

pub fn sync_host_snapshot(
    params: Res<physics::OrbitalParams>,
    display_mode: Res<physics::DisplayMode>,
    mut snapshot: ResMut<HostSnapshot>,
) {
    snapshot.n = params.n;
    snapshot.l = params.l;
    snapshot.m = params.m;
    snapshot.particle_count = params.particle_count;
    snapshot.display_mode = *display_mode;
    snapshot.display_mode_label = display_mode.label();
    snapshot.rendered_cloud_label = physics::rendered_cloud_label(&params, *display_mode);
}
