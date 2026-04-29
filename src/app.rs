use bevy::prelude::*;

use crate::{host_api, physics, render};

const PARTICLE_COUNT_STEP: usize = 1_000;

#[derive(Clone, Default)]
pub struct AppConfig {
    pub window_host: render::WindowHostConfig,
}

pub fn build_app() -> App {
    build_app_with_config(AppConfig::default())
}

pub fn build_app_with_config(config: AppConfig) -> App {
    let mut app = App::new();

    app
        .add_plugins(DefaultPlugins.set(render::window_plugin(&config.window_host)))
        .add_plugins(render::RenderPlugin)
        .insert_resource(physics::DisplayMode::default())
        .insert_resource(physics::FlowAnimation::default())
        .insert_resource(physics::OrbitalParams::default())
        .insert_resource(host_api::HostCommandQueue::default())
        .insert_resource(host_api::HostSnapshot::default())
        .add_systems(
            Update,
            (
                host_api::apply_host_commands,
                handle_cloud_input,
                host_api::sync_host_snapshot,
            )
                .chain(),
        );

    #[cfg(target_arch = "wasm32")]
    app.add_systems(
        Update,
        (
            crate::wasm_api::pull_browser_commands,
            crate::wasm_api::publish_browser_snapshot,
        ),
    );

    app
}

pub fn run() {
    build_app().run();
}

fn handle_cloud_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut params: ResMut<physics::OrbitalParams>,
    mut display_mode: ResMut<physics::DisplayMode>,
    mut needs_regeneration: ResMut<render::CloudNeedsRegeneration>,
) {
    let mut changed = false;
    let mut manual_regeneration = false;

    if keyboard.just_pressed(KeyCode::KeyQ) {
        manual_regeneration = true;
    }

    if keyboard.just_pressed(KeyCode::KeyV) {
        display_mode.toggle();
        changed = true;
    }

    if keyboard.just_pressed(KeyCode::KeyW) {
        params.n += 1;
        changed = true;
    }

    if keyboard.just_pressed(KeyCode::KeyS) && params.n > 1 {
        params.n -= 1;
        changed = true;
    }

    if keyboard.just_pressed(KeyCode::KeyE) {
        params.l += 1;
        changed = true;
    }

    if keyboard.just_pressed(KeyCode::KeyD) && params.l > 0 {
        params.l -= 1;
        changed = true;
    }

    if keyboard.just_pressed(KeyCode::KeyR) {
        params.m += 1;
        changed = true;
    }

    if keyboard.just_pressed(KeyCode::KeyF) {
        params.m -= 1;
        changed = true;
    }

    if keyboard.just_pressed(KeyCode::KeyT) {
        params.particle_count += PARTICLE_COUNT_STEP;
        changed = true;
    }

    if keyboard.just_pressed(KeyCode::KeyG) && params.particle_count > PARTICLE_COUNT_STEP {
        params.particle_count -= PARTICLE_COUNT_STEP;
        changed = true;
    }

    if changed {
        params.normalize_quantum_numbers();
        needs_regeneration.0 = true;
        info!(
            "Regenerating cloud for n={}, l={}, m={} with {} particles in {} mode",
            params.n,
            params.l,
            params.m,
            params.particle_count,
            display_mode.label()
        );
    }

    if manual_regeneration {
        needs_regeneration.0 = true;
        info!("Regenerating cloud with current orbital settings");
    }
}
