//! Handle *Bubabble* properties as **@attached**, **@variables**, **@library**,
//! and **@trans**.
//!
//! Bubabble properties are collected from anywhere in the payload and
//! bubbled to the top. Duplicates are not checked and replaced when reading
//! the payload.  
//! Bubbles are attached to specific fields and transformed for some of
//! them.

use crate::{
    renderable::{Html, Renderable, KEY_COMPONENT, KEY_COMPONENT_VARIANT, SEP_COMPONENT_VARIANT},
    DEFINITION, KEY_PREFIX,
};
use indexmap::IndexMap;
use serde_json::{Map, Value};
use std::collections::HashMap;

const KEY_ATTACHED: &str = "attached";
const KEY_LIBRARY: &str = "library";
const KEY_TRANS: &str = "trans";
const KEY_VARIABLES: &str = "variables";

/// Support bubbable renderable with specific fields and process.
#[derive(Debug, Default, Clone)]
pub struct Bubbable {
    component: Vec<String>,
    /// Render **@attached** for HTML `head`.
    pub attached_build: String,
    /// Manage libraries defined by the component, the payload and default.
    pub library: Library,
    /// Render **@variables** for HTML `style`.
    pub style: String,
    /// Collect translation to be used in templates with filter `|t`.
    pub translation: HashMap<String, String>,
    variables: HashMap<String, HashMap<String, Value>>,
}

impl Bubbable {
    pub fn new() -> Self {
        Self {
            component: Vec::new(),
            attached_build: String::new(),
            library: Library::new(),
            style: String::new(),
            translation: HashMap::new(),
            variables: HashMap::new(),
        }
    }

    fn add_library_css(&mut self, css: String) {
        self.library.add_css(css);
    }

    fn add_library_js(&mut self, key: String, value: Value) {
        self.library.add_js(key, value);
    }

    /// Render `@variables` bubbable and attach to the 'style' field.
    ///
    /// This iterates through the `variables` field, which contains scoped variable
    /// definitions. For each scope, it starts a CSS scope block. Then it iterates
    /// through the variable definitions, checking if the variable name exists in the
    /// global `DEFINITION.variables` config. If so, it renders the variable and value
    /// as CSS custom properties and adds them to the `style` field.
    ///
    /// The end result is that any `@variables` defined in the input data are rendered
    /// to CSS custom properties and included in the output.
    ///
    pub fn render_variables(&mut self) {
        let config_variables = DEFINITION.variables;
        // Render variables as styles.
        for (scope, obj_variables) in self.variables.iter() {
            self.style.push_str(&format!("{} {{\n", scope));
            for (name, value) in obj_variables {
                if !value.is_string() {
                    continue;
                }
                // @todo [devtools] log ignored key variable because not in definition
                // @todo [devtools] check default value and if same do not change
                if config_variables.contains_key(name) {
                    self.style.push_str(&format!(
                        "  --{}: {};\n",
                        name,
                        value.as_str().unwrap().replace('\"', "")
                    ));
                }
            }
            self.style.push_str("}\n");
        }
    }

    /// Collect bubabble properties from a serde_json::Value.
    ///
    /// This recursively traverses the provided `data` value to find and collect any
    /// bubbable properties like components, variables, etc. It checks for specific
    /// property keys starting with the `KEY_PREFIX` and calls handler methods for
    /// each bubbable type.
    ///
    /// For `Object` values, it iterates through each key/value pair and checks the
    /// key before recurring or calling a handler.
    ///
    /// For `Array` values, it simply recurs on each element.
    ///
    /// # Arguments
    ///
    /// * `data` - The serde_json `Value` to collect bubbable from.
    ///
    pub fn collect(&mut self, data: &Value) {
        let key_component = KEY_PREFIX.to_string() + KEY_COMPONENT;
        let key_attached = KEY_PREFIX.to_string() + KEY_ATTACHED;
        let key_variables = KEY_PREFIX.to_string() + KEY_VARIABLES;
        let key_library = KEY_PREFIX.to_string() + KEY_LIBRARY;
        let key_trans = KEY_PREFIX.to_string() + KEY_TRANS;

        match data {
            Value::Object(map) => {
                for (key, value) in map {
                    match key.as_str() {
                        key if key == key_component => self.handle_component(value, map),
                        key if key == key_attached => self.handle_attached(value),
                        key if key == key_variables => self.handle_variables(value),
                        key if key == key_library => self.handle_library(value),
                        key if key == key_trans => self.handle_trans(value),
                        _ => self.collect(value),
                    }
                }
            }
            Value::Array(array) => {
                for value in array.iter() {
                    self.collect(value);
                }
            }
            _ => {}
        }
    }

    fn handle_component(&mut self, value: &Value, map: &Map<String, Value>) {
        if DEFINITION.components_with_library.is_empty() {
            return;
        }

        // Attach library to component and component variants.
        if let Value::String(component_name) = value {
            let mut component_name_variant = String::new();
            if DEFINITION
                .components_variant_template
                .contains_key(component_name)
            {
                component_name_variant = self.get_component_name_from_data(component_name, map);
            }
            for name in [component_name, &component_name_variant] {
                // self.component is used for duplicate check.
                if !self.component.contains(name)
                    && DEFINITION.components_with_library.contains(&name.as_str())
                {
                    // Check for dependencies and add it to our library list for build.
                    // Order is important as js dependencies must be loaded before!
                    self.build_component_library_dependencies(name.as_str());
                    self.build_component_library_component(name.as_str());
                    self.component.push(name.clone());
                }
            }
        }
    }

    // @todo: optimize to avoid duplicate code for renderable.
    fn handle_attached(&mut self, value: &Value) {
        if let Value::Array(arr) = value {
            for obj in arr.iter() {
                if !obj.is_object() {
                    continue;
                }

                let mut renderable = Renderable::new(obj.as_object().unwrap().to_owned());
                renderable.build();
                self.attached_build.push_str(&renderable.to_html_string());
                self.attached_build.push('\n');
            }
        } else if let Value::Object(obj) = value {
            let mut renderable = Renderable::new(obj.to_owned());
            renderable.build();
            self.attached_build.push_str(&renderable.to_html_string());
            self.attached_build.push('\n');
        }
    }

    fn handle_variables(&mut self, value: &Value) {
        if let Value::Object(obj) = value {
            for (key, value) in obj.iter() {
                if !self.variables.contains_key(key) {
                    self.variables.insert(key.to_owned(), HashMap::new());
                }
                if let Some(group) = self.variables.get_mut(key) {
                    if let Some(inner_map) = value.as_object() {
                        for (inner_key, inner_value) in inner_map.iter() {
                            group.insert(inner_key.to_owned(), inner_value.to_owned());
                        }
                    }
                }
            }
        }
    }

    fn handle_library(&mut self, value: &Value) {
        if let Value::Object(obj) = value {
            for (asset_type, asset_data) in obj.iter() {
                if !asset_data.is_object() && !asset_data.is_array() {
                    continue;
                }
                if asset_type == "dependencies" {
                    for name in asset_data.as_array().unwrap_or(&vec![]) {
                        self.build_library_dependencies(name.as_str().unwrap_or_default());
                    }
                } else if asset_type == "css" {
                    for (css_url, attributes) in
                        asset_data.as_object().unwrap_or(&serde_json::Map::new())
                    {
                        let rendered_css = Renderable::link(css_url, attributes).to_html_string();
                        if !self.library.css.contains(&rendered_css) {
                            self.add_library_css(rendered_css);
                        }
                    }
                } else if asset_type == "js" {
                    for (js_url, attributes) in
                        asset_data.as_object().unwrap_or(&serde_json::Map::new())
                    {
                        self.add_library_js(js_url.to_owned(), attributes.to_owned());
                    }
                }
            }
        }
    }

    fn handle_trans(&mut self, value: &Value) {
        if let Value::Object(obj) = value {
            for (term, translation) in obj.iter() {
                if !self.translation.contains_key(term) {
                    self.translation
                        .insert(term.to_owned(), translation.to_string());
                }
            }
        }
    }

    fn build_library_dependencies(&mut self, name: &str) {
        if !DEFINITION.libraries_keys.contains(&name) {
            return;
        }

        let libs_css = DEFINITION.libraries_css_html;
        if libs_css.contains_key(name) {
            // @todo [devtools] alert missing dependency css
            let lib_to_add = libs_css.get(name).unwrap_or(&"");
            if !self.library.css.contains(&lib_to_add.to_string()) {
                self.add_library_css(lib_to_add.to_string());
            }
        }

        let libs_js = DEFINITION.libraries_js;
        if libs_js.contains_key(name) {
            let vec_map_lib = libs_js.get(name).unwrap();

            for (url, phf_attributes) in vec_map_lib.iter() {
                let mut attributes = Map::new();

                for (key, value) in phf_attributes {
                    attributes.insert(key.to_string(), Value::String(value.to_string()));
                }

                self.add_library_js(url.to_string(), Value::Object(attributes));
            }
        }
    }

    fn build_component_library_component(&mut self, value: &str) {
        if DEFINITION.components_library_css_html.contains_key(value) {
            let lib_to_add = DEFINITION.components_library_css_html.get(value).unwrap();

            self.add_library_css(lib_to_add.to_string());
        }

        if DEFINITION.components_library_js.contains_key(value) {
            let vec_map_lib: &&[(&str, phf::Map<&str, &str>)] =
                DEFINITION.components_library_js.get(value).unwrap();

            for (url, phf_attributes) in vec_map_lib.iter() {
                let json_map: serde_json::Map<String, Value> = phf_attributes
                    .entries()
                    .map(|(key, value)| (key.to_string(), Value::String(value.to_string())))
                    .collect();

                let attributes = serde_json::Value::Object(json_map);

                self.add_library_js(url.to_string(), attributes);
            }
        }
    }

    fn build_component_library_dependencies(&mut self, value: &str) {
        if !DEFINITION
            .components_library_dependencies
            .contains_key(value)
        {
            return;
        }

        let available_libraries = DEFINITION.libraries_keys;
        let dependencies = DEFINITION
            .components_library_dependencies
            .get(value)
            .unwrap();
        for dependency in dependencies.iter() {
            if !available_libraries.contains(dependency) {
                continue;
            }
            if DEFINITION
                .libraries_css_html
                .contains_key(dependency.as_ref())
            {
                // @todo [devtools] alert missing dependency css
                let lib_to_add = DEFINITION.libraries_css_html.get(dependency).unwrap();
                if !self.library.css.contains(&lib_to_add.to_string()) {
                    self.add_library_css(lib_to_add.to_string());
                }
            }
            if DEFINITION.libraries_js.contains_key(dependency.as_ref()) {
                let vec_map_lib = DEFINITION.libraries_js.get(dependency).unwrap();
                for (url, phf_attributes) in vec_map_lib.iter() {
                    let json_map: serde_json::Map<String, Value> = phf_attributes
                        .entries()
                        .map(|(key, value)| (key.to_string(), Value::String(value.to_string())))
                        .collect();

                    let attributes = serde_json::Value::Object(json_map);

                    self.add_library_js(url.to_string(), attributes);
                }
            }
        }
    }

    /// Get component name from variant and data if defined in Design system definition `components_variant_template`.
    fn get_component_name_from_data(
        &mut self,
        name: &str,
        data: &Map<String, serde_json::Value>,
    ) -> String {
        let key_variant = format!("{}{}", KEY_PREFIX, KEY_COMPONENT_VARIANT);
        data.get(&key_variant)
            .and_then(serde_json::Value::as_str)
            .filter(|variant| !variant.is_empty())
            .and_then(|variant| {
                DEFINITION
                    .components_variant_template
                    .get(name)
                    .filter(|variants| variants.contains(&variant))
                    .map(|_| format!("{}{}{}", name, SEP_COMPONENT_VARIANT, variant))
            })
            .unwrap_or_else(|| name.to_string())
    }
}

#[derive(Debug, Default, Clone)]
pub struct Library {
    pub css: Vec<String>,
    pub js: IndexMap<String, Value>,
}

impl Library {
    fn new() -> Self {
        Self {
            css: Vec::new(),
            js: IndexMap::new(),
        }
    }

    fn add_css(&mut self, css: String) {
        self.css.push(css);
    }

    fn add_js(&mut self, key: String, value: Value) {
        if !self.has_js(&key) {
            self.js.insert(key, value);
        }
    }

    fn has_js(&self, key: &str) -> bool {
        self.js.contains_key(key)
    }
}
