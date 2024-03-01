mod utils;

#[test]
fn test_all() {
    test_output();
    test_element();
    test_component();
    test_engine();
    test_filter();
}

#[test]
fn test_output() {
    utils::test_loop("output", "_test", "_test.html");
    utils::test_loop("output", "_test_full", "_test_full.html");
    utils::test_loop("output", "full", "_full.html");
    utils::test_json("output", "payload", "_result.json");
}

#[test]
fn test_element() {
    utils::test_loop("element", "_test", ".html");
    utils::test_loop("element_bubbable", "_test_full", ".html");
}

#[test]
fn test_component() {
    utils::test_loop("component", "_test", ".html");
    utils::test_loop("component_bubbable", "_test_full", ".html");
}

#[test]
fn test_engine() {
    utils::test_loop("attribute", "_test", ".html");
    utils::test_loop("template", "_test", ".html");
    utils::test_loop("filter", "_test", ".html");
    utils::test_loop("other", "_test", ".html");
    utils::test_loop("variables", "_test_full", ".html");
}

#[test]
fn test_filter() {
    // utils::test_loop("attribute", "_test", ".html");
    utils::test_loop("filter", "_test", ".html");
}
