mod display;
mod error;

extern crate proc_macro;

use syn::{Attribute, Meta, MetaList};
use synstructure::decl_derive;

decl_derive!([Error, attributes(error)] => error::derive);
decl_derive!([Display, attributes(display)] => display::derive);

fn parse_meta_with_path<P>(attr: &Attribute, path: &P) -> Option<MetaList>
where
    P: ?Sized + AsRef<str>,
{
    match attr.parse_meta() {
        Ok(Meta::List(list)) if list.path.is_ident(path) => Some(list),
        _ => None,
    }
}
