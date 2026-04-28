use bevy::prelude::*;

use crate::{physics, render};

const PARTICLE_COUNT_STEP: usize = 1_000;
const FLOW_SPEED_STEP: f32 = 0.25;

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins.set(render::window_plugin()))
        .add_plugins(render::RenderPlugin)
        .insert_resource(physics::DisplayMode::default())
        .insert_resource(physics::FlowAnimation::default())
        .insert_resource(physics::OrbitalParams::default())
        .add_systems(Update, handle_cloud_input)
        .run();
}

fn handle_cloud_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut params: ResMut<physics::OrbitalParams>,
    mut display_mode: ResMut<physics::DisplayMode>,
    mut flow_animation: ResMut<physics::FlowAnimation>,
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

    if keyboard.just_pressed(KeyCode::Space) {
        flow_animation.toggle_paused();
        info!(
            "Probability-flow animation is now {}",
            flow_animation.status_label()
        );
    }

    if keyboard.just_pressed(KeyCode::KeyY) {
        flow_animation.adjust_speed(FLOW_SPEED_STEP);
        info!(
            "Probability-flow speed is now {:.2}x",
            flow_animation.speed_multiplier
        );
    }

    if keyboard.just_pressed(KeyCode::KeyH) {
        flow_animation.adjust_speed(-FLOW_SPEED_STEP);
        info!(
            "Probability-flow speed is now {:.2}x",
            flow_animation.speed_multiplier
        );
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
