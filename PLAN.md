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
- controls for quantum numbers and core view actions
- browser/WASM build as a first-class target

Current focus clarification:

- the immediate goal is the `atom_realtime.cpp` experience from `Atoms`
- the 2D Bohr-style mode is currently not needed for the active workstream
- the raytracer may still be needed later, but it is not the current implementation target

## Live Web Reference

Related visual and UX reference:

- `https://www.kavang.com/atom`

This site is related to the `Atoms` project and is a useful browser target for:

- control layout and browser embedding expectations
- clipping controls
- color-scale controls
- overall composition and interaction feel

Important clarification:

- it is a strong product reference, especially for browser delivery
- it is not a literal runtime of the native C++ app
- when the web reference and `atom_realtime.cpp` differ, treat the C++ app as the physics/parity reference and the website as the browser UX reference

Current implementation note from inspection of the live site:

- the live site is a Next.js/React app
- it uses a Three.js-based canvas through React Three Fiber
- it fetches precomputed orbital JSON files such as `/orbitals/n4_l3_m1.json`
- clipping and color-scale behavior are applied in the frontend viewer

That means the live site is best treated as a browser product reference, not as proof that the orbital generation is already running live in WASM.

## Website Modes

For the browser product, plan around two web modes:

### Mode A: Precomputed Web Viewer

Goal:

- match the current `kavang.com/atom` product shape quickly
- use React UI plus a web renderer
- load precomputed orbital datasets by `n/l/m`
- support clipping and color-scale controls in the browser

Why keep this mode:

- it is the fastest path to a polished web experience
- it matches the live site's current implementation style
- it gives the React app a stable product target while Rust work continues

### Mode B: Rust Runtime Web Viewer

Goal:

- run the orbital generation and viewer logic from Rust
- target `wasm32-unknown-unknown`
- expose state and commands to a React host
- eventually reduce or replace dependence on precomputed orbital assets

Why keep this mode:

- it preserves the repo's long-term identity as the Rust rewrite of `Atoms`
- it keeps feature and behavior parity work inside the Rust codebase
- it is the better long-term architecture if Rust is meant to be the real runtime

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

### Phase 3: Browser Controls and Host API

Status: in progress

Goal:

- remove the temporary in-engine HUD from the target product path
- expose app state and commands so a React host can own the UI
- match the browser-facing control model more closely to `https://www.kavang.com/atom`
- keep native keyboard controls only as a local debugging fallback

Browser-facing controls to support:

- `n`
- `l`
- `m`
- regenerate
- display mode toggle
- camera reset
- orbit drag
- zoom

Required host integration deliverables:

- a serializable "HUD/app state" snapshot API for the host UI
- command APIs for changing orbital and render settings from JavaScript/React
- a clear boundary between:
  - Rust simulation and rendering
  - React controls and browser UI
- no dependency on Bevy UI for the shipped browser experience

### Phase 3A: Precomputed Web Mode

Status: not started

Goal:

- build a React-hosted browser viewer that follows the live site's shape
- load precomputed orbital datasets by `n/l/m`
- support the smaller browser control set for orbital changes and core view actions
- keep this mode usable even before Rust/WASM host integration is finished

Key deliverables:

- React project shell
- web renderer for precomputed particle data
- orbital JSON loading contract
- browser controls for:
  - `n`
  - `l`
  - `m`
  - regenerate
  - display mode toggle
  - camera reset
  - orbit drag
  - zoom

### Phase 3B: Rust/WASM Web Mode

Status: not started

Goal:

- compile the Rust viewer/runtime for the browser
- replace the temporary Bevy HUD with host-driven controls
- expose Rust state and command APIs to React
- allow the React app to switch between or compare the two web modes if desired

Key deliverables:

- confirmed `wasm32-unknown-unknown` build path
- host-readable state snapshot API
- host-to-Rust commands for orbital and render settings
- browser embedding strategy for the Rust canvas/runtime

### Phase 4: Probability-Flow Animation

Status: in progress

Goal:

- animate particles using the probability-flow logic
- keep the current fixed animation behavior stable
- allow regeneration without breaking the motion path

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

The next implementation focus is now browser-first delivery with two tracks:

- start the React web product using the precomputed-data mode
- define the data contract for orbital JSON files and browser controls
- keep the Rust app as the source of physics/parity work
- prepare a later Rust/WASM integration path without blocking the faster web mode
- avoid spending more time on temporary in-engine UI

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
- explicit regenerate control
- combined cloud-mesh rendering instead of one entity per particle
- temporary on-screen HUD for native debugging
- realtime-style keyboard controls for `n`, `l`, `m`
- mouse orbit camera and mouse-wheel zoom
- per-particle color derived from orbital intensity
- optional display mode toggle:
  - spherical density
  - real orbital basis for `p` and `d` orbitals

Current source responsibilities:

- `src/app.rs`
  - top-level app setup
  - temporary native input handling for local debugging
- `src/render/mod.rs`
  - scene setup
  - combined cloud-mesh building/regeneration
  - per-frame particle animation
  - temporary HUD
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

Important remaining gaps versus the browser target:

- no confirmed WASM build path is in place yet
- no React/JavaScript host API exists yet for reading app state or sending commands
- the current Bevy HUD is still present even though it should not be part of the shipped browser UI
- the current control scheme is still keyboard-first instead of browser UI-first
- no precomputed orbital export pipeline or web data contract has been defined yet
- no React project exists yet for either web mode

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

The next implementation target should be the React web project with explicit dual modes:

- Mode A:
  - build the faster precomputed-data web viewer first
  - use it to match the live site's browser UX quickly
- Mode B:
  - continue preparing Rust for eventual WASM host integration
  - do not block the web product on this path

Concrete next tasks:

- create the React project
- define the orbital JSON schema the web viewer will consume
- decide whether the Rust repo will export those JSON files directly or whether a separate exporter script will
- remove the Bevy HUD from the product plan and treat it as debug-only
- keep the Rust code focused on parity, data generation, and future WASM readiness

Why this is next:

- it gives the fastest path to a working website
- it matches the current live site's implementation style
- it still leaves room for Rust/WASM as a second web mode instead of forcing an early all-or-nothing choice
- it prevents the React project from waiting on unfinished Rust host APIs

## New Chat Handoff

If a future session starts fresh, assume all of the following are already true:

- generic orbital sampling is implemented in Rust
- first-pass probability-flow animation is implemented in Rust
- the current target is `atom_realtime.cpp`, not the 2D mode
- the raytracer is postponed until after realtime parity is stronger
- the current Bevy HUD is temporary and should be removed from the shipped browser experience
- the website should support two modes:
  - a faster precomputed-data web mode
  - a longer-term Rust/WASM web mode
- the next best step is to start the React project and define the data/API boundary
- the first files to inspect for next work are `src/app.rs`, `src/render/mod.rs`, and `src/physics/mod.rs`

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
