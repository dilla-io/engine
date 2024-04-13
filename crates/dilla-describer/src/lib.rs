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
//! **Dilla describer, providing introspection.**
use serde_json::{json, Value};

include!(concat!(env!("OUT_DIR"), "/codegen_definitions.rs"));

/// Prints the description of artifacts or a single artifact if an ID is provided.
///
/// # Arguments
///
/// * `artefact` - An optional reference to a string representing the artifact.
/// * `id` - An optional reference to a string representing the ID.
///
/// # Example
///
/// ```rust
/// use serde_json::json;
/// use dilla_describer::describe;
///
/// let result = describe("tests", "test");
/// assert_eq!(result, "\"test\"");
/// let result = describe("tests", "");
/// assert_eq!(result, "{\n  \"test\": \"test\"\n}");
/// ```
///
/// # Errors
///
/// If an error occurs while parsing the JSON or executing the query, an error message will be printed.
///
/// # Panics
///
/// This function assumes that the JSON value defined by the `DEFINITIONS` constant is well-formed and can be successfully queried.
/// It may panic if the JSON is invalid.
pub fn describe(artefact: &str, id: &str) -> String {
    match serde_json::from_str::<Value>(DEFINITIONS) {
        Ok(json) => describe_inner(artefact, id, &json),
        Err(error) => {
            format!("Failed to parse JSON: {}", error)
        }
    }
}

/// Returns a formatted JSON string representing a specific value in a JSON structure.
///
/// # Arguments
///
/// * `artefact` - The key of the parent value in the JSON structure. If empty all json is returned.
/// * `id` - The key of the specific value within the parent value or '_list' to return children ids. If empty all artefact are returned.
/// * `json` - The JSON structure represented as a `serde_json::Value`.
fn describe_inner(artefact: &str, id: &str, json: &Value) -> String {
    let mut value = json;

    let mut mut_artefact = artefact.to_owned();
    if !mut_artefact.ends_with('s') {
        mut_artefact.push('s');
    }

    if id == "_list" {
        if let Some(parent) = json.get(&mut_artefact) {
            if let Some(keys) = parent.as_object().map(|obj| obj.keys().collect::<Vec<_>>()) {
                return serde_json::to_string_pretty(&keys).unwrap();
            }
        }
        return json!({"error":format!("Not found artefact: {}", mut_artefact)}).to_string();
    }

    if !artefact.is_empty() {
        if let Some(v) = json.get(&mut_artefact) {
            value = v;
        } else {
            return json!({"error":format!("Not found artefact: {}", mut_artefact)}).to_string();
        }
    }

    if !id.is_empty() && id != "_list" {
        if let Some(v) = value.get(id) {
            value = v;
        } else {
            return json!({"error":format!("Not found artefact id: {}::{}", mut_artefact, id)})
                .to_string();
        }
    }

    serde_json::to_string_pretty(value).unwrap()
}
