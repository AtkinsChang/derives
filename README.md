# Derives
Some useful derive macro for rust.

## `Display`
Derive for `Display` trait.
```rust
use derives::{Display};

#[derive(Display)]
enum MyStruct {
    #[display("unit")]
    Unit,
    // display can be omit if the only field is Display
    OneField(usize),
    OneFieldStruct {
        s: &'static str,
    },
    #[display("number: {}", _0)]
    OneFieldCustom(usize),
    #[display("({},{})", _0, _1)]
    MultiField(usize, usize),
    #[display("({},{})", _0, _1)]
    MultiFieldStruct {
        x: usize,
        y: usize,
    },
}

assert_eq!(MyStruct::Unit.to_string(), "unit");
assert_eq!(MyStruct::OneField(87).to_string(), "87");
assert_eq!(MyStruct::OneFieldStruct { s: "test" }.to_string(), "test");
assert_eq!(MyStruct::OneFieldCustom(87).to_string(), "number: 87");
assert_eq!(MyStruct::MultiField(87, 88).to_string(), "(87,88)");
assert_eq!(MyStruct::MultiFieldStruct { x: 87, y: 88 }.to_string(), "(87,88)");
```

## `Error`
TODO
