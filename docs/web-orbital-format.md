# Web Orbital JSON Format

This document defines the Part A export contract between Rust and the React app.

## Purpose

The Rust app does not need to run in the browser for Part A.

Instead, Rust exports static orbital datasets as JSON files, and the React app:

- loads one file for the selected `n/l/m`
- renders the particle cloud in the browser
- uses precomputed `omegas` for lightweight client-side animation

## File Naming

Each file is written as:

- `n{n}_l{l}_m{m}.json`

Example:

- `n4_l3_m1.json`

## JSON Shape

```json
{
  "n": 4,
  "l": 3,
  "m": 1,
  "particle_count": 100000,
  "display_mode": "spherical density",
  "rendered_cloud_label": "4f (m=1)",
  "positions": [x0, y0, z0, x1, y1, z1],
  "colors": [r0, g0, b0, r1, g1, b1],
  "omegas": [omega0, omega1]
}
```

## Field Meanings

- `n`, `l`, `m`
  - the orbital quantum numbers used to generate the dataset
- `particle_count`
  - number of particles in the exported cloud
- `display_mode`
  - Rust display mode label used during generation
- `rendered_cloud_label`
  - human-readable label for UI/debug use
- `positions`
  - flat XYZ float array
  - length is `particle_count * 3`
- `colors`
  - flat RGB float array in srgb space
  - length is `particle_count * 3`
- `omegas`
  - per-particle angular velocity around the Y axis
  - length is `particle_count`

## Export Command

Default usage:

```bash
cargo run --bin export_orbitals -- generated/orbitals
```

Optional arguments:

```bash
cargo run --bin export_orbitals -- <output_dir> <max_n> <particle_count> <display_mode>
```

Examples:

```bash
cargo run --bin export_orbitals -- generated/orbitals 4 100000 spherical-density
cargo run --bin export_orbitals -- generated/orbitals 3 50000 real-orbital-basis
```

## React Expectations

For Part A, the React app can treat these files as the full orbital source of truth.

The React side does not need Rust at runtime for this mode.
