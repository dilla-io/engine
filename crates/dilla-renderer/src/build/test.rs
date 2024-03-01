#![allow(unused)]

use crate::build_config;
use crate::SystemConfig;
use std::collections::HashMap;

pub fn config() {
    #[allow(dead_code)]
    let design_system: &str = "test";

    let mut components_variant_template: HashMap<&str, Vec<&str>> = HashMap::new();
    let variant: Vec<&str> = vec!["variant", "variant2"];
    components_variant_template.insert("other", variant);

    let mut variables: HashMap<&str, &str> = HashMap::new();
    variables.insert("var-1", "#111111");
    variables.insert("var-2", "#222222");
    variables.insert("var-3", "#333333");
    variables.insert("var-4", "#444444");
    variables.insert("var-5", "#555555");

    let mut themes: HashMap<&str, HashMap<&str, &str>> = HashMap::new();
    let mut theme: HashMap<&str, &str> = HashMap::new();
    theme.insert("target", "");
    theme.insert("key", "data-bs-theme");
    theme.insert("val", "dark");
    themes.insert("dark", theme);
    let mut theme: HashMap<&str, &str> = HashMap::new();
    theme.insert("target", "");
    theme.insert("key", "data-bs-theme");
    theme.insert("val", "light");
    themes.insert("light", theme);

    let mut theme: HashMap<&str, &str> = HashMap::new();
    theme.insert("target", "");
    theme.insert("key", "class");
    theme.insert("val", "dark");
    themes.insert("dark-class", theme);
    let mut theme: HashMap<&str, &str> = HashMap::new();
    theme.insert("target", "");
    theme.insert("key", "class");
    theme.insert("val", "light");
    themes.insert("light-class", theme);

    #[allow(dead_code)]
    let styles: Vec<&str> = vec!["style-1", "style-2", "style-3", "style-4", "style-5"];

    let libraries_keys: Vec<&str> = vec!["test/test.dependency", "test/test.dependency-payload"];

    let mut default_libraries_js: Vec<(&str, HashMap<&str, &str>)> = Vec::new();

    let mut js_data_1: HashMap<&str, &str> = HashMap::new();
    js_data_1.insert("async", "true");
    default_libraries_js.push(("default-1.js", js_data_1));

    let mut js_data_2: HashMap<&str, &str> = HashMap::new();
    js_data_2.insert("async", "true");
    default_libraries_js.push(("default-2.js", js_data_2));

    let default_libraries_css_html: &str = r#"<link type=\"text/css\" rel=\"stylesheet\" href=\"default-1.css\" crossorigin=\"anonymous\">\n<link type=\"text/css\" rel=\"stylesheet\" href=\"default-2.css\" crossorigin=\"anonymous\">"#;

    let mut libraries_css_html: HashMap<&str, &str> = HashMap::new();

    let html: &str = r#"<link type=\"text/css\" rel=\"stylesheet\" href=\"test.dependency-1.css\" media=\"screen\">\n<link type=\"text/css\" rel=\"stylesheet\" href=\"test.dependency-2.css\" crossorigin=\"anonymous\">"#;
    libraries_css_html.insert("test/test.dependency", html);

    let html: &str =
        r#"<link type=\"text/css\" rel=\"stylesheet\" href=\"test.dependency-payload.css\">"#;
    libraries_css_html.insert("test/test.dependency-payload", html);

    let mut libraries_js: HashMap<&str, Vec<(&str, HashMap<&str, &str>)>> = HashMap::new();

    let mut lib_js: Vec<(&str, HashMap<&str, &str>)> = Vec::new();
    let mut js_data_1: HashMap<&str, &str> = HashMap::new();
    js_data_1.insert("async", "true");
    lib_js.push(("test.dependency-1.js", js_data_1));

    let mut js_data_2: HashMap<&str, &str> = HashMap::new();
    js_data_2.insert("defer", "true");
    lib_js.push(("test.dependency-2.js", js_data_2));

    libraries_js.insert("test/test.dependency", lib_js);

    let mut lib_js: Vec<(&str, HashMap<&str, &str>)> = Vec::new();
    let mut js_data_1: HashMap<&str, &str> = HashMap::new();
    js_data_1.insert("async", "true");
    lib_js.push(("test.dependency-payload.js", js_data_1));

    libraries_js.insert("test/test.dependency-payload", lib_js);

    let components_with_library: Vec<&str> = vec!["test_with_library", "other", "other.variant"];

    let dependencies: Vec<&str> = vec!["test/test.dependency"];
    let mut components_library_dependencies: HashMap<&str, Vec<&str>> = HashMap::new();
    components_library_dependencies.insert("test_with_library", dependencies);

    let mut components_library_css_html: HashMap<&str, &str> = HashMap::new();

    let html: &str = r#"<link type=\"text/css\" rel=\"stylesheet\" href=\"component-library-1.css\" media=\"screen\">\n<link type=\"text/css\" rel=\"stylesheet\" href=\"component-library-2.css\" media=\"screen\">"#;
    components_library_css_html.insert("test_with_library", html);

    let html: &str = r#"<link type=\"text/css\" rel=\"stylesheet\" href=\"component-library-other.css\" media=\"screen\">"#;
    components_library_css_html.insert("other", html);

    let mut components_library_js: HashMap<&str, Vec<(&str, HashMap<&str, &str>)>> = HashMap::new();

    let mut lib_js: Vec<(&str, HashMap<&str, &str>)> = Vec::new();
    let mut js_data_1: HashMap<&str, &str> = HashMap::new();
    js_data_1.insert("async", "true");
    lib_js.push(("component-library-1.js", js_data_1));

    let mut js_data_2: HashMap<&str, &str> = HashMap::new();
    js_data_2.insert("defer", "true");
    lib_js.push(("component-library-2.js", js_data_2));

    components_library_js.insert("test_with_library", lib_js);

    let mut lib_js: Vec<(&str, HashMap<&str, &str>)> = Vec::new();
    let js_data_1: HashMap<&str, &str> = HashMap::new();
    lib_js.push(("component-library-other.variant.js", js_data_1));

    components_library_js.insert("other.variant", lib_js);

    let config: SystemConfig = SystemConfig {
        design_system,
        components_library_css_html,
        components_library_dependencies,
        components_library_js,
        components_variant_template,
        components_with_library,
        default_libraries_css_html,
        default_libraries_js,
        libraries_css_html,
        libraries_js,
        libraries_keys,
        styles,
        variables,
        themes,
    };

    build_config(config);
}
