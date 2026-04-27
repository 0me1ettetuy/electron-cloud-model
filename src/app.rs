use bevy::prelude::*;

use crate::{physics, render};

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins.set(render::window_plugin()))
        .add_plugins(render::RenderPlugin)
        .insert_resource(physics::OrbitalParams::default())
        .add_systems(Update, handle_cloud_input)
        .run();
}

fn handle_cloud_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut params: ResMut<physics::OrbitalParams>,
    mut needs_regeneration: ResMut<render::CloudNeedsRegeneration>,
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

    if keyboard.just_pressed(KeyCode::ArrowRight) {
        params.n += 1;
        changed = true;
    }

    if keyboard.just_pressed(KeyCode::ArrowLeft) && params.n > 1 {
        params.n -= 1;
        changed = true;
    }

    if keyboard.just_pressed(KeyCode::KeyD) && params.l < params.n.saturating_sub(1) {
        params.l += 1;
        changed = true;
    }

    if keyboard.just_pressed(KeyCode::KeyA) && params.l > 0 {
        params.l -= 1;
        changed = true;
    }

    if keyboard.just_pressed(KeyCode::KeyW) {
        params.m += 1;
        changed = true;
    }

    if keyboard.just_pressed(KeyCode::KeyS) {
        params.m -= 1;
        changed = true;
    }

    if changed {
        params.normalize_quantum_numbers();
        needs_regeneration.0 = true;
        info!(
            "Regenerating cloud for n={}, l={}, m={} with {} particles",
            params.n, params.l, params.m, params.particle_count
        );
    }
}
