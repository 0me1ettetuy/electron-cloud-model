# Electron Cloud Model Plan

## Goal

Build a Rust version of the `Atoms` C++ project inside this repository.

The long-term target is to cover the same major functionality:

- 3D realtime orbital cloud viewer
- 2D Bohr-style teaching/demo mode
- raytraced or raytrace-like presentation mode
- controls for quantum numbers and particle counts

## Source Repos

- `Atoms/`
  - current C++ reference implementation
- `electron-cloud-model/`
  - Rust rewrite target

## Current Decision

This repository is the main Rust project going forward.

We are not doing a line-by-line port from C++.
We are rebuilding the app in phases so the code stays understandable for a Rust beginner.

## Why This Order

The C++ project mixes math, rendering, input, and state into large files.
That is fine for experiments, but it is harder to learn from and harder to extend in Rust.

So we will split the work into simple layers:

- `src/physics`
  - orbital formulas and sampling
- `src/render`
  - drawing, camera, UI
- `src/modes`
  - app modes like realtime, 2D, and raytraced
- `src/app.rs`
  - app startup and top-level flow

## Phase Plan

### Phase 1: Rust Project Base

Status: started

Goal:

- keep this repo as the single Rust home
- add a written roadmap
- create a clean source layout

Deliverables:

- `PLAN.md`
- base source folders
- compileable placeholder app

### Phase 2: Realtime Orbital Viewer MVP

Goal:

- open a window
- render a 3D orbital cloud
- generate particle positions from `n`, `l`, `m`, and `N`
- allow regenerate on value changes

Core C++ reference:

- `Atoms/src/atom_realtime.cpp`

Port first:

- radial sampling
- theta sampling
- phi sampling
- spherical-to-cartesian conversion
- color mapping

Do not port yet:

- raytracer
- 2D mode

### Phase 3: Controls and UI

Goal:

- add UI controls for:
  - `n`
  - `l`
  - `m`
  - particle count
- add mode switching
- show current values on screen

### Phase 4: Probability-Flow Animation

Goal:

- animate particles using the probability-flow logic
- allow pause/resume
- allow regenerate vs animate behavior

Core C++ reference:

- `calculateProbabilityFlow(...)` in `Atoms/src/atom_realtime.cpp`

### Phase 5: 2D Bohr Mode

Goal:

- rebuild the simpler 2D atom demo
- keep it as a separate app mode, not a separate project

Core C++ reference:

- `Atoms/src/atom.cpp`

### Phase 6: Raytraced Mode

Goal:

- add a high-quality presentation mode

Important note:

This is the hardest feature.
We should first decide whether to build:

- a true raytraced mode
- or a visually similar advanced render mode

Core C++ reference:

- `Atoms/src/atom_raytracer.cpp`

## Beginner Notes

Rust concepts to learn as we build:

1. `fn`, `let`, `mut`
2. `struct`
3. `impl`
4. `Vec<T>`
5. references with `&`
6. `Option<T>` and `Result<T, E>`
7. modules and files

We do not need to learn advanced Rust before starting.
We will learn only the pieces needed for the next step.

## Rules For This Rewrite

- do not port giant C++ files directly
- keep files small and focused
- keep the project compileable after each step
- prefer simple code over clever code
- write comments for confusing Rust parts

## Immediate Next Step

Build Phase 2:

- choose the rendering stack
- create a real windowed app
- port the orbital point generation code first

## Questions Already Answered

### Do we want the full functionality of the C++ app?

Yes.

### Are we still building in phases?

Yes.
That is the safest way to reach full functionality without getting lost.
