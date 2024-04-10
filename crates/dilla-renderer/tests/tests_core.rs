mod utils;

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
fn test_output_json() {
    utils::test_json("output", "payload", "_result.json");
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
