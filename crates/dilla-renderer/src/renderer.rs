//! Dilla renderer to output the result.

use crate::bubbable::Bubbable;
use crate::renderable::{Html, Renderable};
use crate::{engine, DEFINITION};

use indexmap::IndexMap;
use minijinja::{context, Environment, HtmlEscape};
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::collections::HashMap;

/// Wrap the Render to build the HTML markup.
#[derive(Debug, Default, Serialize)]
pub struct RendererWrapper {
    pub body: String,
    pub head: String,
    pub style: String,
    /// Global libraries provided by the design system.
    pub system_stylesheet: String,
    #[serde(with = "indexmap::map::serde_seq")]
    pub system_javascript_src: IndexMap<String, Value>,
    pub stylesheet: String,
    #[serde(with = "indexmap::map::serde_seq")]
    pub javascript_src: IndexMap<String, Value>,
    // Internal container for the string version of js to use on non json output.
    system_javascript: String,
    javascript: String,
}

impl RendererWrapper {
    pub fn new() -> Self {
        RendererWrapper {
            body: String::new(),
            head: String::new(),
            style: String::new(),
            system_stylesheet: String::new(),
            system_javascript_src: IndexMap::new(),
            stylesheet: String::new(),
            javascript_src: IndexMap::new(),
            system_javascript: String::new(),
            javascript: String::new(),
        }
    }

    pub fn add_body(&mut self, body: &str) {
        self.body.push_str(body);
    }

    pub fn add_body_nl(&mut self, body: &str) {
        if body.is_empty() {
            return;
        }
        self.body.push('\n');
        self.body.push_str(body);
    }

    pub fn add_head(&mut self, head: &str) {
        if head.is_empty() {
            return;
        }
        self.head.push('\n');
        self.head.push_str(head);
    }

    pub fn add_style(&mut self, style: &str) {
        if style.is_empty() {
            return;
        }
        self.style.push('\n');
        self.style.push_str(style);
    }

    pub fn add_system_stylesheet(&mut self, stylesheet: &str) {
        if stylesheet.is_empty() {
            return;
        }
        self.system_stylesheet.push('\n');
        self.system_stylesheet.push_str(stylesheet);
    }

    pub fn add_system_javascript(&mut self, script: &str) {
        self.system_javascript.push('\n');
        self.system_javascript.push_str(script);
    }

    pub fn add_system_javascript_src(&mut self, script_url: &str, data: Value) {
        self.system_javascript_src
            .insert(script_url.to_string(), data);
    }

    pub fn add_stylesheet(&mut self, stylesheet: &str) {
        if stylesheet.is_empty() {
            return;
        }
        self.stylesheet.push('\n');
        self.stylesheet.push_str(stylesheet);
    }

    pub fn add_javascript(&mut self, script: &str) {
        self.javascript.push('\n');
        self.javascript.push_str(script);
    }

    pub fn add_javascript_src(&mut self, script_url: &str, data: Value) {
        self.javascript_src.insert(script_url.to_string(), data);
    }

    /// Build Bubabble for this render.
    pub fn build(&mut self, bubbable: Bubbable) {
        self.build_system_library();
        self.build_bubbable(bubbable);
    }

    fn build_system_library(&mut self) -> &mut Self {
        // Get libraries defined by the design system (always loaded).
        let default_css: &str = DEFINITION.default_libraries_css_html;
        self.add_system_stylesheet(default_css);

        for (url, phf_attributes) in DEFINITION.default_libraries_js.iter() {
            let mut attributes = Map::new();
            for (key, value) in phf_attributes {
                attributes.insert(key.to_string(), Value::String(value.to_string()));
            }
            let new_attributes = Value::Object(attributes);

            // Create a string js with attributes for non json output.
            let js = Renderable::script(url, &new_attributes).to_html_string();

            self.add_system_javascript(&js);
            // Add regular js map with attributes as object.
            self.add_system_javascript_src(url, new_attributes);
        }

        self
    }

    fn build_bubbable(&mut self, bubbable: Bubbable) -> &mut Self {
        let css: String = bubbable.library.css.join("\n");
        self.add_stylesheet(&css);

        for (url, attributes) in bubbable.library.js {
            // Create a string js with attributes for non json output.
            let js = Renderable::script(&url, &attributes).to_html_string();
            self.add_javascript(&js);
            // Add regular js map with attributes as object.
            self.add_javascript_src(&url, attributes);
        }

        let attached_build: String = bubbable.attached_build.clone();
        self.add_head(&attached_build);

        let style: String = bubbable.style;
        if !style.is_empty() {
            self.add_style(&style);
        }

        self
    }
}

/// Simple render struct to process the data.
#[derive(Debug, Default)]
pub(crate) struct Renderer {
    pub output: RendererWrapper,
    pub translation: HashMap<String, String>,
}

impl Renderer {
    pub fn new() -> Self {
        Renderer {
            output: RendererWrapper::new(),
            translation: HashMap::new(),
        }
    }

    pub fn set_translation(&mut self, translation: HashMap<String, String>) {
        self.translation = translation;
    }

    pub fn render(&mut self, json: &Value) {
        // First pass is to collect all bubbable from 'json' recursively.
        let mut bubbable: Bubbable = Bubbable::new();
        bubbable.collect(json);
        // @todo move from bubbable to renderer.
        bubbable.render_variables();

        self.set_translation(bubbable.translation.clone());
        self.output.build(bubbable);

        let mut env: minijinja::Environment = engine::init_jinja_environnement();

        if json.is_array() {
            self.do_render(json.as_array().unwrap(), &mut env);
        } else {
            self.do_render(&[json.to_owned()], &mut env);
        }
    }

    /// Recursively render a serde_json Value.
    /// @todo is it a better place to wrap Value in a vec?
    pub fn do_render(&mut self, data: &[Value], env: &mut Environment) {
        for element in data.iter() {
            match element {
                Value::String(string) => {
                    let escaped: String = HtmlEscape(string).to_string();
                    self.output.add_body(&escaped);
                }
                Value::Bool(boolean) => {
                    self.output.add_body(boolean.to_string().as_str());
                }
                Value::Number(number) => {
                    self.output.add_body(number.to_string().as_str());
                }
                Value::Array(array) => {
                    self.do_render(array, env);
                }
                Value::Object(obj) => {
                    let ctx = context! { _translation => self.translation };
                    let mut renderable = Renderable::new(obj.to_owned());
                    renderable.build_with_env(env, ctx);
                    self.output.add_body_nl(&renderable.to_html_string());
                }
                _ => {
                    // @todo [devtools] log something
                    // todo!();
                }
            };
        }
    }
}

impl Output for Renderer {
    // @todo deprecate as we want to work only with serde_json::Value or minijinja::value::Value
    fn to_output_string(&self, output: &str) -> String {
        let mut style: String = "".to_string();
        if !self.output.style.is_empty() {
            style = format!(r#"
            <style>
                {}
            </style>
            "#, self.output.style);
        }

        let response: String = match output {
            "_test" => format!(
                r#"
                {}
                {}
                {}
                "#,
                self.output.body,
                self.output.stylesheet,
                self.output.javascript,
            ),
            "_test_full"=> format!(
                r#"
                {}
                {}
                {}
                {}
                {}
                {}
                {}
                "#,
                self.output.body,
                self.output.head,
                style,
                self.output.system_stylesheet,
                self.output.stylesheet,
                self.output.system_javascript,
                self.output.javascript,
            ),
            "full" => format!(
                r#"<!DOCTYPE html>
                <html>
                    <head>
                        {}
                        {}
                        {}
                        {}
                    </head>
                    <body>
                        {}
                        {}
                        {}
                    </body>
                </html>"#,
                self.output.head,
                self.output.system_stylesheet,
                self.output.stylesheet,
                style,
                self.output.body,
                self.output.system_javascript,
                self.output.javascript,
            ),
            "json" => serde_json::to_string(&self.to_output()).unwrap_or_else(|_| "".to_string()),
            "_logs" => "".to_string(),
            _ => format!("<!-- Unknown output: {} -->", output),
        };

        response
    }

    fn to_output(&self) -> Value {
        json!({
            "attached": self.output.head,
            "body": self.output.body,
            "system_stylesheet": self.output.system_stylesheet,
            "system_javascript": self.output.system_javascript_src,
            "stylesheet": self.output.stylesheet,
            "javascript": self.output.javascript_src,
            "variables": self.output.style,
        })
    }
}

/// Generate the response based on output type.
pub trait Output: std::fmt::Debug {
    fn to_output_string(&self, output: &str) -> String;
    fn to_output(&self) -> Value;
}

impl std::fmt::Display for dyn Output {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_output_string("full"))
    }
}
