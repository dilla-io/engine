//! Handle Dilla Renderable.

use crate::{
    attribute::{Attribute, KEY_ATTRIBUTES},
    renderer::Renderer,
    DEFINITION, KEY_PREFIX,
};

use minijinja::{context, value::ValueKind, Environment};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map};

/// Component renderable key name.
pub const KEY_COMPONENT: &str = "component";
pub const KEY_COMPONENT_VARIANT: &str = "variant";
pub const SEP_COMPONENT_VARIANT: &str = ".";

/// Element renderable key name.
pub const KEY_ELEMENT: &str = "element";
const KEY_ELEMENT_CONTENT: &str = "content";
const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track",
    "wbr", "use",
];

/// Name of the template property.
pub const KEY_TEMPLATE: &str = "template";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum RenderableType {
    Component,
    Element,
    Template,
    #[default]
    Unknown,
}

/// This struct represents a renderable object.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Renderable {
    attributes: Attribute,
    data: Map<String, serde_json::Value>,
    fields: Map<String, serde_json::Value>,
    output: String,
    renderable_type: RenderableType,
    /// Component: Name of the component, used to resolve template.
    component_name: String,
    /// Component: Template name of the component.
    component_template: String,
    // Template: Source of the template.
    template_source: String,
    // Element: HTML tag
    element_tag: String,
    // Element: content property.
    element_content: String,
}

impl Renderable {
    /// Creates a new [`Renderable`].
    pub fn new(data: Map<String, serde_json::Value>) -> Self {
        Self {
            data,
            attributes: Attribute::new(),
            fields: Map::new(),
            output: String::new(),
            renderable_type: RenderableType::Unknown,
            component_name: String::new(),
            component_template: String::new(),
            template_source: String::new(),
            element_tag: String::new(),
            element_content: String::new(),
        }
    }

    pub fn add_attr(&mut self, name: &str, class: String) {
        self.attributes.add_attr(name, vec![class])
    }

    pub fn output(&self) -> String {
        self.output.clone() as _
    }

    pub fn build(&mut self) -> &mut Self {
        self.set_type_from_data();

        if self.renderable_type == RenderableType::Element {
            self.set_element_content_string()
        }

        self.set_renderable_values();
        self.render();

        self
    }

    pub fn build_with_env(&mut self, env: &mut Environment, ctx: minijinja::Value) -> &mut Self {
        self.set_type_from_data();

        if self.renderable_type == RenderableType::Element {
            self.set_element_content(env)
        }

        self.set_renderable_values();
        self.render_with_env(env, ctx);

        self
    }

    fn render_with_env(&mut self, env: &mut Environment, ctx: minijinja::Value) {
        if self.renderable_type == RenderableType::Template {
            self.output = self.render_template(env, ctx)
        } else if self.renderable_type == RenderableType::Component {
            self.output = self.render_component(env, ctx)
        } else if self.renderable_type == RenderableType::Element {
            self.render_element();
        }
    }

    fn render(&mut self) {
        if self.renderable_type == RenderableType::Element {
            self.render_element();
        } else {
            self.output = format!("[Warn] no render for: {:?}", self.renderable_type)
        }
    }

    fn render_element(&mut self) {
        // Differentiate tag with closing and void elements without content.
        if VOID_ELEMENTS.contains(&self.element_tag.as_str()) {
            self.output = format!(
                "<{tag}{attributes} />",
                tag = self.element_tag,
                attributes = self.attributes,
            );
            return;
        }

        self.output = format!(
            "<{tag}{attributes}>{content}</{tag}>",
            tag = self.element_tag,
            attributes = self.attributes,
            content = self.element_content,
        )
    }

    fn render_component(&mut self, env: &mut Environment, ctx: minijinja::Value) -> String {
        // Merge context values to have translation and fields.
        // @todo have fields directly as minijinja::value?
        let ctx_fields = minijinja::value::Value::from_serializable(&self.fields);
        let ctx = context! { ..ctx, ..ctx_fields };

        // Add attributes object to the template for manipulation and functions.
        // @todo use or not?
        // env.add_global(
        //     "attributes",
        //     minijinja::value::Value::from_object(component.attributes()),
        // );

        let template = env.get_template(self.component_template.as_str());
        if template.is_err() {
            #[cfg(feature = "debug")]
            println!("<!-- Debug\n{:?}\n-->", &env);
            return format!(
                "<!-- [Error] component: {}, template not found: {} -->",
                self.component_name, self.component_template
            );
        }

        template.unwrap().render(&ctx).unwrap()
    }

    fn render_template(&mut self, env: &mut Environment, ctx: minijinja::Value) -> String {
        let mut env: Environment = env.clone();

        env.add_template("inline", &self.template_source).unwrap();
        let template = env.get_template("inline").unwrap();

        // Merge context values to have translation and fields.
        let ctx_fields = minijinja::value::Value::from_serializable(&self.data);
        let ctx = context! { ..ctx, ..ctx_fields };

        let output = template.render(&ctx).unwrap();

        // @todo do we need to?
        env.remove_template("inline");

        output
    }

    fn set_type_from_data(&mut self) {
        let component_key: String = format!("{KEY_PREFIX}{KEY_COMPONENT}");
        let element_key: String = format!("{KEY_PREFIX}{KEY_ELEMENT}");
        let template_key: String = format!("{KEY_PREFIX}{KEY_TEMPLATE}");

        self.renderable_type = match (
            self.data.get(&element_key),
            self.data.get(&component_key),
            self.data.get(&template_key),
        ) {
            (Some(_), _, _) => RenderableType::Element,
            (_, Some(_), _) => RenderableType::Component,
            (_, _, Some(_)) => RenderableType::Template,
            _ => RenderableType::Unknown,
        }
    }

    fn set_renderable_values(&mut self) {
        if self.renderable_type == RenderableType::Component {
            self.set_component_name();
            self.set_component_name();
            self.set_fields();
            self.set_component_template();
            self.set_component_attributes();
        } else if self.renderable_type == RenderableType::Template {
            self.set_template_source();
            self.set_fields();
        } else if self.renderable_type == RenderableType::Element {
            self.set_element_tag();
            self.set_element_attributes();
        }
    }

    fn set_type(&mut self, renderable_type: RenderableType) {
        self.renderable_type = renderable_type;
    }

    fn set_tag(&mut self, tag: String) {
        self.element_tag = tag;
    }

    fn set_attributes(&mut self, attributes: Attribute) {
        self.attributes = attributes;
    }

    fn set_fields(&mut self) {
        self.fields = self
            .data
            .iter()
            .filter(|(k, _)| !k.starts_with(KEY_PREFIX))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
    }

    fn set_element_tag(&mut self) {
        self.element_tag = self
            .data
            .get(&format!("{KEY_PREFIX}{KEY_ELEMENT}"))
            .unwrap()
            .as_str()
            .unwrap_or_default()
            .to_string();
    }

    fn set_element_content(&mut self, env: &mut Environment) {
        // Element content need to access env and create a new renderer.
        let content_key = format!("{KEY_PREFIX}{KEY_ELEMENT_CONTENT}");

        if self.data.contains_key(&content_key) {
            let content_element = self.data.get(&content_key).unwrap();

            let mut single_renderer = Renderer::new();
            single_renderer.do_render(&[content_element.to_owned()], env);

            self.element_content = single_renderer.output.body;
        }
    }

    fn set_element_content_string(&mut self) {
        let content_key = format!("{KEY_PREFIX}{KEY_ELEMENT_CONTENT}");

        if self.data.contains_key(&content_key) {
            let content_element = self.data.get(&content_key).unwrap();
            if content_element.is_string() {
                // @todo set a @raw key or @markup to allow some html tags?
                // self.element_content = HtmlEscape(content_element.as_str().unwrap()).to_string();
                self.element_content = content_element.as_str().unwrap().to_string();
            } else if content_element.is_array() {
                for cont in content_element.as_array().unwrap().iter() {
                    // self.element_content
                    //     .push_str(&HtmlEscape(cont.as_str().unwrap()).to_string());
                    self.element_content.push_str(cont.as_str().unwrap());
                }
            }
        }
    }

    fn set_element_attributes(&mut self) {
        let attributes: &serde_json::Value = &self
            .data
            .iter()
            .filter(|(k, _)| !k.starts_with(KEY_PREFIX))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let mut obj_attributes = Attribute::new();

        obj_attributes.build_scoped(&self.data);
        obj_attributes.add_attr_from_serde(attributes);

        self.attributes = obj_attributes;
    }

    /// Shortcut to generate a specific `link` HTML tag with Element.
    pub fn link(href: &str, attrs: &serde_json::Value) -> Self {
        let mut defined_attributes = Attribute::new();

        defined_attributes.add_attr("type", vec!["text/css"]);
        defined_attributes.add_attr("rel", vec!["stylesheet"]);
        defined_attributes.add_attr("href", vec![href]);

        defined_attributes.add_attr_from_serde(attrs);

        let mut renderable = Renderable::default();
        renderable.set_type(RenderableType::Element);
        renderable.set_tag("link".to_string());
        renderable.set_attributes(defined_attributes);
        renderable.render();

        renderable
    }

    /// Shortcut to generate a specific `script` HTML tag with Element.
    pub fn script(src: &str, attrs: &serde_json::Value) -> Self {
        let mut defined_attributes = Attribute::new();

        defined_attributes.add_attr("src", vec![src]);

        defined_attributes.add_attr_from_serde(attrs);

        let mut renderable = Renderable::default();
        renderable.set_type(RenderableType::Element);
        renderable.set_tag("script".to_string());
        renderable.set_attributes(defined_attributes);
        renderable.render();

        renderable
    }

    fn set_template_source(&mut self) {
        let template_key: String = format!("{KEY_PREFIX}{KEY_TEMPLATE}");

        self.template_source = match self.data.get(&template_key) {
            Some(serde_json::Value::String(value)) => value.to_string(),
            _ => "".to_string(),
        };
    }

    // Set name and trim to avoid error.
    fn set_component_name(&mut self) {
        let key_component = KEY_PREFIX.to_string() + KEY_COMPONENT;

        self.component_name = match self
            .data
            .get(&key_component)
            .unwrap_or(&serde_json::Value::from(""))
            .as_str()
        {
            Some(name) if !name.is_empty() => name.trim().to_string(),
            _ => String::new(),
        };
    }

    // Detect @variant injected from config to add a slot with the related value.
    fn set_component_template(&mut self) {
        let key_variant = KEY_PREFIX.to_string() + KEY_COMPONENT_VARIANT;

        if self.data.get(&key_variant).is_some() {
            let variant = self.data.get(&key_variant).unwrap();

            // Insert the 'variant' property to be used in Jinja template.
            self.fields
                .insert(KEY_COMPONENT_VARIANT.to_owned(), variant.to_owned());

            if let Some(variants) = DEFINITION
                .components_variant_template
                .get(&self.component_name)
            {
                if variants.contains(&variant.as_str().unwrap()) {
                    self.component_template = self.component_name.to_string()
                        + SEP_COMPONENT_VARIANT
                        + variant.as_str().unwrap();
                    return;
                }
            }
        }

        self.component_template = self.component_name.clone()
    }

    fn set_component_attributes(&mut self) {
        // If `@attributes`, use it as Attribute and ignore `attributes`
        // If NOT `@attributes` and `attributes`, use it as Attribute
        let key_attributes = KEY_PREFIX.to_string() + KEY_ATTRIBUTES;

        let attributes = self.data.get(&key_attributes).unwrap_or_else(|| {
            self.data
                .get(KEY_ATTRIBUTES)
                .unwrap_or(&serde_json::Value::Null)
        });

        let mut obj_attributes = Attribute::new();

        obj_attributes.build_scoped(&self.data);
        if !attributes.is_null() {
            obj_attributes.build_attributes(attributes);
        }

        self.fields
            .insert(KEY_ATTRIBUTES.to_owned(), obj_attributes.into());
    }
}

impl Html for Renderable {
    fn to_html_string(&self) -> String {
        self.output()
    }
}

impl From<Renderable> for minijinja::Value {
    fn from(renderable: Renderable) -> Self {
        minijinja::Value::from_serializable(&renderable)
    }
}

impl From<minijinja::Value> for Renderable {
    fn from(v: minijinja::Value) -> Self {
        let json = serde_json::to_string(&v).unwrap();
        let data: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&json).unwrap();
        let mut renderable = Renderable::new(data);
        renderable.build();

        renderable
    }
}

pub trait Html: std::fmt::Debug {
    fn to_html_string(&self) -> String;
}

impl std::fmt::Display for dyn Html {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_html_string())
    }
}

impl Html for String {
    fn to_html_string(&self) -> String {
        self.clone()
    }
}

impl Html for &str {
    fn to_html_string(&self) -> String {
        self.to_string()
    }
}

/// Checks if a MiniJinja Value contains a renderable element.
///
/// This function takes an immutable reference to a minijinja::Value and checks if it contains
/// a renderable element.
///
/// # Arguments
///
/// * value - An immutable reference to the minijinja::Value that needs to be checked.
///
/// # Returns
///
/// * true if the value contains a renderable element, otherwise false.
///
pub fn is_renderable(value: &minijinja::Value) -> bool {
    if let Ok(mut iter) = value.try_iter() {
        iter.any(|key| match key.as_str() {
            Some(name) => {
                name == format!("{}{}", KEY_PREFIX, KEY_COMPONENT)
                    || name == format!("{}{}", KEY_PREFIX, KEY_ELEMENT)
                    || name == format!("{}{}", KEY_PREFIX, KEY_TEMPLATE)
            }
            _ => false,
        })
    } else {
        false
    }
}

/// Detect the RenderableType of a Minijinja value.
///
/// # Arguments
///
/// * value - An immutable reference to the minijinja::Value that needs to be checked.
///
/// # Returns
///
/// * [RenderableType] for this value.
///
pub fn get_renderable_type_from(value: &minijinja::Value) -> RenderableType {
    if let Ok(iter) = value.try_iter() {
        for key in iter {
            if let Some(cmp) = key.as_str() {
                if cmp == format!("{}{}", KEY_PREFIX, KEY_ELEMENT) {
                    return RenderableType::Element;
                } else if cmp == format!("{}{}", KEY_PREFIX, KEY_COMPONENT) {
                    return RenderableType::Component;
                } else if cmp == format!("{}{}", KEY_PREFIX, KEY_TEMPLATE) {
                    return RenderableType::Template;
                }
            }
        }
    }
    RenderableType::Unknown
}

/// Add class to a payload by converting minijinja to [`serde_json::Map`] for manipulation.
///
/// # Arguments
///
/// * `v` - A reference to a `minijinja::Value` object.
/// * `class` - A `minijinja::Value` string representing the class to be added.
///
/// # Returns
///
/// A `minijinja::Value` object representing the updated data map.
///
pub fn add_class_to_json_component(
    v: &minijinja::Value,
    class: minijinja::Value,
) -> minijinja::Value {
    let mut data = convert_to_map(v);

    let key_attributes = KEY_PREFIX.to_string() + KEY_ATTRIBUTES;
    let key_to_update = if !data.contains_key(KEY_ATTRIBUTES) && !data.contains_key(&key_attributes)
    {
        if class.kind() == ValueKind::String {
            data.insert(key_attributes, json!({"class": class.to_string()}));
        } else if class.kind() == ValueKind::Seq {
            let mut classes: String = "".to_string();
            if let Ok(iter) = class.try_iter() {
                for val in iter {
                    classes.push_str(&format!(" {} ", &val.to_string()));
                }
                classes = classes.trim().to_string();
            }
            data.insert(key_attributes, json!({"class": classes}));
        }
        return minijinja::Value::from_serializable(&data);
    } else if data.contains_key(KEY_ATTRIBUTES) {
        KEY_ATTRIBUTES.to_string()
    } else if data.contains_key(&key_attributes) {
        key_attributes
    } else {
        return minijinja::Value::from_serializable(&data);
    };

    let attributes = data.get(&key_to_update).unwrap().as_object().unwrap();
    let mut new_data = attributes.clone();

    add_or_merge_attr(&mut new_data, "class", class);

    data.remove(&key_to_update);
    data.insert(key_to_update, serde_json::Value::Object(new_data));

    minijinja::Value::from_serializable(&data)
}

/// Adds a class attribute to a JSON component.
///
/// # Arguments
///
/// * `v` - A reference to a `minijinja::Value` that represents the JSON component to which the attribute should be added.
/// * `class` - A minijinja::Value that represents the class name to be added to the JSON element.
///
/// # Returns
///
/// * `minijinja::Value` - The updated JSON component with the added attribute.
pub fn add_class_to_json_element(
    v: &minijinja::Value,
    class: minijinja::Value,
) -> minijinja::Value {
    let mut data = convert_to_map(v);
    add_or_merge_attr(&mut data, "class", class);
    minijinja::Value::from_serializable(&data)
}

/// Adds an attribute to a JSON component.
///
/// # Arguments
///
/// * `v` - A reference to a `minijinja::Value` that represents the JSON component to which the attribute should be added.
/// * `name` - A `String` that represents the name of the attribute to be added.
/// * `value` - An `Option<minijinja::Value>` that represents the value of the attribute to be added. If `None`, an empty string is used as the value.
///
/// # Returns
///
/// * `minijinja::Value` - The updated JSON component with the added attribute.
pub fn add_attr_to_json_component(
    v: &minijinja::Value,
    name: String,
    value: Option<minijinja::Value>,
) -> minijinja::Value {
    let mut data = convert_to_map(v);
    let key_attributes = KEY_PREFIX.to_string() + KEY_ATTRIBUTES;
    let key_to_update;

    if !data.contains_key(KEY_ATTRIBUTES) && !data.contains_key(&key_attributes) {
        let value_str = value.map_or_else(|| "".to_string(), |v| v.to_string());
        data.insert(
            key_attributes,
            json!({
                name: value_str
            }),
        );
        return minijinja::Value::from_serializable(&data);
    } else if data.contains_key(KEY_ATTRIBUTES) {
        key_to_update = KEY_ATTRIBUTES.to_string();
    } else if data.contains_key(&key_attributes) {
        key_to_update = key_attributes;
    } else {
        return minijinja::Value::from_serializable(&data);
    }

    let attributes = data.get(&key_to_update).unwrap().as_object().unwrap();
    let mut new_data = attributes.clone();

    let value_str = value.map_or_else(|| "".to_string(), |v| v.to_string());
    add_or_merge_attr(&mut new_data, &name, minijinja::Value::from(value_str));

    data.remove(&key_to_update);
    data.insert(key_to_update, serde_json::Value::Object(new_data));

    minijinja::Value::from_serializable(&data)
}

/// Adds an attribute to a JSON element.
///
/// # Arguments
///
/// * `v` - A reference to the `minijinja::Value` that will be converted to a map and modified.
/// * `name` - The name of the attribute to be added to the JSON element.
/// * `value` - The value of the attribute to be added. If `None`, an empty string will be added.
///
/// # Returns
///
/// * `minijinja::Value` - The modified JSON element with the new attribute added.
pub fn add_attr_to_json_element(
    v: &minijinja::Value,
    name: String,
    value: Option<minijinja::Value>,
) -> minijinja::Value {
    let mut data = convert_to_map(v);
    if value.is_none() {
        data.insert(name, json!(""));
    } else {
        data.insert(name, json!(value.unwrap().to_string()));
    }
    minijinja::Value::from_serializable(&data)
}

fn add_or_merge_attr(
    data: &mut Map<String, serde_json::Value>,
    key: &str,
    value: minijinja::Value,
) {
    if !data.contains_key(key) {
        data.insert(key.to_string(), serde_json::Value::from(value.to_string()));
        return;
    }

    let existing_value = data.get_mut(key).unwrap();
    let mut merged_values: Vec<String> = Vec::new();

    // Check the type of the existing value
    match existing_value {
        serde_json::Value::Array(existing_array) => {
            // If the existing value is an array, convert each element to a string and add it to the merged values vector
            for val in existing_array.iter() {
                if let Some(string_val) = val.as_str() {
                    merged_values.push(string_val.to_owned());
                }
            }
        }
        serde_json::Value::String(existing_string) => {
            // If the existing value is a string, add it to the merged values vector
            merged_values.push(existing_string.to_owned());
        }
        _ => {}
    }

    merged_values.push(value.to_string());

    // Remove the existing key from the data map
    data.remove(key);

    // Insert the key back into the data map with the merged values vector as the value
    data.insert(key.to_string(), serde_json::Value::from(merged_values));
}

fn convert_to_map(value: &minijinja::Value) -> Map<String, serde_json::Value> {
    let json = serde_json::to_string(value).unwrap();
    serde_json::from_str(&json).unwrap()
}
