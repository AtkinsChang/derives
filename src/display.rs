use proc_macro2::{Span, TokenStream};
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{punctuated::IntoIter, spanned::Spanned, Lit, Meta, MetaList, NestedMeta};
use synstructure::{Structure, VariantInfo};

use crate::parse_meta_with_path;

fn find_display(info: &VariantInfo) -> Option<(Lit, IntoIter<NestedMeta>)> {
    let mut result = None;

    for attr in info.ast().attrs {
        if let Some(MetaList { nested, .. }) = parse_meta_with_path(attr, "display") {
            let mut iter = nested.into_iter();
            if let Some(NestedMeta::Lit(lit)) = iter.next() {
                if result.is_some() {
                    abort!(attr.span(), "Cannot have two `display` attributes");
                }
                result = Some((lit, iter));
            }
        }
    }

    result
}

fn cvt_arg(s: &Structure, v: &VariantInfo, arg: &NestedMeta) -> TokenStream {
    let abort_not_exist = |span: Span, index| -> ! {
        abort!(
            span,
            "attempted to access field `{}` in `{}::{}` which does not exist (there {} {} field{})",
            index,
            s.ast().ident,
            v.ast().ident,
            if v.bindings().len() != 1 { "are" } else { "is" },
            v.bindings().len(),
            if v.bindings().len() != 1 { "s" } else { "" }
        );
    };
    match &arg {
        NestedMeta::Lit(Lit::Int(id)) => {
            let index = id
                .base10_parse::<usize>()
                .unwrap_or_else(|_| abort!(id.span(), "integer literal overflows usize"));

            if let Some(binding) = v.bindings().get(index) {
                quote! { #binding }
            } else {
                abort_not_exist(id.span(), index)
            }
        }
        NestedMeta::Meta(Meta::Path(path)) => {
            if let Some(id) = path.get_ident() {
                // same ident
                for binding in v.bindings() {
                    if let Some(ref ident) = binding.ast().ident {
                        if ident == id {
                            return quote! { #binding };
                        }
                    }
                }

                {
                    let id_s = id.to_string();
                    if id_s.starts_with('_') {
                        if let Ok(index) = id_s[1..].parse::<usize>() {
                            if let Some(binding) = v.bindings().get(index) {
                                return quote! { #binding };
                            } else {
                                abort_not_exist(id.span(), index)
                            }
                        }
                    }
                }

                abort!(
                    id.span(),
                    "attempted to access unknown field `{}` in `{}::{}`",
                    id,
                    s.ast().ident,
                    v.ast().ident
                )
            } else {
                quote! { #arg }
            }
        }
        NestedMeta::Meta(Meta::List(list)) => quote! { #list },
        _ => abort!(arg.span(), "invalid argument to `display` attribute."),
    }
}

#[proc_macro_error(allow_not_macro, assert_unwind_safe)]
pub(crate) fn derive(s: Structure) -> proc_macro::TokenStream {
    let display_impl = {
        let fmt_method = {
            let body = s.each_variant(|v| {
                if let Some((format, args)) = find_display(v) {
                    let args = args.map(|arg| cvt_arg(&s, v, &arg));
                    quote! {
                        write!(f, #format #(, #args)*)
                    }
                } else if let [binding] = v.bindings() {
                    quote! {
                        std::fmt::Display::fmt(#binding, f)
                    }
                } else {
                    abort!(
                        v.ast().ident.span(),
                        "variant with more then one field must have `display` attribute."
                    );
                }
            });

            quote! {
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    match *self { #body }
                }
            }
        };

        s.unbound_impl(
            quote!(::core::fmt::Display),
            quote! {
                #fmt_method
            },
        )
    };

    quote!(#display_impl).into()
}
