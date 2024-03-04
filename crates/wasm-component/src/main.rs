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
//! **Dilla WASM builder with WASM Component model.**
#[allow(warnings)]
mod bindings;

use bindings::Guest;

use dilla_renderer::render as dilla_render;
use std::env;
use std::fs;
use std::path::Path;

#[cfg(feature = "describer")]
use dilla_describer::describe as dilla_describe;

#[cfg(feature = "prettify")]
use html_minifier::minify;
#[cfg(feature = "prettify")]
use serde_json::Value;

use dilla_renderer::DESIGN_SYSTEM;
const VERSION: &str = env!("CARGO_PKG_VERSION");

struct Component;

/**
 * Specific implementation for wit-bindgen and bindings generation.
 */
impl Guest for Component {
    fn version() -> String {
        version()
    }

    fn render(payload: String) -> String {
        #[cfg(feature = "debug")]
        info("Guest");

        #[cfg(feature = "prettify")]
        let value: Value = serde_json::from_str(&result).unwrap();
        #[cfg(feature = "prettify")]
        return serde_json::to_string_pretty(&value).unwrap();

        #[cfg(not(feature = "prettify"))]
        dilla_render(&payload, "json").unwrap_or("Dilla engine::render error!".to_string())
    }

    fn render_html(payload: String) -> String {
        #[cfg(feature = "debug")]
        info("Guest");
        #[cfg(feature = "prettify")]
        return minify(result).expect("Failed to minify string");
        #[cfg(not(feature = "prettify"))]
        dilla_render(&payload, "full").unwrap_or("Dilla engine::render error!".to_string())
    }

    fn describe(req: String) -> String {
        #[cfg(feature = "debug")]
        info("Guest");
        main_describe(&req)
    }
}

bindings::export!(Component with_types_in bindings);

/**
 * Module will be used by Wasmtime or Node WASI runtime.
 */
fn main() {
    let help = r#"Dilla WASM Component requires two arguments.

    USAGE:
        wasmtime MY_WASM.wasm <function> <request>

    ARGUMENTS:
        <function> - can be one of the following:
                        render: render the payload with Dilla Engine and return a json response.
                        render_html: render the payload with Dilla Engine and return a text HTML response.
                     for a DEV build:
                        describe: return a JSON response from the Dilla Describe API.
        <request>  - request to the function:
                        'version': Get current version and Design System
                        'render'
                            - JSON payload file to load
                            - Optional flag to silence output (for benchmark)
                        'describe': Artefact and Id separated by '::', ie: `component::alert`
    EXAMPLES:
        wasmtime MY_WASM.wasm version
        wasmtime MY_WASM.wasm render ./payload.json
        wasmtime MY_WASM.wasm render ./payload.json true
        wasmtime MY_WASM.wasm render_html ./payload.json
        wasmtime MY_WASM.wasm describe component::alert
        wasmtime MY_WASM.wasm describe component::_list
    "#;

    let args: Vec<String> = env::args().collect();

    #[cfg(feature = "debug")]
    println!("[DEBUG] {:?}", args);

    match args.len() {
        2 => {
            let result = dispatch(args[1].to_owned(), "".to_string(), "");
            #[cfg(feature = "debug")]
            info("Main");
            println!("{}", result);
        }
        3 => {
            let result = dispatch(args[1].to_owned(), args[2].to_owned(), "");
            #[cfg(feature = "debug")]
            info("Main");
            println!("{}", result);
        }
        4 => {
            let result = dispatch(args[1].to_owned(), args[2].to_owned(), args[3].as_str());
            #[cfg(feature = "debug")]
            info("Main");
            println!("{}", result);
        }
        _ => print!("{}", help),
    }
}

fn dispatch(function: String, req: String, silent: &str) -> String {
    match function.as_str() {
        "version" => version(),
        "render" => main_render(&req, silent),
        "render_html" => main_render_html(&req),
        "describe" => main_describe(&req),
        _ => format!("Unknown function: {}", function),
    }
}

fn version() -> String {
    #[cfg(feature = "describer")]
    return format!(
        "[DEBUG] Dilla DEV Component v{VERSION} | ds: {}", DESIGN_SYSTEM
    );

    #[cfg(not(feature = "describer"))]
    format!(
        "[DEBUG] Dilla Component v{VERSION} | ds: {}", DESIGN_SYSTEM
    )
}

fn main_render(name: &str, silent: &str) -> String {
    let payload = get_payload(name);

    if silent.is_empty() {
        #[cfg(feature = "prettify")]
        let value: Value = serde_json::from_str(&result).unwrap();
        #[cfg(feature = "prettify")]
        return serde_json::to_string_pretty(&value).unwrap();

        #[cfg(not(feature = "prettify"))]
        dilla_render(&payload, "json").unwrap_or("Dilla engine::render error!".to_string())
    } else {
        dilla_render(&payload, "json").unwrap_or("Dilla engine::render error!".to_string());
        "".to_string()
    }
}

fn main_render_html(name: &str) -> String {
    let payload = get_payload(name);

    #[cfg(feature = "prettify")]
    return minify(result).expect("Failed to minify string");
    #[cfg(not(feature = "prettify"))]
    dilla_render(&payload, "full").unwrap_or("Dilla engine::render error!".to_string())
}

fn main_describe(req: &str) -> String {
    let parts: Vec<&str> = req.split("::").collect();
    match parts.len() {
        0 => dilla_describe("", ""),
        1 => dilla_describe(parts[0], ""),
        _ => dilla_describe(parts[0], parts[1]),
    }
}

#[cfg(not(feature = "describer"))]
fn dilla_describe(_: &str, _: &str) -> String {
    "Not a DEV build, `describe` is not implemented in this Dilla Engine WASM!".to_string()
}

fn get_payload(name: &str) -> String {
    if Path::new(name).exists() {
        #[cfg(feature = "debug")]
        println!("[DEBUG] Load file {}", name);
        fs::read_to_string(name).unwrap_or_else(|_| panic!("Error opening the file: {}", name))
    } else {
        #[cfg(feature = "debug")]
        println!("[DEBUG] Load payload");
        name.to_string()
    }
}

#[cfg(feature = "debug")]
fn info(scope: &str) {
    #[cfg(feature = "describer")]
    println!(
        "[DEBUG] Dilla DEV Component {} v{VERSION} | ds: {}",
        scope, DESIGN_SYSTEM
    );
    #[cfg(not(feature = "describer"))]
    println!(
        "[DEBUG] Dilla Component {} v{VERSION} | ds: {}",
        scope, DESIGN_SYSTEM
    );
}
