#![allow(unused_imports)]

mod utils;

use similar_asserts::assert_eq;

// Current DS tests generated.
include!(concat!(env!("OUT_DIR"), "/codegen_tests.rs"));
