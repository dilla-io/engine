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
//! **Dilla WASM builder with Extism.**
use extism_pdk::*;

#[cfg(feature = "prettify")]
use html_minifier::minify;

use dilla_renderer::{render as dilla_render, DESIGN_SYSTEM};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(feature = "describer")]
use dilla_describer::describe as dilla_describe;

#[plugin_fn]
pub fn version() -> FnResult<String> {
    Ok(format!(
        "Dilla Component v{VERSION} | ds: {}",
        DESIGN_SYSTEM
    ))
}

#[plugin_fn]
pub fn render_html(payload: String) -> FnResult<String> {
    let result = dilla_render(&payload, "full").unwrap();

    #[cfg(feature = "prettify")]
    return Ok(minify(result.clone()).expect("Failed to minify string"));

    #[cfg(not(feature = "prettify"))]
    Ok(result)
}

#[plugin_fn]
pub fn render(payload: String) -> FnResult<String> {
    let result = dilla_render(&payload, "json").unwrap();
    Ok(result)
}

#[plugin_fn]
#[cfg(feature = "describer")]
pub fn describe(req: String) -> FnResult<String> {
    let parts: Vec<&str> = req.split("::").collect();
    match parts.len() {
        0 => Ok(dilla_describe("", "")),
        1 => Ok(dilla_describe(parts[0], "")),
        _ => Ok(dilla_describe(parts[0], parts[1])),
    }
}
