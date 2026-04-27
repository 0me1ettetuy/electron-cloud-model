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
- 2D Bohr-style teaching/demo mode
- raytraced or raytrace-like presentation mode
- controls for quantum numbers and particle counts
- browser/WASM build as a first-class target

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

Status: started

Goal:

- keep this repo as the single Rust home
- add a written roadmap and standing context
- create a clean source layout

Deliverables:

- `PLAN.md`
- base source folders
- compilable placeholder app

### Phase 2: Realtime Orbital Viewer MVP

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

Goal:

- animate particles using the probability-flow logic
- allow pause and resume
- allow regenerate vs animate behavior

Core C++ reference:

- `calculateProbabilityFlow(...)` in `Atoms/src/atom_realtime.cpp`

### Phase 5: 2D Bohr Mode

Goal:

- rebuild the simpler 2D atom demo
- keep it as another mode inside the same Rust project

Core C++ reference:

- `Atoms/src/atom.cpp`

### Phase 6: Raytraced Mode

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

The next implementation focus is still Phase 2, but with the browser target in mind:

- confirm or choose the rendering stack
- make sure the stack supports WASM well
- create a real app entry point
- port orbital point generation from the C++ realtime mode first
- get a minimal cloud visible before adding many controls

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
