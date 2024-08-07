use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_quote, Visibility};

#[derive(Debug)]
pub(crate) struct UnstableAttribute {
    feature: Option<String>,
    issue: Option<String>,
}

impl UnstableAttribute {
    fn crate_feature_name(&self) -> String {
        if let Some(name) = self.feature.as_deref() {
            format!("nonstable-{}", name)
        } else {
            String::from("nonstable")
        }
    }

    pub(crate) fn expand(&self, mut item: impl ItemLike + ToTokens + Clone) -> TokenStream {
        // We only care about public items.
        if item.is_public() {
            let feature_name = self.crate_feature_name();

            if let Some(issue) = &self.issue {
                let doc_addendum = format!(
                    "\n\
                    # Availability\n\
                    \n\
                    **This API is marked as nonstable** and is only available when \
                    the `{}` crate feature is enabled. This comes with no stability \
                    guarantees, and could be changed or removed at any time.
                    The tracking issue is: [#{}](https://github.com/ZoeS17/CCG_Bot/issues/{})
                    ",
                    feature_name, &issue, issue
                );
                item.push_attr(parse_quote! {
                    #[doc = #doc_addendum]
                });
            } else {
                let doc_addendum = format!(
                    "\n\
                    # Availability\n\
                    \n\
                    **This API is marked as nonstable** and is only available when \
                    the `{}` crate feature is enabled. This comes with no stability \
                    guarantees, and could be changed or removed at any time.\
                ",
                    feature_name
                );
                item.push_attr(parse_quote! {
                    #[doc = #doc_addendum]
                });
            }

            let mut hidden_item = item.clone();
            *hidden_item.visibility_mut() = parse_quote! {
                pub(crate)
            };

            TokenStream::from(quote! {
                #[cfg(feature = #feature_name)]
                #item

                #[cfg(not(feature = #feature_name))]
                #[allow(dead_code)]
                #hidden_item
            })
        } else {
            item.into_token_stream().into()
        }
    }
}

impl From<syn::AttributeArgs> for UnstableAttribute {
    fn from(args: syn::AttributeArgs) -> Self {
        let mut feature = None;
        let mut issue = None;

        for arg in args {
            if let syn::NestedMeta::Meta(syn::Meta::NameValue(name_value)) = arg {
                if name_value.path.is_ident("feature") {
                    match name_value.lit {
                        syn::Lit::Str(s) => feature = Some(s.value()),
                        _ => panic!(),
                    }
                } else if name_value.path.is_ident("issue") {
                    match name_value.lit {
                        syn::Lit::Str(s) => issue = Some(s.value()),
                        _ => panic!(),
                    }
                }
            }
        }
        Self { feature, issue }
    }
}

pub(crate) trait ItemLike {
    fn attrs(&self) -> &[syn::Attribute];

    fn push_attr(&mut self, attr: syn::Attribute);

    fn visibility(&self) -> &Visibility;

    fn visibility_mut(&mut self) -> &mut Visibility;

    fn is_public(&self) -> bool {
        matches!(self.visibility(), Visibility::Public(_))
    }
}

macro_rules! impl_has_visibility {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl ItemLike for $ty {
                fn attrs(&self) -> &[syn::Attribute] {
                    &self.attrs
                }

                fn push_attr(&mut self, attr: syn::Attribute) {
                    self.attrs.push(attr);
                }

                fn visibility(&self) -> &Visibility {
                    &self.vis
                }

                fn visibility_mut(&mut self) -> &mut Visibility {
                    &mut self.vis
                }
            }
        )*
    };
}

impl_has_visibility!(
    syn::ItemType,
    syn::ItemEnum,
    syn::ItemStruct,
    syn::ItemFn,
    syn::ItemMod,
    syn::ItemTrait,
    syn::ItemConst,
    syn::ItemStatic,
);
