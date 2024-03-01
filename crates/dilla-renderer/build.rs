#[path = "src/build/ds.rs"]
mod ds;

#[path = "src/build/test.rs"]
mod test;

use const_gen::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::{env, fs};
use walkdir::WalkDir;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/build/ds.rs");
    println!("cargo:rerun-if-env-changed=DS");

    ds::config();

    let design_system = env::var("DS").unwrap_or_else(|_| "test".to_string());

    let tpl_dir = dotenv::var("DILLA_TPL_DIR").unwrap_or_else(|_| "../../var/run/".to_string());
    let tpl_sub_dir =
        dotenv::var("DILLA_TPL_SUB_DIR").unwrap_or_else(|_| "/components".to_string());

    let templates_path = if design_system == "test" {
        test::config();
        let tpl_dir = "./tests";
        format!("{}{}", tpl_dir, tpl_sub_dir)
    } else {
        format!("{}{}{}", tpl_dir, design_system, tpl_sub_dir)
    };

    println!("[DEBUG] design_system: {}", design_system);
    println!("[DEBUG] templates_path: {}", templates_path);

    // dotenv::vars().for_each(|(key, value)| {
    env::vars().for_each(|(key, value)| {
        if key.starts_with("DILLA_") {
            println!("cargo:rustc-env={}={}", key, value);
        }
    });

    build_templates(templates_path);
}

#[derive(CompileConst, Default)]
struct SystemConfig {
    pub design_system: &'static str,
    pub components_library_dependencies: HashMap<&'static str, Vec<&'static str>>,

    pub components_library_css_html: HashMap<&'static str, &'static str>,

    // Used for library js url on json output.
    pub components_library_js:
        HashMap<&'static str, Vec<(&'static str, HashMap<&'static str, &'static str>)>>,

    pub components_variant_template: HashMap<&'static str, Vec<&'static str>>,
    pub components_with_library: Vec<&'static str>,

    pub default_libraries_css_html: &'static str,

    // Used for library js url on json output.
    pub default_libraries_js: Vec<(&'static str, HashMap<&'static str, &'static str>)>,

    pub libraries_css_html: HashMap<&'static str, &'static str>,

    // Used for library js url on json output.
    pub libraries_js:
        HashMap<&'static str, Vec<(&'static str, HashMap<&'static str, &'static str>)>>,
    pub libraries_keys: Vec<&'static str>,

    pub themes: HashMap<&'static str, HashMap<&'static str, &'static str>>,
    #[allow(dead_code)]
    pub styles: Vec<&'static str>,
    pub variables: HashMap<&'static str, &'static str>,
}

// Helper to wrap the config build.
fn build_config(config: SystemConfig) {
    let out_dir: std::ffi::OsString = env::var_os("OUT_DIR").unwrap();
    let dest_path: std::path::PathBuf = Path::new(&out_dir).join("codegen_config.rs");

    let contents: String = [
        // @todo should merge or remove to use DS?
        const_declaration!(#[doc = "The current Design System name."] pub DESIGN_SYSTEM = config.design_system),
        // const_definition!(#[doc = "Support the whole Design system configuration."] #[derive(Debug)] pub(crate) SystemConfig),
        const_definition!(#[doc = "Support the whole Design system configuration."] #[allow(dead_code)] pub(crate) SystemConfig),
        const_declaration!(#[doc = "The current Design System configuration."] pub(crate) DEFINITION = config),
    ]
    .join("\n");

    fs::write(dest_path, contents).unwrap();
}

// Example:
// static TEMPLATES: phf::Map<&'static str, &str> = ::phf::Map {
//     key: 12913932095322966823,
//     disps: &[(0, 0)],
//     entries: &[
//         ("badge", include_str!("../../../run/components/badge/badge.jinja")),
//         ("alert", include_str!("../../../run/components/alert/alert.jinja")),
//     ],
// };
fn build_templates(templates_path: String) {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen_templates.rs");
    let mut file = BufWriter::new(File::create(path).unwrap());
    let map = get_templates_paths(templates_path);
    writeln!(
        &mut file,
        "#[allow(dead_code)]\nstatic TEMPLATES: phf::Map<&'static str, &str> = {};",
        map.build()
    )
    .unwrap();
}

// Generate the template paths before compiling.
fn get_templates_paths(templates_path: String) -> phf_codegen::Map<String> {
    let mut templates = phf_codegen::Map::new();
    let root = env::var("CARGO_MANIFEST_DIR").unwrap();

    let walk_path = WalkDir::new(templates_path);

    for entry in walk_path
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if path.is_dir() {
            continue;
        }

        if path.extension().unwrap() != "jinja" {
            continue;
        }

        let include_path = path.display();
        let name = path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .replace(".jinja", "");

        templates.entry(
            name,
            format!("include_str!(\"{root}/{include_path}\")").as_str(),
        );
    }
    templates
}
