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
//! **Dilla WASM builder with WASM Bindgen.**
use dilla_renderer::render_obj as dilla_render;
use gloo_utils::format::JsValueSerdeExt;
use serde_json::Value;
use wasm_bindgen::prelude::*;

#[cfg(feature = "describer")]
use dilla_describer::describe as dilla_describe;

#[cfg(feature = "debug")]
use dilla_renderer::DESIGN_SYSTEM;

#[cfg(feature = "debug")]
use instant::Instant;

#[cfg(feature = "debug")]
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[wasm_bindgen]
extern "C" {
    // #[wasm_bindgen(js_namespace = console)]
    // fn log(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn debug(s: &str);
}

/// Render with Dilla from a Payload as JavaScript Object.
///
/// # Arguments
///
/// * `payload` - The JavaScript value representing the payload to be rendered.
///
/// # Returns
///
/// The result of the rendering process as a JavaScript value.
///
/// # Errors
///
/// This function may encounter errors during the rendering process if the payload is not a valid JavaScript object or array.
/// If an error occurs, an error message will be printed and the corresponding error message will be returned as a `JsValue`.
///
/// # Panics
///
/// This function assumes that the provided payload can be successfully parsed as a `serde_json::Value` and rendered using Dilla.
/// It may panic if the payload cannot be parsed or if an unexpected error occurs during rendering.
#[wasm_bindgen]
pub fn render(payload: JsValue, silent: bool) -> JsValue {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    if !payload.is_object() && !payload.is_array() {
        let msg = "[Dilla] Payload is not valid, not a JS Object or Array.";
        error(msg);
        return JsValue::from_str(msg);
    }

    #[cfg(feature = "debug")]
    let now = Instant::now();

    let js_value: Value = payload.into_serde::<Value>().unwrap_or_default();

    // If payload is an object, replace it with an array containing that object.
    // Otherwise, use the original 'js_value' value.
    // @todo test this!
    let js_value: Value = if js_value.is_object() {
        Value::Array(vec![js_value])
    } else {
        js_value
    };

    if !js_value.is_array() {
        let msg = "[Dilla] JsValue is not valid, not an Array.";
        error(msg);
        return JsValue::from_str(msg);
    }

    let render = dilla_render(&js_value);
    let result = JsValue::from_serde(&render).ok().unwrap_or_default();

    #[cfg(feature = "debug")]
    debug(&print_time(now));

    if silent {
        return JsValue::from_str("");
    }
    result
}

/// Prints the description of artifacts or a single artifact if an ID is provided.
///
/// # Arguments
///
/// * `artefact` - An optional reference to a string representing the artifact.
/// * `id` - An optional reference to a string representing the ID.
///
/// # Errors
///
/// If an error occurs while parsing the JSON or executing the describe, an error message will be printed.
#[wasm_bindgen]
#[cfg(feature = "describer")]
pub fn describe(artefact: String, id: String) -> JsValue {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    let result = dilla_describe(&artefact, &id);
    JsValue::from_serde(&result).ok().unwrap_or_default()
}

/// Internal minimal test without parameters to be invoked, for example with wasmtime.
#[wasm_bindgen]
#[cfg(feature = "describer")]
pub fn test() -> String {
    dilla_describe("component", "test_component")
}

/// Helper to print information on version and time to render.
#[doc(hidden)]
#[cfg(feature = "describer")]
#[cfg(feature = "debug")]
fn print_time(now: Instant) -> String {
    let elapsed: f32 = now.elapsed().as_micros() as f32 / 1000.0;
    format!(
        "Dilla DEV Bindgen v{VERSION} | ds: {}, render: {:.2} ms",
        DESIGN_SYSTEM, elapsed
    )
}

/// Helper to print information on version and time to render.
#[doc(hidden)]
#[cfg(not(feature = "describer"))]
#[cfg(feature = "debug")]
fn print_time(now: Instant) -> String {
    let elapsed: f32 = now.elapsed().as_micros() as f32 / 1000.0;
    format!(
        "Dilla Bindgen v{VERSION} | ds: {}, render: {:.2} ms",
        DESIGN_SYSTEM, elapsed
    )
}
