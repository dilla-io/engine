mod utils;

#[test]
fn test_output_string() {
    let name = "payload";
    let dir = "output";
    let suffix_expected = "_result_string.json";

    let data = utils::load(name, dir, suffix_expected);

    let result = dilla_renderer::render_string(data.0);

    let result_trim = utils::trim_whitespace(result.ok().unwrap().as_str());
    let expected = utils::trim_whitespace(data.1.as_str());
    similar_asserts::assert_eq!(
        expected,
        result_trim,
        "\n\n\n[TEST] payload: {dir}/{name}.json\n\n\n"
    );
}
#[test]
fn test_output_obj() {
    let name = "payload";
    let dir = "output";
    let suffix_expected = "_result_obj.json";

    let data = utils::load(name, dir, suffix_expected);

    let payload_obj: serde_json::Value = serde_json::from_str(&data.0).unwrap();
    let result: serde_json::Value = dilla_renderer::render_obj(&payload_obj);

    similar_asserts::assert_eq!(
        data.1,
        result.to_string(),
        "\n\n\n[TEST] payload: {dir}/{name}.json\n\n\n"
    );
}

#[test]
fn test_output_json() {
    let name = "payload";
    let dir = "output";
    let suffix_expected = "_result.json";

    let data = utils::load(name, dir, suffix_expected);

    let result = dilla_renderer::render_string(data.0);

    let result_trim = utils::trim_whitespace(result.ok().unwrap().as_str());
    let expected = utils::trim_whitespace(data.1.as_str());
    similar_asserts::assert_eq!(
        expected,
        result_trim,
        "\n\n\n[TEST] payload: {dir}/{name}.json\n\n\n"
    );
}

#[test]
fn test_output_test() {
    utils::test_loop("output", "_test", "_test.html");
}

#[test]
fn test_output_test_full() {
    utils::test_loop("output", "_test_full", "_test_full.html");
}

#[test]
fn test_output_full() {
    utils::test_loop("output", "full", "_full.html");
}

#[test]
fn test_element() {
    utils::test_loop("element", "_test", ".html");
}

#[test]
fn test_element_bubbable() {
    utils::test_loop("element_bubbable", "_test_full", ".html");
}

#[test]
fn test_component() {
    utils::test_loop("component", "_test", ".html");
}

#[test]
fn test_component_bubbable() {
    utils::test_loop("component_bubbable", "_test_full", ".html");
}

#[test]
fn test_engine_attribute() {
    utils::test_loop("attribute", "_test", ".html");
}

#[test]
fn test_engine_template() {
    utils::test_loop("template", "_test", ".html");
}

#[test]
fn test_engine_filter() {
    utils::test_loop("filter", "_test", ".html");
}

#[test]
fn test_engine_other() {
    utils::test_loop("other", "_test", ".html");
}

#[test]
fn test_engine_variables() {
    utils::test_loop("variables", "_test_full", ".html");
}

#[test]
fn test_filter() {
    utils::test_loop("filter", "_test", ".html");
}
