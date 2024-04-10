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

    let root_path = env::var("CARGO_MANIFEST_DIR").unwrap();
    let design_system = env::var("DS").unwrap_or_else(|_| "test".to_string());
    let tpl_dir = dotenv::var("DILLA_TPL_DIR").unwrap_or_else(|_| "../../var/run/".to_string());

    let ds_path = if design_system == "test" {
        test::config();
        let tpl_dir = "./tests";
        format!("{tpl_dir}")
    } else {
        format!("{tpl_dir}{design_system}")
    };

    if !std::path::Path::new(&ds_path).exists() {
        panic!("[Fatal] Not found Design System: {ds_path}, please check the Design system id!");
    }

    env::vars().for_each(|(key, value)| {
        if key.starts_with("DILLA_") {
            println!("cargo:rustc-env={}={}", key, value);
        }
    });

    build_templates(&ds_path, &root_path);

    if design_system != "test" {
        build_tests(&design_system, &ds_path, &root_path);
    }
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
fn build_templates(ds_path: &str, root_path: &str) {
    let templates_path = format!("{ds_path}/components");

    // println!("cargo::warning=[DEBUG] templates_path: {}", templates_path);

    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen_templates.rs");
    let mut file = BufWriter::new(File::create(path).unwrap());
    let map = get_templates_paths(templates_path, root_path);

    writeln!(
        &mut file,
        "#[allow(dead_code)]\nstatic TEMPLATES: phf::Map<&'static str, &str> = {};",
        map.build()
    )
    .unwrap();
}

// Generate the template paths before compiling.
fn get_templates_paths(templates_path: String, root_path: &str) -> phf_codegen::Map<String> {
    let mut templates = phf_codegen::Map::new();

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
            format!("include_str!(\"{root_path}/{include_path}\")").as_str(),
        );
    }
    templates
}

// build the tests code for this DS.
// @todo: list of missing tests?
fn build_tests(design_system: &str, ds_path: &str, root_path: &str) {
    let tests_path = format!("{root_path}/{ds_path}/tests");

    // println!("cargo::warning=[DEBUG] tests_path: {tests_path}");

    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen_tests.rs");
    let mut file = BufWriter::new(File::create(path).unwrap());

    let walk_path = fs::read_dir(&tests_path).unwrap();
    for entry in walk_path {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            continue;
        }

        if path.extension().unwrap() != "json" {
            continue;
        }

        let name = path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .replace(".json", "");

        // println!("cargo::warning=[DEBUG] build test: {name}");

        let payload_path = format!("{tests_path}/{name}.json");
        let result_path = format!("{tests_path}/{name}.html");

        // println!("cargo::warning=[DEBUG] {payload_path}");
        // println!("cargo::warning=[DEBUG] {result_path}");

        let fn_name = name.replace("_", "").replace("--", "_").replace("-", "_");
        let code = format!(
            r#"
    #[test]
    fn test_{design_system}_{fn_name} () {{
        let res = utils::test_ds_generic_diff(&"{payload_path}", &"{result_path}", "_test");
        assert_eq!(res.0, res.1);
    }}
    "#
        );
        writeln!(&mut file, "{code}").unwrap();
    }
}
