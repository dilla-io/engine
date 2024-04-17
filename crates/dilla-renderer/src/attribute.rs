//! Handle HTML Attributes.
//!
//! An HTML attribute is managed and created to be rendered with an element or
//! from inside a MiniJinja template. **class** and **style** have some specific
//! management as they can come from scoped properties.
//! Other attributes are managed without any process.

// @todo see https://docs.rs/html5ever/latest/html5ever/struct.Attribute.html
// @todo check https://docs.rs/html_parser/0.7.0/src/html_parser/dom/mod.rs.html#328

use crate::scoped::Scoped;

use indexmap::IndexMap;
use minijinja::{
    value::{from_args, ValueKind},
    Error, State,
};
use serde::{Deserialize, Serialize};
use serde_json::Map;
use std::fmt;

pub const KEY_ATTRIBUTES: &str = "attributes";

/// An Attribute support data to generate HTML attribute markup.
/// @todo IndexMap is needed to ensure working test, as it can have a performance impact this could switch only for tests.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Attribute {
    attrs: IndexMap<String, Vec<String>>,
}

impl Attribute {
    pub fn new() -> Self {
        Attribute {
            attrs: IndexMap::new(),
        }
    }

    /// Returns a slice reference to the attribute values for the given name.
    ///
    /// If the attribute does not exist, returns an empty slice.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the attribute to retrieve
    ///
    /// # Returns
    ///
    /// A slice reference to the attribute values, or empty slice if not found.
    ///
    /// # Examples
    ///
    /// ```
    /// use dilla_renderer::attribute::Attribute;
    ///
    /// let mut attribute = Attribute::new();
    /// attribute.add_attr("class", vec!["foo", "bar"]);
    ///
    /// let empty = attribute.get_attr("unknown");
    /// assert!(empty.is_empty());
    ///
    /// let class = attribute.get_attr("class");
    /// assert_eq!(class, &["foo", "bar"]);
    /// ```
    pub fn get_attr(&self, name: &str) -> &[String] {
        match self.attrs.get(name) {
            Some(vec) => vec,
            None => &[],
        }
    }

    /// Build and update HTML attributes based on data, scoped properties.
    ///
    /// This function takes data, collect scoped properties, and combines them to generate
    /// a final set of HTML attributes. It specifically handles classes, styles, and other
    /// attributes.
    ///
    /// # Arguments
    ///
    /// * `data` - A reference to a JSON-like data structure (`serde_json::Map`) that contain
    /// attribute information from Payload.
    ///
    /// # Examples
    ///
    /// ```
    /// use dilla_renderer::attribute::Attribute;
    /// use serde_json::{json, Map};
    ///
    /// let mut attribute = Attribute::new();
    /// let mut data = Map::new();
    ///
    /// data.insert(
    ///   format!("@styles"),
    ///   json!(["style-1", "style-2"]),
    /// );
    /// data.insert(
    ///   format!("@local_variables"),
    ///   json!({"var-1": "#222", "var-2": "#333"}),
    /// );
    ///
    /// attribute.build_scoped(&data);
    ///
    /// assert!(attribute.has_attribute("class".into()));
    /// assert!(attribute.has_attribute("style".into()));
    /// assert!(attribute.has_class("style-1".into()));
    /// assert!(attribute.has_class("style-2".into()));
    /// attribute.remove_attr_by_name("class");
    /// assert_eq!(attribute.to_string(), " style=\"--var-1: #222; --var-2: #333;\"");
    /// ```
    ///
    /// # Returns
    ///
    /// A modified [`Attribute`] object with updated HTML attributes.
    pub fn build_scoped(&mut self, data: &Map<String, serde_json::Value>) {
        // Collect scoped to be used for attributes build.
        let mut scoped = Scoped::new();
        scoped.collect(data);

        // Add class from '@styles' if any.
        match !scoped.styles.is_empty() {
            true => {
                self.add_attr("class", scoped.styles);
            }
            false => (),
        }

        // Add style attribute values from '@local_variables' if any.
        match !scoped.local_variables.is_empty() {
            true => {
                self.add_attr("style", scoped.local_variables);
            }
            false => (),
        }

        // Add theme attribute values from '@theme' if any.
        match !scoped.theme_class.is_empty() {
            true => {
                self.add_attr("class", scoped.theme_class);
            }
            false => (),
        }
        match !scoped.theme_attribute.is_empty() {
            true => {
                let attr_values = scoped.theme_attribute.first().unwrap();
                self.add_attr(&attr_values.0.clone(), &attr_values.1.clone());
            }
            false => (),
        }
    }

    /// Process the HTML attributes based on the provided data.
    ///
    /// This function extracts and processes the HTML attributes from the given data.
    /// It handles class, style, and other attributes from the data payload.
    ///
    /// # Arguments
    ///
    /// * `data` - A reference to a `serde_json::Map` containing the attribute information.
    ///
    /// # Examples
    ///
    /// ```
    /// use dilla_renderer::attribute::Attribute;
    /// use serde_json::{json, Map, Value};
    ///
    /// let mut attribute = Attribute::new();
    /// let mut attr = json!({
    ///   "class": ["foo", "bar"],
    ///   "style": "color: blue;",
    ///   "data": "foo",
    ///   "data-bool": true,
    ///   "data-num": 56
    /// });
    ///
    /// attribute.build_attributes(&attr);
    ///
    /// assert!(attribute.has_attribute("class".into()));
    /// assert!(attribute.has_class("foo".into()));
    /// assert!(attribute.has_class("bar".into()));
    ///
    /// attribute.remove_attr_by_name("class");
    /// assert_eq!(attribute.to_string(), " data-num=\"56\" style=\"color: blue;\" data=\"foo\" data-bool=\"true\"");
    /// ```
    ///
    /// # Returns
    ///
    /// A modified [`Attribute`] object with updated HTML attributes.
    pub fn build_attributes(&mut self, attributes: &serde_json::Value) {
        if !attributes.is_object() {
            return;
        }

        let mut other_data = attributes.as_object().unwrap().clone();

        // "class" is always an array, even when set under "attributes" manually.
        if let Some(class) = attributes
            .get("class")
            .and_then(serde_json::Value::as_array)
        {
            let class_values: Vec<String> = class
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(String::from)
                .collect();
            self.add_attr("class", class_values);
            other_data.remove("class");
        }

        if let Some(serde_json::Value::String(style)) = attributes.get("style") {
            if style.ends_with(';') {
                self.add_attr("style", vec![style]);
            } else {
                self.add_attr("style", vec![format!("{};", style)]);
            };
            other_data.remove("style");
        }

        self.add_attr_from_serde(&serde_json::Value::Object(other_data));
    }

    /// Add attributes by name and value.
    /// This is used in context of element and internally in this Attribute.
    ///
    /// # Arguments
    ///
    /// * name - A string slice that represents the name of the attribute.
    /// * values - An iterable representing the attributes to be added.
    ///
    /// # Examples
    ///
    /// ```
    /// use dilla_renderer::attribute::Attribute;
    ///
    /// let mut attribute = Attribute::new();
    /// attribute.add_attr("class", vec!["foo", "bar"]);
    ///
    /// assert!(attribute.has_attribute("class".into()));
    /// assert!(attribute.has_class("foo".into()));
    /// assert!(attribute.has_class("bar".into()));
    /// assert!(!attribute.has_class("other".into()));
    /// assert!(!attribute.has_class("unknown".into()));
    /// ```
    pub fn add_attr(&mut self, name: &str, values: impl IntoIterator<Item = impl AsRef<str>>) {
        // Replace attribute instead of adding, this is the set_attribute default behavior.
        if name != "class" && name != "style" && self.attrs.contains_key(name) {
            self.attrs.swap_remove(name);
        }
        let entry = self.attrs.entry(name.to_string()).or_default();
        for value in values {
            entry.push(value.as_ref().to_string());
        }
    }

    /// Adds attributes from a minijinja::value::Value.
    ///
    /// This method takes a Value object, which is expected to be an iterable, and loops through each key-value pair. For each key-value pair, it attempts to retrieve the name and values of the attribute. If successful, it calls the add_attr_from_jinja method to add the attribute to the Attribute object.
    ///
    /// # Arguments
    ///
    /// * values - A minijinja::value::Value representing the attributes to be added.
    ///
    /// # Examples
    ///
    /// ```
    /// use dilla_renderer::attribute::Attribute;
    /// use minijinja::value::Value;
    ///
    /// let mut attribute = Attribute::new();
    /// let jinja_attrs = Value::from_iter(vec![
    ///   ("class", Value::from_iter(vec!["foo", "bar"])),
    ///   ("data-id", Value::from("123")),
    /// ]);
    /// attribute.add_attrs_from_jinja(&jinja_attrs);
    ///
    /// assert!(attribute.has_attribute("class".into()));
    /// assert!(attribute.has_class("foo".into()));
    /// assert!(attribute.has_class("bar".into()));
    /// assert!(!attribute.has_class("other".into()));
    /// assert!(attribute.has_attribute("data-id".into()));
    /// assert!(!attribute.has_attribute("unknown".into()));
    /// ```
    pub fn add_attrs_from_jinja(&mut self, values: &minijinja::value::Value) {
        if let Ok(iter) = values.try_iter() {
            for key in iter {
                if let Ok(values) = values.get_item(&key) {
                    let name = key.as_str().unwrap_or_default();
                    self.add_attr_from_jinja(name, values);
                }
            }
        }
    }

    /// Add attributes by name from a minijinja::value::Value.
    /// This is used in context of filters |add_class and |set_attribute
    ///
    /// # Arguments
    ///
    /// * name - A string slice that represents the name of the attribute.
    /// * values - A minijinja::value::Value representing the attributes to be added.
    ///
    /// # Examples
    ///
    /// ```
    /// use dilla_renderer::attribute::Attribute;
    /// use minijinja::value::Value;
    /// use serde_json::json;
    ///
    /// let mut attribute = Attribute::new();
    ///
    /// attribute.add_attr_from_jinja("class".into(), Value::from_iter(vec!["foo", "bar"]));
    /// // Note: style as manual input must include formatting.
    /// attribute.add_attr_from_jinja("style".into(), [("--var-1:", "#222;"), ("--var-2:", "#333;")].into_iter().collect());
    /// attribute.add_attr_from_jinja("data-id".into(), Value::from(123));
    ///
    /// assert!(attribute.has_attribute("class".into()));
    /// assert!(attribute.has_attribute("style".into()));
    /// assert!(attribute.has_class("foo".into()));
    /// assert!(attribute.has_class("bar".into()));
    /// assert!(!attribute.has_class("other".into()));
    ///
    /// assert!(attribute.has_attribute("data-id".into()));
    /// assert!(!attribute.has_attribute("data-unknown".into()));
    ///
    /// ```
    pub fn add_attr_from_jinja(&mut self, name: &str, values: minijinja::value::Value) {
        match values.kind() {
            ValueKind::Seq => {
                let mut new_values: Vec<String> = Vec::new();
                if let Ok(iter) = values.try_iter() {
                    for val in iter {
                        new_values.push(val.to_string())
                    }
                }
                self.add_attr(name, new_values);
            }
            ValueKind::Map => {
                let mut new_values: Vec<String> = Vec::new();
                if let Ok(iter) = values.try_iter() {
                    for key in iter {
                        if let Some(str_key) = key.as_str() {
                            if let Ok(val) = values.get_item(&key) {
                                new_values.push(str_key.to_string() + " " + &val.to_string())
                            }
                        }
                    }
                    self.add_attr(name, new_values);
                }
            }
            // Every other type is cast as string to handle int, bool, float...
            // @todo do we have a Map case?
            _ => self.add_attr(name, &[values.to_string()]),
        }
    }

    /// Add attributes from an iterable [`serde_json::Value`] serialized as [`minijinja::value`].
    /// This is used for component and element attributes management as we are working
    /// with serde_json::value.
    ///
    /// # Arguments
    ///
    /// * values - A serde_json::Value reference representing the attributes to be added.
    ///
    /// # Examples
    ///
    /// ```
    /// use dilla_renderer::attribute::Attribute;
    /// use serde_json::{json, Map, Value};
    ///
    /// let mut attribute = Attribute::new();
    /// let mut serde_attrs = Map::new();
    /// serde_attrs.insert(
    ///   "class".into(),
    ///   json!(["foo", "bar"]),
    /// );
    /// serde_attrs.insert(
    ///   "data-id".into(),
    ///   Value::from(123),
    /// );
    /// attribute.add_attr_from_serde(&Value::from(serde_attrs));
    ///
    /// assert!(attribute.has_attribute("class".into()));
    /// assert!(attribute.has_class("foo".into()));
    /// assert!(attribute.has_class("bar".into()));
    /// assert!(!attribute.has_class("other".into()));
    /// assert!(attribute.has_attribute("data-id".into()));
    /// assert!(!attribute.has_attribute("unknown".into()));
    /// ```
    pub fn add_attr_from_serde(&mut self, values: &serde_json::Value) {
        let jinja_attributes: minijinja::Value =
            minijinja::Value::from_serialize(&values);
        self.add_attrs_from_jinja(&jinja_attributes);
    }

    /// Merges attributes from a minijinja Value map into the current struct.
    ///
    /// This takes a minijinja Value that is a Map and extracts any string keys
    /// and values, merging them into the existing attributes of this struct.
    ///
    /// If an attribute already exists, the new values are appended to the existing
    /// Vec of values. Otherwise a new attribute is added with the given values.
    ///
    /// # Arguments
    ///
    /// * `values` - The minijinja Value map to extract attributes from
    ///
    /// # Examples
    ///
    /// ```
    /// use dilla_renderer::attribute::Attribute;
    /// use minijinja::value::Value;
    ///
    /// let mut attribute = Attribute::new();
    /// let jinja_attrs = Value::from_iter(vec![
    ///   ("class", Value::from_iter(vec!["foo", "bar"])),
    ///   ("data-id", Value::from("123")),
    /// ]);
    /// attribute.add_attrs_from_jinja(&jinja_attrs);
    ///
    /// assert!(attribute.has_attribute("class".into()));
    /// assert!(attribute.has_class("foo".into()));
    /// assert!(attribute.has_class("bar".into()));
    /// assert_eq!(attribute.get_attr("data-id".into()), &vec!["123".to_string()]);
    ///
    /// let merge_attrs = Value::from_iter(vec![
    ///   ("class", Value::from_iter(vec!["alpha", "bar"])),
    ///   ("data-id", Value::from("456")),
    ///   ("data-new", Value::from("new")),
    /// ]);
    /// attribute.merge_attrs_from_jinja(&merge_attrs);
    ///
    /// assert!(attribute.has_attribute("class".into()));
    /// assert!(attribute.has_class("foo".into()));
    /// assert!(attribute.has_class("bar".into()));
    /// assert!(attribute.has_class("alpha".into()));
    /// assert!(attribute.has_attribute("data-id".into()));
    /// assert_eq!(attribute.get_attr("data-id".into()), &vec!["456".to_string()]);
    /// assert!(attribute.has_attribute("data-new".into()));
    /// ```
    pub fn merge_attrs_from_jinja(&mut self, values: &minijinja::value::Value) {
        if values.kind() != ValueKind::Map {
            return;
        }

        if let Ok(iter) = values.try_iter() {
            for key in iter {
                if let Some(str_key) = key.as_str() {
                    if let Ok(val) = values.get_item(&key) {
                        let mut new_values: Vec<String> = Vec::new();

                        if self.has_attribute(key.clone()) {
                            match val.kind() {
                                ValueKind::String => {
                                    new_values.push(val.to_string());
                                    let existing = self.attrs.get(str_key).unwrap();
                                    if existing.len() > 1 {
                                        for value in self.attrs.get(str_key).unwrap().iter() {
                                            new_values.push(value.to_string());
                                        }
                                    }
                                    self.add_attr(str_key, &new_values);
                                }
                                ValueKind::Seq => {
                                    if let Ok(iter) = val.try_iter() {
                                        for vv in iter {
                                            new_values.push(vv.to_string());
                                        }
                                    }
                                    for value in self.attrs.get(str_key).unwrap().iter() {
                                        new_values.push(value.to_string());
                                    }
                                    self.add_attr(str_key, &new_values);
                                }
                                ValueKind::Map => {
                                    if let Ok(iter) = val.try_iter() {
                                        for kk in iter {
                                            let vv = val
                                                .get_item(&kk)
                                                .unwrap_or(minijinja::Value::UNDEFINED);
                                            new_values.push(kk.to_string());
                                            new_values.push(vv.to_string());
                                        }
                                    }
                                    self.add_attr(str_key, &new_values);
                                }
                                _ => {}
                            }
                        } else {
                            self.add_attr(str_key, vec![val.to_string()]);
                        }
                    }
                }
            }
        }
    }

    /// Removes the attribute with the specified name from the element.
    ///
    /// # Arguments
    ///
    /// * name - A string slice representing the name of the attribute to be removed.
    ///
    /// # Example
    ///
    /// ```
    /// use dilla_renderer::attribute::Attribute;
    ///
    /// let mut attribute = Attribute::new();
    /// attribute.add_attr("class", vec!["foo"]);
    /// assert!(attribute.has_attribute("class".into()));
    ///
    /// attribute.remove_attr_by_name("class");
    /// assert!(!attribute.has_attribute("class".into()));
    /// ```
    ///
    pub fn remove_attr_by_name(&mut self, name: &str) {
        self.attrs.swap_remove(name);
    }

    /// Removes the attribute with the specified name from a minijinja::value::Value.
    /// This is used in context of filter |remove_class
    ///
    /// # Arguments
    ///
    /// * name - A string slice representing the name of the attribute to be checked.
    /// * values - A minijinja::value::Value representing the attributes to be removed from the name.
    ///
    /// # Example
    ///
    /// ```
    /// use dilla_renderer::attribute::Attribute;
    /// use minijinja::value::Value;
    ///
    /// let mut attribute = Attribute::new();
    /// attribute.add_attr("class", vec!["foo", "bar"]);
    ///
    /// attribute.remove_class_by_name("bar".into());
    ///
    /// assert!(attribute.has_class("foo".into()));
    /// assert!(!attribute.has_class("bar".into()));
    ///
    /// attribute.add_attr("class", vec!["some", "other"]);
    /// attribute.remove_class_by_name("foo".into());
    /// attribute.remove_class_by_name("some".into());
    ///
    /// assert!(!attribute.has_class("foo".into()));
    /// assert!(!attribute.has_class("some".into()));
    /// assert!(attribute.has_class("other".into()));
    /// ```
    pub fn remove_class_by_name(&mut self, class: &str) {
        let mut classes = match self.attrs.get_mut("class") {
            Some(classes) => classes.clone(),
            None => return,
        };

        classes.retain(|c| *c != class);
        self.attrs.insert("class".to_string(), classes);
    }

    /// Check if the element has a specific class name under the 'class' attribute.
    /// This is used in context of filter |has_class
    ///
    /// # Arguments
    ///
    /// * class - A minijinja::value::Value representing the class name to check.
    ///
    /// # Returns
    ///
    /// * `bool`: `true` if the attribute has the specified class, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use dilla_renderer::attribute::Attribute;
    ///
    /// let mut attribute = Attribute::new();
    /// attribute.add_attr("class", vec!["container"]);
    /// assert!(attribute.has_class("container".into()));
    /// assert!(!attribute.has_class("row".into()));
    /// ```
    pub fn has_class(&self, class: minijinja::value::Value) -> bool {
        if let Some(classes) = self.attrs.get("class") {
            if classes.contains(&class.to_string()) {
                return true;
            }
        }
        false
    }

    /// Checks if the given attribute name exists.
    /// This is used in context of filter |has_attribute
    ///
    /// # Arguments
    ///
    /// * `name`: The name of the attribute to check.
    ///
    /// # Returns
    ///
    /// * `bool`: `true` if the attribute exists, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use dilla_renderer::attribute::Attribute;
    ///
    /// let mut attribute = Attribute::new();
    /// attribute.add_attr("foo", vec!["bar"]);
    /// assert!(attribute.has_attribute("foo".into()));
    /// assert!(!attribute.has_attribute("bar".into()));
    /// ```
    pub fn has_attribute(&self, name: minijinja::value::Value) -> bool {
        self.attrs.contains_key(&name.to_string())
    }
}

impl fmt::Display for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();
        for (name, values) in &self.attrs {
            let value = values.to_vec().join(" ");
            // Do not populate right part of attribute unless it's class or style.
            // @todo a list and a boolean check for value?
            // @todo async and defer when true?
            if value.is_empty() && name != "class" && name != "style" {
                output.push_str(&format!("{} ", name));
            } else {
                output.push_str(&format!("{}=\"{}\" ", name, values.to_vec().join(" ")));
            }
        }

        if !output.is_empty() {
            write!(f, r#" {}"#, output.trim_end())?;
        }

        Ok(())
    }
}

impl minijinja::value::Object for Attribute {
    fn call_method(
        &self,
        _state: &State,
        name: &str,
        args: &[minijinja::value::Value],
    ) -> Result<minijinja::value::Value, Error> {
        match name {
            "removeAttribute" => {
                let mut new_attributes = self.clone();
                let (attribute,): (&str,) = from_args(args)?;
                new_attributes.remove_attr_by_name(attribute);

                Ok(minijinja::value::Value::from_object(new_attributes))
            }
            _ => Err(Error::new(
                minijinja::ErrorKind::UnknownMethod,
                format!("Attribute object has no method named {}", name),
            )),
        }
    }
}

impl From<Attribute> for serde_json::Value {
    fn from(attribute: Attribute) -> Self {
        let json_map: serde_json::Map<String, serde_json::Value> = attribute
            .attrs
            .into_iter()
            .map(|(key, values)| (key, serde_json::Value::from(values)))
            .collect();
        serde_json::Value::Object(json_map)
    }
}

impl From<Attribute> for minijinja::Value {
    fn from(attribute: Attribute) -> Self {
        attribute
            .attrs
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().map(Into::into).collect::<Vec<String>>()))
            .collect()
    }
}

impl From<minijinja::Value> for Attribute {
    fn from(v: minijinja::Value) -> Self {
        let mut attrs: IndexMap<String, Vec<String>> = IndexMap::new();
        match v.kind() {
            ValueKind::Map => {
                if let Ok(iter) = v.try_iter() {
                    for key in iter {
                        if let Some(str_key) = key.as_str() {
                            if let Ok(val) = v.get_item(&key) {
                                if val.kind() == ValueKind::Seq {
                                    let mut new_values: Vec<String> = Vec::new();
                                    if let Ok(iter) = val.try_iter() {
                                        for vv in iter {
                                            // We can have some undefined or empty value here.
                                            // @todo find source of undefined
                                            if vv.to_string() != "undefined"
                                                || vv.to_string().is_empty()
                                            {
                                                new_values.push(vv.to_string());
                                            }
                                        }
                                    }
                                    attrs.insert(str_key.to_string(), new_values);
                                }
                                // Every other type is cast as string to handle int, bool, float...
                                // @todo add tests
                                else {
                                    attrs.insert(str_key.to_string(), vec![val.to_string()]);
                                }
                            }
                        }
                    }
                }
            }
            // Every other type is cast as string to handle int, bool, float...
            _ => {
                // @todo
            }
        }

        Attribute { attrs }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_class_present() {
        let mut attribute = Attribute::new();
        attribute.add_attr("class", vec!["example", "test"]);
    
        assert!(attribute.has_class(minijinja::value::Value::from("test")));
    }
    
    #[test]
    fn test_has_class_absent() {
        let mut attribute = Attribute::new();
        attribute.add_attr("class", vec!["example"]);
    
        assert!(!attribute.has_class(minijinja::value::Value::from("missing")));
    }

    #[test]
    fn test_get_attr_existing() {
        let mut attribute = Attribute::new();
        attribute.add_attr("class", vec!["btn", "active"]);

        let class_attrs = attribute.get_attr("class");
        assert_eq!(class_attrs, &["btn", "active"]);
    }

    #[test]
    fn test_get_attr_non_existing() {
        let attribute = Attribute::new();

        let class_attrs = attribute.get_attr("class");
        assert!(class_attrs.is_empty());
    }

    #[test]
    fn test_add_attr_single_value() {
        let mut attribute = Attribute::new();
        attribute.add_attr("class", vec!["btn"]);

        let expected_attrs: IndexMap<String, Vec<String>> = vec![(
            "class".to_string(),
            vec!["btn".to_string()].into_iter().collect(),
        )]
        .into_iter()
        .collect();

        assert_eq!(attribute.attrs, expected_attrs);
    }

    #[test]
    fn test_add_attr_multiple_values() {
        let mut attribute = Attribute::new();
        attribute.add_attr("class", vec!["btn", "active"]);

        let expected_attrs: IndexMap<String, Vec<String>> = vec![(
            "class".to_string(),
            vec!["btn".to_string(), "active".to_string()]
                .into_iter()
                .collect(),
        )]
        .into_iter()
        .collect();

        assert_eq!(attribute.attrs, expected_attrs);
    }
}
