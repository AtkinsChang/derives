use std::fmt::Display;

use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{self, Meta, MetaList, NestedMeta};
use synstructure::{BindingInfo, Structure, VariantInfo};

use crate::parse_meta_with_path;

fn find_binding<'a, 'b, I>(info: &'a VariantInfo<'b>, ident: &I) -> Option<&'a BindingInfo<'b>>
where
    I: ?Sized + Display + AsRef<str>,
{
    let mut result = None;

    for binding in info.bindings() {
        for attr in &binding.ast().attrs {
            if let Some(MetaList { nested, .. }) = parse_meta_with_path(attr, "error") {
                for meta in nested {
                    if let NestedMeta::Meta(Meta::Path(path)) = meta {
                        if path.is_ident(ident) {
                            if result.is_some() {
                                abort!(
                                    info.ast().ident.span(),
                                    "Cannot have two `{}` attributes",
                                    ident
                                );
                            }
                            result = Some(binding);
                        }
                    }
                }
            }
        }
    }

    result
}

#[proc_macro_error(allow_not_macro, assert_unwind_safe)]
pub(crate) fn derive(s: Structure) -> proc_macro::TokenStream {
    let error_impl = if cfg!(feature = "std") {
        let source_method = {
            let body = s.each_variant(|v| {
                match (find_binding(v, "source"), find_binding(v, "maybe_source")) {
                    (Some(_), Some(_)) => abort!(
                        v.ast().ident.span(),
                        "Cannot have both `source` and `maybe_source` attributes"
                    ),
                    (Some(source), None) => quote!(Some(#source)),
                    (None, Some(maybe_source)) => quote! {
                        match #maybe_source {
                            Some(source) => Some(source),
                            None => None,
                        }
                    },
                    (None, None) => quote!(None),
                }
            });

            quote! {
                fn source(&self) -> ::std::option::Option<&(dyn ::std::error::Error +'static)> {
                    match *self { #body }
                }
            }
        };

        let backtrace_method = if cfg!(feature = "unstable") {
            let body = s.each_variant(|v| {
                match (find_binding(v, "backtrace"), find_binding(v, "maybe_backtrace")) {
                    (Some(_), Some(_)) => abort!(
                        v.ast().ident.span(),
                        "Cannot have both `backtrace` and `maybe_backtrace` attributes"
                    ),
                    (Some(backtrace), None) => quote!(Some(#backtrace)),
                    (None, Some(maybe_backtrace)) => quote! {
                        match #maybe_backtrace {
                            Some(backtrace) => Some(backtrace),
                            None => None,
                        }
                    },
                    (None, None) => quote!(None),
                }
            });

            quote! {
                fn backtrace(&self) -> ::std::option::Option<&::std::backtrace::Backtrace> {
                    match *self { #body }
                }
            }
        } else {
            quote!()
        };

        s.unbound_impl(
            quote!(::std::error::Error),
            quote! {
                #source_method
                #backtrace_method
            },
        )
    } else {
        quote!()
    };

    quote!(#error_impl).into()
}
