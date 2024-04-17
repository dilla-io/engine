//! **Dilla bin, minimal code to analyse dilla engine lib, bloats and time.**

use dilla_renderer::render as dilla_render;
use std::fs;
use std::path::PathBuf;

fn main() {
    let payload_file = PathBuf::from("./payload.json");

    let payload = fs::read_to_string(payload_file).expect("Failed to read payload file!");
    let result = dilla_render(&payload, "json").expect("Dilla rendering failed!");

    println!("{}", result);
}
