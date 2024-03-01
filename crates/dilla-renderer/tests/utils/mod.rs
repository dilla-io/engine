#[allow(unused)]
use assert_str::assert_str_trim_all_eq;
use dilla_renderer::render as dilla_render;
use similar_asserts::assert_eq;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use walkdir::WalkDir;

/// Integration tests for Dilla.
///
/// Check folder with json and html files to compare results.
pub fn test_loop(dir: &str, output: &str, suffix_expected: &str) {
    // eprintln!("[notice] Test {dir} :: {output}");
    let root = env::var("CARGO_MANIFEST_DIR").expect("Failed to get directory");
    let source_path = format!("{root}/tests/{dir}");
    // eprintln!("[notice] source_path: {source_path}");

    let path_exist = std::path::Path::new(&source_path).exists();
    // eprintln!("[notice] path_exist: {path_exist}");

    if !path_exist {
        println!("[Error] Test not found in `{dir}`");
        return;
    }

    for entry in WalkDir::new(source_path).into_iter().filter_map(|e| e.ok()) {
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

        if name.contains("_result") {
            continue;
        }

        test_generic_diff(&name, dir, output, suffix_expected);
        // eprintln!("[info] payload: {name}.json");
    }
}

pub fn test_json(dir: &str, name: &str, suffix_expected: &str) {
    // eprintln!("[notice] Test {dir} :: json");
    let data = load(name, dir, suffix_expected);
    if data.0.is_empty() {
        return;
    }

    let result = dilla_render(data.0.as_str(), "json");
    let result_json: serde_json::Value =
        serde_json::from_str(&result.ok().unwrap()).expect("file should be proper JSON");

    let root = env::var("CARGO_MANIFEST_DIR").expect("Failed to get directory");
    let filename = format!("{root}/tests/{dir}/{name}{suffix_expected}");
    let file = File::open(filename).expect("file should open read only");
    let expected_json: serde_json::Value =
        serde_json::from_reader(file).expect("file should be proper JSON");

    let result_javascript_src = result_json.get("javascript_src");
    let expected_javascript_src = expected_json.get("javascript_src");

    assert_eq!(result_javascript_src, expected_javascript_src);
}

fn test_generic_diff(name: &str, dir: &str, output: &str, suffix_expected: &str) {
    let data = load(name, dir, suffix_expected);
    if data.0.is_empty() {
        return;
    }

    let result = dilla_render(data.0.as_str(), output);

    // assert_str_trim_all_eq!(
    //     result.ok().unwrap().as_str(),
    //     data.1.as_str(),
    //     "\n\n\n[TEST] payload: {dir}/{name}.json\n\n\n"
    // );

    let result_trim = trim_whitespace(result.ok().unwrap().as_str());
    let expected = trim_whitespace(data.1.as_str());
    assert_eq!(
        // expected: data.1.as_str(),
        expected,
        // result: result.ok().unwrap().as_str(),
        result_trim,
        "\n\n\n[TEST] payload: {dir}/{name}.json\n\n\n"
    );
}

fn load(name: &str, dir: &str, suffix_expected: &str) -> (String, String) {
    let root = env::var("CARGO_MANIFEST_DIR").expect("Failed to get directory");
    let filename = format!("{root}/tests/{dir}/{name}{suffix_expected}");
    let html_exist = std::path::Path::new(&filename).exists();
    if !html_exist {
        println!("[SKIP TEST {name}] No file for {dir}/{name}{suffix_expected}");
        return ("".to_string(), "".to_string());
    }
    (
        String::from(&load_file(name, dir, ".json")),
        String::from(&load_file(name, dir, suffix_expected)),
    )
}

fn load_file(name: &str, dir: &str, suffix_expected: &str) -> String {
    let root = env::var("CARGO_MANIFEST_DIR").expect("Failed to get directory");
    let filename = format!("{root}/tests/{dir}/{name}{suffix_expected}");
    let mut contents = String::new();
    File::open(&filename)
        .unwrap_or_else(|_| panic!("File not found: {}", filename))
        .read_to_string(&mut contents)
        .unwrap_or_else(|_| panic!("Failed to read file: {}", filename));
    contents
}

// https://stackoverflow.com/questions/71864137/whats-the-ideal-way-to-trim-extra-spaces-from-a-string
#[allow(dead_code)]
fn trim_whitespace(s: &str) -> String {
    // let mut new_str = s.trim().to_owned();

    // // Some arbitrary replace.
    // new_str = new_str
    //     .replace("\n\n\n\n", "\n")
    //     .replace("\n\n\n", "\n")
    //     .replace("\n\n", "\n")
    //     .replace("  \n\n", "")
    //     .replace("  \n", "")
    //     .replace("\t", "");

    // let mut prev = ' '; // The initial value doesn't really matter
    // new_str.retain(|ch| {
    //     let result = ch != ' ' || prev != ' ';
    //     prev = ch;
    //     result
    // });

    // new_str

    let mut new_str: String = String::with_capacity(s.len()); // Avoid reallocations by preallocating memory

    // https://stackoverflow.com/questions/71864137/whats-the-ideal-way-to-trim-extra-spaces-from-a-string
    let mut prev: char = ' ';
    for ch in s.trim().chars() {
        if ch == '\n' && prev == '\n' {
            continue;
        }
        if ch == ' ' && (prev == ' ' || prev == '\n') {
            continue;
        }
        new_str.push(ch);
        prev = ch;
    }
    new_str = new_str
        .replace('\t', "")
        .replace("><link", ">\n<link")
        .replace("</script><script", "</script>\n<script");
    new_str
}
