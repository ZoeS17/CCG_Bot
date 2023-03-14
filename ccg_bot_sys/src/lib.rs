use proc_macro::TokenStream;
use syn::{parse_macro_input, Item};

mod unstable;

#[proc_macro_attribute]
pub fn unstable(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as syn::AttributeArgs);
    let attr = unstable::UnstableAttribute::from(args);

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
