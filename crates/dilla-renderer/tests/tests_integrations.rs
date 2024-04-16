#![allow(unused_imports)]

mod utils;

use similar_asserts::assert_eq;

// Current DS tests generated.
#[cfg(feature = "test_ds")]
include!(concat!(env!("OUT_DIR"), "/codegen_tests.rs"));
