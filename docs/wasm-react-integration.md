# WASM React Integration

This document describes the intended Part B integration shape.

## Goal

React owns the browser UI.
Rust owns the viewer runtime.

React does not need to know Bevy internals.
It only needs a small browser-facing API.

## Mounting Model

The React app should render a container for the Rust canvas.

Example flow:

1. React renders a container element with a stable selector or id.
2. React loads the Rust WASM package.
3. React calls `start_app("#rust-canvas")`.
4. Bevy attaches its canvas to that selector.
5. React talks to Rust only through exported commands and snapshot reads.

## Exported Browser API

The current WASM-facing exports are:

- `start_app(canvas_selector)`
- `is_app_started()`
- `get_snapshot()`
- `set_quantum_numbers(n, l, m)`
- `set_particle_count(count)`
- `regenerate()`
- `toggle_display_mode()`
- `reset_camera()`

## Snapshot Shape

`get_snapshot()` returns a serializable state object with:

- `n`
- `l`
- `m`
- `particle_count`
- `display_mode`
- `rendered_cloud_label`

## React Control Model

The current Rust-side scope assumes React may control:

- `n`
- `l`
- `m`
- particle count
- regenerate
- display mode toggle
- reset camera

Camera drag and zoom still happen directly on the Rust canvas.

## Important Note

This bridge is intentionally small.

If React needs more viewer state later, extend the snapshot and commands instead of reaching into Bevy internals from JavaScript.
