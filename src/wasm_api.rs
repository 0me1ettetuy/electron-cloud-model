use std::cell::RefCell;

use bevy::prelude::*;
use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::{
    app::{build_app_with_config, AppConfig},
    host_api::{HostCommand, HostCommandQueue, HostSnapshot},
    render,
};

thread_local! {
    static BROWSER_BRIDGE: RefCell<BrowserBridge> = RefCell::new(BrowserBridge::default());
}

#[derive(Default)]
struct BrowserBridge {
    started: bool,
    queued_commands: Vec<HostCommand>,
    latest_snapshot: Option<WasmSnapshot>,
}

#[derive(Clone, Serialize)]
struct WasmSnapshot {
    n: u32,
    l: u32,
    m: i32,
    particle_count: usize,
    display_mode: String,
    rendered_cloud_label: String,
}

#[wasm_bindgen]
pub fn start_app(canvas_selector: Option<String>) -> Result<(), JsValue> {
    let already_started = BROWSER_BRIDGE.with(|bridge| bridge.borrow().started);
    if already_started {
        return Err(JsValue::from_str("Rust app is already running"));
    }

    console_error_panic_hook::set_once();

    BROWSER_BRIDGE.with(|bridge| {
        bridge.borrow_mut().started = true;
    });

    let mut config = AppConfig::default();
    config.window_host = render::WindowHostConfig { canvas_selector };
    build_app_with_config(config).run();
    Ok(())
}

#[wasm_bindgen]
pub fn is_app_started() -> bool {
    BROWSER_BRIDGE.with(|bridge| bridge.borrow().started)
}

#[wasm_bindgen]
pub fn get_snapshot() -> Result<JsValue, JsValue> {
    let snapshot = BROWSER_BRIDGE.with(|bridge| bridge.borrow().latest_snapshot.clone());
    let snapshot = snapshot.ok_or_else(|| JsValue::from_str("No snapshot available yet"))?;
    serde_wasm_bindgen::to_value(&snapshot)
        .map_err(|err| JsValue::from_str(&format!("snapshot serialization failed: {err}")))
}

#[wasm_bindgen]
pub fn set_quantum_numbers(n: u32, l: u32, m: i32) {
    queue_browser_command(HostCommand::SetQuantumNumbers { n, l, m });
}

#[wasm_bindgen]
pub fn set_particle_count(particle_count: usize) {
    queue_browser_command(HostCommand::SetParticleCount(particle_count));
}

#[wasm_bindgen]
pub fn regenerate() {
    queue_browser_command(HostCommand::Regenerate);
}

#[wasm_bindgen]
pub fn toggle_display_mode() {
    queue_browser_command(HostCommand::ToggleDisplayMode);
}

#[wasm_bindgen]
pub fn reset_camera() {
    queue_browser_command(HostCommand::ResetCamera);
}

fn queue_browser_command(command: HostCommand) {
    BROWSER_BRIDGE.with(|bridge| {
        bridge.borrow_mut().queued_commands.push(command);
    });
}

pub fn pull_browser_commands(mut host_queue: ResMut<HostCommandQueue>) {
    let commands = BROWSER_BRIDGE.with(|bridge| {
        let mut bridge = bridge.borrow_mut();
        std::mem::take(&mut bridge.queued_commands)
    });

    host_queue.0.extend(commands);
}

pub fn publish_browser_snapshot(snapshot: Res<HostSnapshot>) {
    let next_snapshot = WasmSnapshot {
        n: snapshot.n,
        l: snapshot.l,
        m: snapshot.m,
        particle_count: snapshot.particle_count,
        display_mode: snapshot.display_mode_label.to_string(),
        rendered_cloud_label: snapshot.rendered_cloud_label.clone(),
    };

    BROWSER_BRIDGE.with(|bridge| {
        bridge.borrow_mut().latest_snapshot = Some(next_snapshot);
    });
}
