//! Handle *Scoped* properties as **@styles** and **@local_variables**.
//!
//! Scoped properties are limited to the component or element they are in.

use crate::{DEFINITION, KEY_PREFIX};
use serde_json::{Map, Value};

const KEY_STYLES: &str = "styles";
const KEY_THEME: &str = "theme";
const KEY_THEME_TARGET: &str = "target";
const KEY_THEME_KEY: &str = "key";
const KEY_THEME_VAL: &str = "val";
const KEY_LOCAL_VARIABLES: &str = "local_variables";

#[derive(Debug, Default)]
pub struct Scoped {
    pub styles: Vec<String>,
    pub theme_attribute: Vec<(String, Vec<String>)>,
    pub theme_class: Vec<String>,
    pub local_variables: Vec<String>,
}

impl Scoped {
    pub fn new() -> Self {
        Self {
            styles: Vec::new(),
            theme_attribute: Vec::new(),
            theme_class: Vec::new(),
            local_variables: Vec::new(),
        }
    }

    /// Collect scoped @styles, @local_variables and @theme from the provided data Map.
    ///
    /// # Arguments
    ///
    /// * `data` - A reference to a Map containing the data to collect scoped
    ///            variables and styles from.
    /// # Example
    ///
    /// ```rust
    /// use serde_json::{json, Map};
    /// use dilla_renderer::scoped::Scoped;
    ///
    /// let mut scoped = Scoped::new();
    ///
    /// let mut data = Map::new();
    /// data.insert(format!("@styles"), json!(["style-1", "style-2"]));
    /// data.insert(
    ///   format!("@local_variables"),
    ///   json!({"var-1": "#222", "var-2": "#333"}),
    /// );
    ///
    /// scoped.collect(&data);
    ///
    /// assert_eq!(scoped.styles, vec!["style-1", "style-2"]);
    /// assert_eq!(scoped.local_variables, vec!["--var-1: #222;", "--var-2: #333;"]);
    /// ```
    pub fn collect(&mut self, data: &Map<String, Value>) {
        self.collect_local_variables(data);
        self.collect_styles(data);
        self.collect_theme(data);
    }

    /// Collects scoped variables from the provided data map.
    ///
    /// This looks for a key formatted as "@local_variables" in the data map.
    /// If that key contains a JSON object, it iterates through the key-value pairs.
    /// For each variable key that is defined in the global DEFINITION.variables,
    /// it formats the variable and value as "--{key}: {value};" and adds it to the
    /// local_variables field on self.
    ///
    /// # Arguments
    ///
    /// * `data` - Reference to the data map to collect variables from
    fn collect_local_variables(&mut self, data: &Map<String, Value>) {
        let key_variables = format!("{KEY_PREFIX}{KEY_LOCAL_VARIABLES}");
        let data_variable = data.get(&key_variables);

        if let Some(Value::Object(variables)) = data_variable {
            // Get global defined variables
            let config_variables = &DEFINITION.variables;

            // Add valid variables to local_variables
            for (key, value) in variables.iter() {
                if config_variables.contains_key(key) {
                    self.local_variables
                        .push(format!("--{}: {};", key, value.as_str().unwrap()));
                }
            }
        }
    }

    /// Collects scoped styles from the provided data map.
    ///
    /// This looks for a key formatted as "{KEY_PREFIX}{KEY_STYLES}" in the data map.
    /// If that key contains a JSON array, it iterates through each style string.
    /// The style strings are added to the styles field on self.
    ///
    /// # Arguments
    ///
    /// * `data` - Reference to the data map to collect styles from
    fn collect_styles(&mut self, data: &Map<String, Value>) {
        let key_styles = format!("{KEY_PREFIX}{KEY_STYLES}");
        let data_styles = data.get(&key_styles);

        // If styles array found, iterate elements
        if let Some(Value::Array(styles)) = data_styles {
            // Add each style string to styles field
            for style in styles {
                if let Some(style_str) = style.as_str() {
                    self.styles.push(style_str.to_string());
                }
            }
        }
    }

    /// Collects scoped theme from the provided data map.
    ///
    /// This looks for a key formatted as "{KEY_PREFIX}{KEY_THEME}" in the data map.
    ///
    /// # Arguments
    ///
    /// * `data` - Reference to the data map to collect styles from
    fn collect_theme(&mut self, data: &Map<String, Value>) {
        let key_theme = format!("{}{}", KEY_PREFIX, KEY_THEME);

        if let Some(data_theme) = data.get(&key_theme).and_then(|v| v.as_str()) {
            if let Some(theme) = DEFINITION.themes.get(data_theme) {
                // Theme as body attribute
                let target = theme.get(KEY_THEME_TARGET).unwrap_or(&"").to_owned();
                if target == "html" {
                    // @todo implement target html
                    println!("<!-- @todo implement @theme: target = 'html' -->");
                    return;
                }

                // Theme as attribute or class
                let key: &str = theme.get(KEY_THEME_KEY).unwrap_or(&"").to_owned();
                let val: &str = theme.get(KEY_THEME_VAL).unwrap_or(&"").to_owned();
                match key {
                    "class" => self.theme_class.push(val.to_owned()),
                    _ => self
                        .theme_attribute
                        .push((key.to_owned(), vec![val.to_owned()])),
                }
            }
        }
        // @todo [devtools] log non valid theme
    }
}
