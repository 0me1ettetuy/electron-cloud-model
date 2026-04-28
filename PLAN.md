# Electron Cloud Model Plan

## What This Repo Is

This repository is the Rust rewrite target for the `Atoms` C++ project.

Reference source:

- `../Atoms/`
  - current C++ implementation
- `./`
  - Rust rewrite target

This file is both:

- the project roadmap
- the standing context brief for future work sessions

If a future helper or agent reads only one file first, it should be this one.

## Core Project Direction

The goal is a faithful Rust rewrite of `Atoms` with a browser-first WebAssembly target.

That means:

- keep the same major features and behavior as the C++ project
- use Rust code and Rust project structure instead of copying C++ file layout directly
- prefer web-friendly architecture and dependencies
- keep native desktop support optional and secondary unless needed for local debugging

Important clarification:

- this is not a "Rust-inspired remake"
- this is intended to become the Rust version of `Atoms`
- feature parity matters
- behavior parity matters
- code does not need to be a literal line-by-line translation

## Main Targets

Long-term feature targets:

- 3D realtime orbital cloud viewer
- raytraced or raytrace-like presentation mode
- controls for quantum numbers and particle counts
- browser/WASM build as a first-class target

Current focus clarification:

- the immediate goal is the `atom_realtime.cpp` experience from `Atoms`
- the 2D Bohr-style mode is currently not needed for the active workstream
- the raytracer may still be needed later, but it is not the current implementation target

## Platform Direction

Primary target:

- `wasm32-unknown-unknown`
- browser delivery

Secondary target:

- native desktop build for local iteration when useful

When choosing libraries or architecture, prefer options that do not block WASM support.

## Developer Context

The main developer on this repo:

- is a total beginner in Rust
- has only minor C++ experience
- is stronger in Python and JavaScript
- has a web developer background

This should affect how work is done:

- explain Rust in beginner-friendly terms
- compare Rust ideas to Python and JavaScript when useful
- avoid unnecessary advanced Rust patterns
- keep files small and readable
- keep the project compiling after each step
- include short comments when Rust syntax is likely to be confusing

## Working Style For Future Sessions

Assume the following unless explicitly changed:

- explain decisions, not just results
- prefer small incremental steps over large rewrites
- port one feature slice at a time
- do not require the user to restate the repo goal every session
- when possible, point back to this file instead of asking for repeated background

When proposing changes, prefer:

1. the simplest version that works
2. clear module boundaries
3. WASM-safe choices
4. easy-to-learn Rust over clever Rust

## Rewrite Rules

- preserve the spirit and behavior of the C++ project
- do not blindly copy giant C++ files into giant Rust files
- split physics, rendering, app state, and modes into smaller Rust modules
- keep names recognizable when that helps map Rust code to the C++ reference
- keep the project buildable after each milestone
- prefer straightforward code over abstraction-heavy code
- document non-obvious Rust or math choices

## Suggested Source Layout

- `src/physics`
  - orbital formulas, sampling, probability density, flow math
- `src/render`
  - drawing, camera, materials, UI integration
- `src/modes`
  - realtime, 2D, raytraced or raytrace-like modes
- `src/app.rs`
  - app startup and top-level orchestration

This structure is allowed to evolve if the real implementation suggests something simpler.

## Phase Plan

### Phase 1: Rust Project Base

Status: done

Goal:

- keep this repo as the single Rust home
- add a written roadmap and standing context
- create a clean source layout

Deliverables:

- `PLAN.md`
- base source folders
- compilable placeholder app

### Phase 2: Realtime Orbital Viewer MVP

Status: in progress

Goal:

- create a real app entry point compatible with the chosen web/WASM path
- render a first 3D orbital cloud
- generate particle positions from `n`, `l`, `m`, and `N`
- support regenerating the cloud when values change

Core C++ reference:

- `Atoms/src/atom_realtime.cpp`

Port first:

- radial sampling
- theta sampling
- phi sampling
- spherical-to-cartesian conversion
- particle generation
- color mapping

Do not prioritize yet:

- raytracer
- full UI polish
- 2D mode

### Phase 3: Controls and UI

Status: in progress

Goal:

- add controls for:
  - `n`
  - `l`
  - `m`
  - particle count
- show current values on screen
- support regenerate-on-change behavior
- keep UI compatible with browser/WASM delivery

### Phase 4: Probability-Flow Animation

Status: in progress

Goal:

- animate particles using the probability-flow logic
- allow pause and resume
- allow regenerate vs animate behavior

Core C++ reference:

- `calculateProbabilityFlow(...)` in `Atoms/src/atom_realtime.cpp`

### Phase 5: 2D Bohr Mode

Status: deferred

Goal:

- rebuild the simpler 2D atom demo
- keep it as another mode inside the same Rust project

Core C++ reference:

- `Atoms/src/atom.cpp`

### Phase 6: Raytraced Mode

Status: not started

Goal:

- add a high-quality presentation mode

Open decision:

- true raytraced mode
- or a visually similar advanced render mode that works better on the web

Core C++ reference:

- `Atoms/src/atom_raytracer.cpp`

## Beginner Rust Learning Order

Rust concepts to learn only as needed:

1. `fn`, `let`, `mut`
2. `struct`
3. `impl`
4. `Vec<T>`
5. references with `&`
6. `Option<T>` and `Result<T, E>`
7. modules and files
8. enums
9. ownership and borrowing in small examples

We do not need advanced Rust before starting real features.
We should learn only the next piece needed for the next milestone.

## Near-Term Priority

The next implementation focus is still Phase 2 and `atom_realtime.cpp` parity, but with the browser target in mind:

- confirm or choose the rendering stack
- make sure the stack supports WASM well
- create a real app entry point
- port orbital point generation from the C++ realtime mode first
- get a minimal cloud visible before adding many controls

## Current Status Snapshot

The Rust app is no longer a placeholder scene.
It now has a functioning realtime 3D orbital viewer with generic orbital sampling.

Current implemented pieces:

- generic hydrogen-orbital sampling driven by valid `n`, `l`, and `m`
- radial CDF sampling
- theta CDF sampling
- uniform `phi` sampling
- spherical-to-cartesian conversion
- first-pass probability-flow animation ported from `calculateProbabilityFlow(...)`
- fixed-step probability-flow updates for more stable parity tuning
- pause and resume flow animation
- live flow-speed tuning controls
- explicit regenerate-vs-animate controls
- combined cloud-mesh rendering instead of one entity per particle
- on-screen HUD
- realtime-style keyboard controls for `n`, `l`, `m`, and particle count
- mouse orbit camera and mouse-wheel zoom
- per-particle color derived from orbital intensity
- optional display mode toggle:
  - spherical density
  - real orbital basis for `p` and `d` orbitals

Current source responsibilities:

- `src/app.rs`
  - top-level app setup
  - parity-oriented input handling
- `src/render/mod.rs`
  - scene setup
  - combined cloud-mesh building/regeneration
  - per-frame particle animation
  - HUD
  - orbit camera controls
- `src/physics/mod.rs`
  - generic orbital sampling
  - display-mode branching
  - orbital intensity coloring
  - probability-flow velocity/update logic

## Realtime Parity Notes

The active parity target is `../Atoms/src/atom_realtime.cpp`.

Areas that now match reasonably well in structure:

- generic `n/l/m` orbital generation
- quantum-number clamping
- radial/theta/phi sampling pipeline
- realtime controls concept
- orbit-style camera interaction

Important remaining gaps versus `atom_realtime.cpp`:

- probability-flow animation is ported, but the motion feel may still need tuning against the C++ app
- color mapping is ported in spirit, but may still need visual tuning against the C++ output
- camera feel and visual scale may still need tuning against the C++ app
- we have removed one-entity-per-particle rendering, but still need real-world performance testing and likely more optimization for large particle counts

## Display Mode Decision

Two display modes currently exist in Rust:

- `SphericalDensity`
  - this is the closer match to the `Atoms` probability-density interpretation
- `RealOrbitalBasis`
  - this is a display-friendly mode for recognizable `p` and `d` shapes

For strict `Atoms` parity work:

- keep `SphericalDensity` as the default and reference mode
- treat `RealOrbitalBasis` as an optional visualization aid, not the parity baseline

## Next Recommended Step

The next implementation target should still stay inside realtime parity work, but shift from "add motion" to "tune and scale motion":

- compare flow speed, orbit size, and camera feel directly against `../Atoms/src/atom_realtime.cpp`
- tune the Bevy-side probability-flow speed so browser and native builds feel closer to the C++ reference
  - the app now has live speed controls, so this can be done interactively before editing constants
- replace one-entity-per-particle rendering with a more scalable approach once parity tuning confirms the desired behavior

Why this is next:

- the major missing behavior gap is no longer static-vs-animated particles
- the most obvious remaining parity risks are visual feel and particle-count scalability
- performance improvements will matter even more before pushing further toward browser-first WASM delivery

## New Chat Handoff

If a future session starts fresh, assume all of the following are already true:

- generic orbital sampling is implemented in Rust
- first-pass probability-flow animation is implemented in Rust
- the current target is `atom_realtime.cpp`, not the 2D mode
- the raytracer is postponed until after realtime parity is stronger
- the next best step is parity tuning plus more scalable particle rendering
- the first files to inspect for next work are `src/render/mod.rs` and `src/physics/mod.rs`

## Session Assumptions

Questions already answered:

### Do we want the full functionality of the C++ app?

Yes.

### Is this repo the main Rust home going forward?

Yes.

### Is the target just a loose inspiration from `Atoms`?

No.
It should become the Rust version of `Atoms`.

### Is WASM a first-class target?

Yes.

### Should explanations assume Rust experience?

No.
Assume beginner level in Rust.
