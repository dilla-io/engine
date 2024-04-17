//! Use MiniJinja engine to render **@component** and inline **@template**.
//!
//! Include all Dilla filter functions and abstractions.

use crate::attribute::Attribute;
use crate::renderable::*;
use crate::renderer::Renderer;

use minijinja::{
    escape_formatter, value::ValueKind, AutoEscape, Environment, Error, Output, State,
};

#[cfg(feature = "random")]
use rand::{thread_rng, Rng};

use serde_json::Map;

/// Initializes a Jinja environment with various filters, functions, globals,
/// and sets a formatter to render Maps in templates.
/// Goal is to instantiate only once the env for the whole payload request.
///
/// # Returns
///
/// A Minijinja `Environment` object is being returned.
///
pub(crate) fn init_jinja_environnement() -> Environment<'static> {
    let mut env: Environment = Environment::new();

    // #[cfg(feature = "debug")]
    // env.set_debug(true);

    // @todo estimate if better?
    // env.set_undefined_behavior(UndefinedBehavior::Chainable);

    env.set_auto_escape_callback(|_| AutoEscape::Html);

    minijinja_embed::load_templates!(&mut env);

    env.add_filter("t", t);
    env.add_filter("split", split);
    env.add_filter("prepend", prepend);
    env.add_filter("append", append);
    env.add_filter("clean_id", clean_id);
    env.add_filter("set_attribute", set_attribute);
    env.add_filter("has_attribute", has_attribute);
    env.add_filter("add_class", add_class);
    env.add_filter("has_class", has_class);
    env.add_filter("remove_class", remove_class);
    env.add_filter("remove_attribute", remove_attribute);
    env.add_filter("merge", merge);

    env.add_function("create_attribute", create_attribute);

    env.add_global(
        "random",
        minijinja::value::Value::from_function(generate_random_string),
    );

    // Important part to be able to loop and call from template to render Maps.
    env.set_formatter(
        |out: &mut Output, state: &State, value: &minijinja::value::Value| {
            engine_formatter(out, state, value)
        },
    );

    env
}

/// Renders a value in an engine-specific format for Minijinja templates.
/// This formatter is used by the init_jinja_environnement() function.
///
/// # Arguments
///
/// * out - A mutable reference to the output where the formatted value will be written.
/// * state - An immutable reference to the state of the rendering process.
/// * value - An immutable reference to the value that needs to be formatted.
///
/// # Errors
///
/// This function returns an Error if there is an error while writing the formatted value to the output.
///
fn engine_formatter(
    out: &mut Output,
    state: &State,
    value: &minijinja::value::Value,
) -> Result<(), Error> {
    // A Map is an object for Minijinja.
    if value.kind() == ValueKind::Map {
        if is_renderable(value) {
            let output: String = render_value(state, value);
            return write!(out, "{}", output).map_err(Error::from);
        }
        // If not renderable then it's probably an attribute.
        else if value.as_object().is_some() {
            if value.as_object().unwrap().is::<Attribute>() {
                return write!(out, "{value}").map_err(Error::from);
            }
            // If not an attribute then nothing to do.
            // @todo log something?
        } else {
            let mut attribute: Attribute = Attribute::new();
            attribute.add_attrs_from_jinja(value);
            return write!(out, "{attribute}").map_err(Error::from);
        }
    // If we have an array (Seq for Minijinja).
    } else if value.kind() == ValueKind::Seq {
        let output: String = render_value(state, value);
        return write!(out, "{output}").map_err(Error::from);
    // Approximately check for already rendered attributes in templates to avoid escape.
    } else if value.kind() == ValueKind::String {
        let test: &str = value.as_str().unwrap();
        if test.contains("class=") || test.contains("id=") {
            return write!(out, "{value}").map_err(Error::from);
        }
    }

    // Fallback to print with escape.
    escape_formatter(
        out,
        state,
        if value.is_none() {
            &minijinja::value::Value::UNDEFINED
        } else {
            value
        },
    )
}

/// Renders a JSON value into a formatted string according to the provided state.
///
/// This function takes a reference to a `State` instance and a reference to a `Value` instance.
/// The `Value` instance represents a JSON value that needs to be rendered. The rendering process
/// considers the state and the structure of the JSON value to generate a formatted string.
///
/// # Arguments
///
/// * `state` - A reference to the `minijinja::State` instance that provides context for rendering.
/// * `value` - A reference to the `minijinja::value::Value` instance representing the JSON value to be rendered.
///
/// # Returns
///
/// A formatted string representing the rendered output of the provided JSON value.
///
/// # Notes
///
/// This function supports rendering JSON objects, arrays, and strings. Other types of JSON values will
/// result in an empty string in the rendered output.
///
/// The rendering process may involve recursive calls when dealing with nested JSON structures.
///
fn render_value(state: &State, value: &minijinja::value::Value) -> String {
    fn _render_value_recursive(state: &State, value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::Object(obj) => _render_object(state, obj),
            serde_json::Value::Array(arr) => _render_array(state, arr),
            serde_json::Value::String(s) => s.clone(),
            _ => String::new(),
        }
    }

    fn _render_array(state: &State, arr: &[serde_json::Value]) -> String {
        let mut output = String::new();
        for value in arr {
            output += &_render_value_recursive(state, value);
        }
        output
    }

    fn _render_object(state: &State, obj: &Map<String, serde_json::Value>) -> String {
        let mut renderer: Renderer = Renderer::new();
        let mut env: Environment = state.env().clone();
        renderer.do_render(&[serde_json::Value::Object(obj.clone())], &mut env);
        renderer.output.body.to_string()
    }

    let data: serde_json::Value = serde_json::to_value(value).unwrap_or_default();
    _render_value_recursive(state, &data)
}

/// Creates an HTML attribute Object as [`Attribute`].
///
/// @deprecated not used anymore in Twig, but perhaps we could keep.
///
/// ```jinja
/// {{ create_attribute({'id': 'bar', class': ['foo', 'bar']}) }}
/// ```
///
pub fn create_attribute(
    value: Option<minijinja::value::Value>,
) -> Result<minijinja::value::Value, Error> {
    if let Some(attributes) = value {
        if attributes.kind() == ValueKind::Map {
            let mut attribute: Attribute = Attribute::new();
            attribute.add_attrs_from_jinja(&attributes);
            return Ok(minijinja::value::Value::from_object(attribute));
        }
    }

    Ok(minijinja::value::Value::from_object(Attribute::new()))
}

/// Translate a string.
///
/// Translation source is from payload key "@trans" which is a dict of
/// original -> translated.
///
/// ```json
/// {
///   "@trans": {
///     "Foo": "Baguette",
///     "Hello @name!": "Bonjour @name!"
///   }
/// }
/// ```
///
/// ```jinja
/// {{ 'Foo'|t }}
///   -> Baguette
/// {{ 'Hello @name!'|t({'@name': 'Bob'}) }}
///   -> Bonjour Bob!
/// ```
///
pub fn t(
    state: &minijinja::State,
    value: String,
    variables: Option<minijinja::value::Value>,
) -> String {
    let translation = state.lookup("_translation").unwrap_or_default();

    if translation.is_undefined() {
        return value;
    }

    if let Ok(existing_trans) = translation.get_item(&minijinja::value::Value::from(value.clone()))
    {
        if existing_trans.is_undefined() {
            return value;
        }

        // Strange serde behaviour adding quote around strings...
        let mut result: String = existing_trans
            .as_str()
            .unwrap()
            .to_string()
            .replace('"', "");
        if let Some(v) = variables {
            if v.kind() == ValueKind::Map {
                if let Ok(iter) = v.try_iter() {
                    for key in iter {
                        if let Some(str_key) = key.as_str() {
                            if let Ok(value) = v.get_item(&key) {
                                result = result.replace(str_key, &value.to_string());
                            }
                        }
                    }
                }
            }
            // @todo [devtools] if not map, ie Seq or string: |t([...]) |t(XXX), should give warning.
        }
        return result;
    }

    value
}

/// Split a string based on a separator and return an array strings.
///
/// ```jinja
/// {{ "My__split__string"|split('__') }}
///   -> ["My", "split", "string"]
/// ```
///
pub fn split(value: minijinja::value::Value, sep: String) -> Result<Vec<String>, Error> {
    if let Some(s) = value.as_str() {
        Ok(s.split(&sep).map(String::from).collect::<Vec<_>>())
    } else {
        Err(Error::new(
            minijinja::ErrorKind::InvalidOperation,
            format!("Cannot split value of type {}", value.kind()),
        ))
    }
}

/// Prepend a string to a [`minijinja::Value`] String or Seq.
///
/// ```jinja
/// {{ 'one'|prepend('Some') }}
///   -> Someone
/// ```
///
/// ```jinja
/// {% set foo = ['foo', 'bar']|prepend('prepend') %}
/// {{ foo|join(', ') }}
///   -> prepend, foo, bar
/// ```
///
pub fn prepend(
    value: minijinja::value::Value,
    prepend: String,
) -> Result<minijinja::value::Value, Error> {
    if let Some(s) = value.as_str() {
        Ok(minijinja::value::Value::from(prepend + s))
    } else if let Some(seq) = value.as_seq() {
        let mut rv: Vec<minijinja::value::Value> = seq.iter().collect::<Vec<_>>();
        rv.insert(0, minijinja::value::Value::from(prepend));
        Ok(minijinja::value::Value::from(rv))
    } else {
        Err(Error::new(
            minijinja::ErrorKind::InvalidOperation,
            format!("Cannot prepend value of type {}", value.kind()),
        ))
    }
}

/// Append a string to the [`minijinja::Value`] String or Seq.
///
/// ```jinja
/// {{ 'Some'|append('one') }}
///   -> Someone
/// ```
///
/// ```jinja
/// {% set foo = ['foo', 'bar']|append('append') %}
/// {{ foo|join(', ') }}
///   -> foo, bar, append
/// ```
///
pub fn append(
    v: minijinja::value::Value,
    append: minijinja::value::Value,
) -> Result<minijinja::value::Value, Error> {
    if let Some(s) = v.as_str() {
        let mut rv: String = String::new();
        rv.push_str(s);
        rv.push_str(append.as_str().unwrap());
        Ok(minijinja::value::Value::from(rv))
    } else if let Some(seq) = v.as_seq() {
        let mut rv: Vec<minijinja::value::Value> = seq.iter().collect::<Vec<_>>();
        rv.push(append);
        Ok(minijinja::value::Value::from(rv))
    } else {
        Err(Error::new(
            minijinja::ErrorKind::InvalidOperation,
            format!("Cannot append value of type {}", v.kind()),
        ))
    }
}

/// Add an HTML attribute to a Seq, which are always treated as [`Attribute`].
///
/// @see <https://git.drupalcode.org/project/drupal/-/blob/11.x/core/lib/Drupal/Core/Template/Attribute.php#L119>
///
/// ```jinja
/// {{ {'id': 'foo'}|set_attribute('href', 'bar') }}
///   -> href="bar" id="foo"
/// ```
///
/// @todo if value is map, concatenate to strings with space?
/// @todo if name exist, replace?
/// @todo when class or styles, should merge?
pub fn set_attribute(
    v: minijinja::Value,
    name: String,
    value: Option<minijinja::Value>,
) -> Result<minijinja::Value, Error> {
    // If Seq, probably a nested element. We loop elements and apply the filter.
    // @todo [devtools] log a nested element when not needed, ie only one.
    let mut nv: Vec<minijinja::Value> = Vec::new();
    if v.kind() == ValueKind::Seq {
        if let Ok(iter) = v.try_iter() {
            for val in iter {
                if val.kind() == ValueKind::Map {
                    match get_renderable_type_from(&val) {
                        RenderableType::Element => {
                            nv.push(add_attr_to_json_element(&val, name.clone(), value.clone()));
                        }
                        RenderableType::Component => {
                            nv.push(add_attr_to_json_component(
                                &val,
                                name.clone(),
                                value.clone(),
                            ));
                        }
                        _ => (),
                    }
                }
            }
        }
        return Ok(minijinja::Value::from_serialize(&nv));
    }

    // String or Array are Map.
    if v.kind() != ValueKind::Map {
        return Ok(v);
    }

    match get_renderable_type_from(&v) {
        RenderableType::Element => Ok(add_attr_to_json_element(&v, name, value)),
        RenderableType::Component => Ok(add_attr_to_json_component(&v, name, value)),
        // It's not possible to add attributes to a template.
        RenderableType::Template => Ok(v),
        // Default is an `Attribute` object definition Map.
        _ => {
            let mut attributes = <minijinja::Value as Into<Attribute>>::into(v);
            attributes.add_attr_from_jinja(&name, value.unwrap_or_default());
            Ok(attributes.into())
        }
    }
}

/// Check existing HTML attribute in a Map of attributes treated as [`Attribute`].
///
/// ```jinja
/// {{ {'data': 'test'}|has_attribute('data') }}
///   -> true
/// {{ {'data': 'test'}|has_attribute('foo') }}
///   -> false
/// ```
///
pub fn has_attribute(
    v: minijinja::value::Value,
    name: minijinja::value::Value,
) -> Result<minijinja::value::Value, Error> {
    if v.kind() != ValueKind::Map {
        return Ok(minijinja::value::Value::from(false));
    }

    if let Some(obj) = v.as_object() {
        if obj.is::<Attribute>() {
            let ref_attribute: &Attribute = obj.downcast_ref().unwrap();
            let attribute: Attribute = ref_attribute.to_owned();
            if attribute.has_attribute(name) {
                return Ok(minijinja::value::Value::from(true));
            }
        }
    } else if let Ok(iter) = v.try_iter() {
        for key in iter {
            if key == name {
                return Ok(minijinja::value::Value::from(true));
            }
        }
    }

    Ok(minijinja::value::Value::from(false))
}

/// Prepares a string for use as a valid HTML ID.
///
/// @see <https://api.drupal.org/api/drupal/core%21lib%21Drupal%21Component%21Utility%21Html.php/function/Html%3A%3AgetId>
///
/// ```jinja
/// id="{{ 'A B_c-d[e]f  %$*#€  éèà G____H'|clean_id }}"
///   -> id="a-b-c-d-e-f-g-h"
/// ```
///
pub fn clean_id(obj: minijinja::value::Value) -> String {
    if obj.kind() != ValueKind::String {
        return obj.as_str().unwrap().to_string();
    }

    obj.as_str()
        .unwrap()
        .to_lowercase()
        .replace([' ', '_', '[', ']'], "-")
        // Strip non ascii and non numeric or alphabetic
        .replace(|c: char| !c.is_ascii_alphanumeric() && c != '-', "")
        // Removing multiple consecutive hyphens.
        .replace("-----", "-")
        .replace("----", "-")
        .replace("---", "-")
        .replace("--", "-")
}

/// Add classes to [`Attribute`], a Seq or renderable.
///
/// ```jinja
/// {{ {'data': 'test'}|add_class('bar') }}
///   -> class="bar" data="test"
/// {{ {'data': 'test', 'class': 'bar'}|add_class('foo') }}
///   -> class="bar foo" data="test"
/// {{ {'data': 'test', 'class': ['some', 'other']}|add_class(['foo', 'bar']) }}
///   -> class="bar foo other some" data="test"
/// ```
///
pub fn add_class(v: minijinja::Value, class: minijinja::Value) -> Result<minijinja::Value, Error> {
    if v.kind() == ValueKind::Undefined {
        return Ok(v);
    }

    let mut attributes: Attribute = Attribute::new();
    if v.kind() == ValueKind::Map {
        match get_renderable_type_from(&v) {
            RenderableType::Element => return Ok(add_class_to_json_element(&v, class)),
            RenderableType::Component => return Ok(add_class_to_json_component(&v, class)),
            RenderableType::Template => return Ok(v),
            // Everything else is treated as an Attribute.
            _ => attributes = v.into(),
        }
    }
    // If Seq, probably a nested element. We only use the first element and apply the filter.
    // @todo devtools: log a nested element when not needed.
    // @todo check if array of different child types, should go recursive? and concatenate if string ?
    // For now only first array is taken into account.
    // ie: {{ ["test1", "test2"]|add_class('some') }}
    // ie: {{ ["test1", { "@element": "test", "@content": "Some"}]|add_class('some') }}
    else if v.kind() == ValueKind::Seq {
        if let Ok(item) = v.get_item_by_index(0) {
            return add_class(item, class);
        }
    } else if v.kind() == ValueKind::String {
        return Ok(v);
    }

    let nc: Vec<String> = _iter_seq_to_vec(class);
    attributes.add_attr("class", nc);

    Ok(attributes.into())
}

fn _iter_seq_to_vec(class: minijinja::Value) -> Vec<String> {
    match class.kind() {
        ValueKind::Seq => {
            let mut tc: Vec<String> = Vec::new();
            class.try_iter().into_iter().for_each(|iter| {
                for c in iter {
                    tc.push(c.to_string());
                }
            });
            tc
        }
        _ => vec![class.to_string()],
    }
}

/// Remove classes to a Seq, which are always treated as [`Attribute`].
///
/// ```jinja
/// {{ {'data': 'test', 'class': ['foo', 'bar']}|remove_class('foo') }}
///   -> class="bar" data="test"
/// {{ {'data': 'test', 'class': 'bar'}|remove_class('bar') }}
///   -> data="test"
/// {{ {'data': 'test', 'class': ['some', 'other', 'foo', 'bar']}|remove_class(['some', 'other']) }}
///   -> class="bar foo" data="test"
/// ```
///
pub fn remove_class(
    v: minijinja::value::Value,
    class: minijinja::value::Value,
) -> Result<minijinja::value::Value, Error> {
    remove(v, Some(class.as_str().unwrap_or_default()), None)
}

/// Remove element by name of an [`Attribute`].
///
/// ```jinja
/// {{ {'id': 'test', 'data': 'test, 'class': ['foo', 'bar']}|remove_attribute('id') }}
///   -> class="bar foo" data="test"
/// {{ {'data': 'test', 'class': 'bar'}|remove_attribute('class') }}
///   -> data="test"
/// ```
///
pub fn remove_attribute(
    v: minijinja::value::Value,
    name: minijinja::value::Value,
) -> Result<minijinja::value::Value, Error> {
    remove(v, None, Some(name.as_str().unwrap_or_default()))
}

/// Removes a class or attribute from a [`minijinja::Value`].
///
/// # Arguments
///
/// * `v` - The value object from which the class or attribute is to be removed.
/// * `class` - An optional class name to be removed from the value object.
/// * `name` - An optional attribute name to be removed from the value object.
///
/// # Returns
///
/// A `Result` containing either the modified value object or an `Error`.
/// If the value object is not of map type, it is returned as is.
/// If the modification is successful, the modified value object is returned.
///
fn remove(
    v: minijinja::value::Value,
    class: Option<&str>,
    name: Option<&str>,
) -> Result<minijinja::value::Value, Error> {
    // String or Array are Map.
    if v.kind() != ValueKind::Map {
        return Ok(v);
    }

    if let Some(obj) = v.as_object() {
        if obj.is::<Attribute>() {
            let ref_attribute: &Attribute = obj.downcast_ref().unwrap();
            let mut attribute: Attribute = ref_attribute.to_owned();

            if let Some(c) = class {
                attribute.remove_class_by_name(c);
            } else if let Some(n) = name {
                attribute.remove_attr_by_name(n);
            }

            return Ok(minijinja::value::Value::from_object(attribute));
        }
    }

    let mut attribute: Attribute = Attribute::new();
    attribute.add_attrs_from_jinja(&v);

    if let Some(c) = class {
        attribute.remove_class_by_name(c);
    } else if let Some(n) = name {
        attribute.remove_attr_by_name(n);
    }

    Ok(minijinja::value::Value::from_object(attribute))
}

/// Check if has class from Seq, which are always treated as [`Attribute`].
///
/// ```jinja
/// {{ {'data': 'test'}|has_class('bar') }}
///   -> false
/// {{ {'data': 'test', 'class': 'foo'}|has_class('foo') }}
///   -> true
/// ```
///
pub fn has_class(
    v: minijinja::value::Value,
    class: minijinja::value::Value,
) -> Result<minijinja::value::Value, Error> {
    // String or Array are Map.
    if v.kind() != ValueKind::Map {
        return Ok(minijinja::value::Value::from(false));
    }
    if let Some(obj) = v.as_object() {
        if obj.is::<Attribute>() {
            let ref_attribute: &Attribute = obj.downcast_ref().unwrap();
            let attribute: Attribute = ref_attribute.to_owned();
            if attribute.has_class(class) {
                return Ok(minijinja::value::Value::from(true));
            }
        }
    } else if let Ok(iter) = v.try_iter() {
        for key in iter {
            let cur = v.get_item(&key).unwrap_or_default();
            if cur.kind() == ValueKind::String {
                if v.get_item(&key).unwrap_or_default() == class {
                    return Ok(minijinja::value::Value::from(true));
                }
            } else if cur.kind() == ValueKind::Seq {
                if let Ok(seq) = cur.try_iter() {
                    for key in seq {
                        if key == class {
                            return Ok(minijinja::value::Value::from(true));
                        }
                    }
                }
            }
        }
    }

    Ok(minijinja::value::Value::from(false))
}

/// Generate a random string from a number of type i32.
///
/// ```jinja
/// {{ random() }}
///   -> 2005036924
/// ```
///
pub fn generate_random_string() -> Result<minijinja::value::Value, Error> {
    #[cfg(not(feature = "random"))]
    return Ok(minijinja::value::Value::from("no-random"));

    // With rand and getrandom.
    #[cfg(feature = "random")]
    let rand: i32 = thread_rng().gen::<i32>();
    #[cfg(feature = "random")]
    Ok(minijinja::value::Value::from(rand))
}

/// Merge Map, which are always treated as [`Attribute`].
///
/// ```jinja
/// {{ { "foo": "bar" }|merge({"wu": "tang"}) }}
///   -> foo="bar" wu="tang"
/// {{ { "foo": "bar" }|merge({"foo": "baz"}) }}
///   -> foo="baz"
/// {{ { "foo": ["bar", "baz"] }|merge({"foo": ["alpha", "beta"]}) }}
///   -> foo="alpha beta bar baz"
/// {{ { "foo": "bar" }|merge({"foo": ["alpha", "beta"]}) }}
///   -> foo="alpha beta bar"
/// {{ {"foo": ["alpha", "beta"]}|merge({ "foo": "bar" }) }}
///   -> foo="bar alpha beta"
/// {{ { "foo": {"bar": "baz"} }|merge({ "foo": {"wu": "tang"} }) }}
///   -> foo="wu tang"
/// ```
///
pub fn merge(
    v: minijinja::value::Value,
    v2: minijinja::value::Value,
) -> Result<minijinja::value::Value, Error> {
    if v.kind() != ValueKind::Map || v2.kind() != ValueKind::Map {
        return Ok(v);
    }

    let mut attribute: Attribute = Attribute::new();
    attribute.add_attrs_from_jinja(&v);
    attribute.merge_attrs_from_jinja(&v2);

    Ok(minijinja::Value::from_object(attribute))
}
