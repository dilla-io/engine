//! <div align=center>
//!   <img src="https://data.dilla.io/dilla.png" alt="" width=320>
//!   <p><strong>Share your design system in a tiny universal package.</strong></p>
//! </div>
//!
//! Dilla is a fast but minimal WASM builder based on the syntax and behavior
//! of the [Jinja2](https://jinja.palletsprojects.com/) implemented on top of
//! [Minijinja](https://docs.rs/minijinja/latest/minijinja). The goal is to
//! be able to pack your design system into a <strong>universal</strong>
//! package, executable through a simple <strong>declarative API</strong>, for
//! both server side and headless rendering.
//!
//! To know more about Dilla visit our website [dilla.io](https://dilla.io).
//!
//! ---
//!
//! **Dilla Renderer, main engine to process a payload to an output.**

pub mod attribute;
pub mod bubbable;
pub mod engine;
pub mod renderable;
pub mod renderer;
pub mod scoped;

use renderer::{Output, Renderer};
use serde_json::Error;

/// Keywords prefix to identify properties in the payload.
pub(crate) const KEY_PREFIX: &str = "@";

include!(concat!(env!("OUT_DIR"), "/codegen_config.rs"));

/// Render a Json String value into HTML String and output in specific Dilla formats.
///
/// # Arguments
///
/// * `payload`: The JSON payload string to be rendered, following Dilla format API
/// * `output`: The output format desired, can be:
///   * `json`: Default. A json response with every rendered parts as:
///     * `attached`: The HTML `@attached` part of the rendered output for `<header>`
///     * `body`: The HTML main rendered content
///     * `system_stylesheet`: The HTML list of global `<link>` tags
///     * `system_javascript`: A list of global javascript files from libraries as `name => {options}`.
///     * `stylesheet`: The HTML list of `<link>` tags
///     * `javascript`: A list of javascript files from libraries as `name => {options}`.
///     * `variables`: The HTML `@variables` wrapped in a `<style>` tag
///   * `full`: Whole HTML page wrapped in `<html>` tag
///   * `_test`: Test is only HTML body without head, styles and libraries, for test purpose
///   * `_test_full`: All HTML parts not wrapped in `<html>` tag, for test purpose
///   * `_logs`: Display on logs, mostly debug and internal test purpose
///
/// # Returns
///
/// * `Result<String, Error>` - A Result containing the rendered output as a String if successful, or an Error if any error occurs.
///
pub fn render(payload: &str, output: &str) -> Result<String, Error> {
    let json: serde_json::Value = serde_json::from_str::<serde_json::Value>(payload)?;
    let mut renderer: Renderer = Renderer::new();
    renderer.render(&json);

    Ok(renderer.to_output_string(output))
}

/// Render a Json String into Json String output. Shortcut for render(payload, "json").
///
/// # Arguments
///
/// * `payload`: The JSON payload string to be rendered, following Dilla format API
///
/// # Returns
///
/// * `Result<String, Error>` - A Result containing the rendered JSON output as a String if successful, or an Error if any error occurs.
///
pub fn render_string(payload: String) -> Result<String, Error> {
    let json: serde_json::Value = serde_json::from_str::<serde_json::Value>(&payload)?;

    let mut renderer: Renderer = Renderer::new();
    renderer.render(&json);

    Ok(renderer.to_output_string("json"))
}

/// Render a Json Object into json Object.
///
/// # Arguments
///
/// * `json`: The Value be rendered, following Dilla format API
///
/// # Returns
///
/// A `serde_json::Value` representing the rendered output.
///
pub fn render_obj(json: &serde_json::Value) -> serde_json::Value {
    let mut renderer: Renderer = Renderer::new();
    renderer.render(json);

    renderer.to_output()
}

#[doc(hidden)]
pub fn _print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}
