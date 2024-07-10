#![allow(dead_code)]

#[cfg(test)]
use trybuild;

use proc_macro::TokenStream;
use syn::{parse_macro_input, Item};

mod nonstable;

#[proc_macro_attribute]
pub fn nonstable(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as syn::AttributeArgs);
    let attr = nonstable::UnstableAttribute::from(args);

    match parse_macro_input!(item as Item) {
        Item::Type(item_type) => attr.expand(item_type),
        Item::Enum(item_enum) => attr.expand(item_enum),
        Item::Struct(item_struct) => attr.expand(item_struct),
        Item::Fn(item_fn) => attr.expand(item_fn),
        Item::Mod(item_mod) => attr.expand(item_mod),
        Item::Trait(item_trait) => attr.expand(item_trait),
        Item::Const(item_const) => attr.expand(item_const),
        Item::Static(item_static) => attr.expand(item_static),
        _ => panic!("unsupported item type"),
    }
}

#[test]
fn nonstable_without_feature() {
    let t = trybuild::TestCases::new();
    t.pass("src/tests/nonstable_without_feature.rs");
}

#[test]
fn nonstable_with_feature() {
    let t = trybuild::TestCases::new();
    t.pass("src/tests/nonstable_with_feature.rs");
}

#[test]
fn nonstable_without_issue() {
    let t = trybuild::TestCases::new();
    t.pass("src/tests/nonstable_without_issue.rs");
}

#[test]
fn nonstable_failure_with_use_item_type() {
    let t = trybuild::TestCases::new();
    t.compile_fail("src/tests/nonstable_fail_item_type.rs");
}

#[test]
fn nonstable_without_issue_or_feature() {
    let t = trybuild::TestCases::new();
    t.pass("src/tests/nonstable_without_issue_or_feature.rs");
}

#[test]
fn nonstable_with_feature_and_reason() {
    let t = trybuild::TestCases::new();
    t.pass("src/tests/nonstable_with_feature_and_reason.rs");
}

#[test]
fn nonstable_failure_with_non_str_feature() {
    let t = trybuild::TestCases::new();
    t.compile_fail("src/tests/nonstable_fail_non_str_feature.rs");
}

#[test]
fn nonstable_failure_with_non_str_issue() {
    let t = trybuild::TestCases::new();
    t.compile_fail("src/tests/nonstable_fail_non_str_issue.rs");
}
