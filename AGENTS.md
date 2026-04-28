# AGENTS.md

This file is the repo-level quick-start for future Codex/agent sessions.

## First Read Order

When starting fresh in this repo, read files in this order:

1. `PLAN.md`
2. `src/render/mod.rs`
3. `src/physics/mod.rs`
4. `src/app.rs`

`PLAN.md` is the main project brief and should be treated as the primary handoff file.

## Project Purpose

This repository is the Rust rewrite target for the `../Atoms` C++ project.

Primary goal:

- faithful Rust rewrite of the `Atoms` realtime orbital viewer
- browser/WASM-friendly architecture
- preserve behavior parity where practical

Current parity reference:

- `../Atoms/src/atom_realtime.cpp`

Related live reference:

- `https://www.kavang.com/atom`

Important note:

- the live web app is related to `Atoms`, but it is not a direct runtime of the native C++ app
- it is useful as a visual parity target, especially for composition, palette feel, and UI behavior

## Current Working State

The app currently has:

- generic hydrogen-orbital cloud generation from `n`, `l`, `m`
- radial and theta CDF sampling
- probability-flow animation
- combined cloud mesh rendering
- orbit camera
- camera-facing cutaway
- dynamic per-cloud color remapping based on the current cloud's intensity distribution

## Current Color Behavior

Color is currently adaptive per cloud, not globally fixed.

High-level flow:

- generate cloud points
- compute per-point orbital intensity from radial and angular probability terms
- sort intensities for the current cloud
- use the middle 90% of that cloud's intensity distribution for palette remapping
- map through a C++-style palette with stop positions shifted upward by `+0.03`

This means different orbitals should use the palette more evenly instead of collapsing into mostly purple or mostly white.

## Current Rendering Notes

These are the current intentional render choices:

- crisp rendering is active
- blur/glow variants are preserved as comments in `src/render/mod.rs`
- particle template radius is currently tuned visually and may change often
- far-particle size scaling exists in the renderer
- cutaway is camera-facing, not web-style axis-aligned clipping

If visual parity work resumes, inspect:

- `particle_scale_for_radius(...)`
- `particle_opacity(...)`
- cloud material settings in `setup_scene(...)`

## Workflow Expectations

- keep explanations beginner-friendly for Rust
- prefer small iterative changes
- avoid large rewrites unless clearly necessary
- keep the project compiling after each step
- do not revert unrelated user changes

## If Doing Visual Parity Work

Before making conclusions from screenshots, check:

- particle count matches the reference view
- same `n`, `l`, `m`
- same cutaway/clipping mode
- same camera angle/FOV
- whether blur is active or commented out

## If Unsure What To Do Next

Default next-step priority:

1. maintain compile health
2. preserve `PLAN.md` direction
3. improve visual parity against `atom_realtime.cpp` and the web reference
4. avoid adding new modes unless explicitly requested
