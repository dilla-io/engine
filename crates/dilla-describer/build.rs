use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=DS");

    let design_system = env::var("DS").unwrap_or_else(|_| "test".to_string());

    let tpl_dir = if design_system == "test" {
        "/tests".to_string()
    } else {
        env::var("DILLA_TPL_DIR").unwrap_or_else(|_| "../../var/run".to_string())
    };

    // if design_system == "test" {
    //     tpl_dir = "/tests".to_string();
    // }

    let root = env::var("CARGO_MANIFEST_DIR").unwrap();

    let definitions_path = format!("{}/{}/{}/definitions.json", root, tpl_dir, design_system);

    // println!("[DEBUG] design_system: {}", design_system);
    // println!("[DEBUG] tpl_dir: {}", tpl_dir);
    // println!("[DEBUG] definitions_path: {}", definitions_path);

    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen_definitions.rs");
    let mut file = BufWriter::new(File::create(path).unwrap());

    writeln!(
        &mut file,
        "#[allow(dead_code,clippy::redundant_static_lifetimes)]\nstatic DEFINITIONS: &'static str = {};",
        format!("include_str!(\"{definitions_path}\")").as_str(),
    )
    .unwrap();
}
