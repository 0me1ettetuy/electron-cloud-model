use std::{env, path::PathBuf, process};

use electron_cloud_model::{
    export,
    physics::{DisplayMode, OrbitalParams},
};

fn main() {
    if let Err(error) = run() {
        eprintln!("export failed: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let output_dir = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("generated/orbitals"));
    let max_n = args
        .next()
        .map(|value| value.parse::<u32>().map_err(|err| err.to_string()))
        .transpose()?
        .unwrap_or(4);
    let particle_count = args
        .next()
        .map(|value| value.parse::<usize>().map_err(|err| err.to_string()))
        .transpose()?
        .unwrap_or(100_000);
    let display_mode = parse_display_mode(args.next().as_deref())?;

    for n in 1..=max_n {
        for l in 0..n {
            for m in -(l as i32)..=(l as i32) {
                let mut params = OrbitalParams {
                    n,
                    l,
                    m,
                    particle_count,
                };
                params.normalize_quantum_numbers();

                let data = export::build_web_orbital_data(&params, display_mode);
                let path = export::write_web_orbital_data(&output_dir, &data)
                    .map_err(|err| err.to_string())?;
                println!(
                    "wrote {} ({}, {} particles)",
                    path.display(),
                    data.rendered_cloud_label,
                    data.particle_count
                );
            }
        }
    }

    Ok(())
}

fn parse_display_mode(input: Option<&str>) -> Result<DisplayMode, String> {
    match input.unwrap_or("spherical-density") {
        "spherical-density" | "spherical" => Ok(DisplayMode::SphericalDensity),
        "real-orbital-basis" | "real-basis" | "real" => Ok(DisplayMode::RealOrbitalBasis),
        other => Err(format!(
            "unknown display mode '{other}', expected 'spherical-density' or 'real-orbital-basis'"
        )),
    }
}
