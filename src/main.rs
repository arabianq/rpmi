mod dnf;
mod gui;
mod utils;

use std::{env, error::Error, fs::canonicalize, path::Path, process::Command};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        Err("Provide at least one package".into())
    } else if args.len() == 2 {
        let arg = &args[1];
        if let Ok(path) = canonicalize(Path::new(arg)) {
            gui::run(path)
        } else {
            Err("Failed to open {arg}".into())
        }
    } else {
        let binary_path = canonicalize(Path::new(&args[0]))?;

        for arg in args.iter().skip(1) {
            if let Ok(pkg_path) = canonicalize(Path::new(arg)) {
                Command::new(&binary_path).arg(pkg_path).spawn().ok();
            } else {
                eprintln!("Failed to open {arg}");
            }
        }

        Ok(())
    }
}
