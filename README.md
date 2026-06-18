# serde-kv

[![crates.io](https://img.shields.io/crates/v/serde-kv.svg)](https://crates.io/crates/serde-kv)
[![docs.rs](https://docs.rs/serde-kv/badge.svg)](https://docs.rs/serde-kv)
[![license](https://img.shields.io/crates/l/serde-kv.svg)](#license)

`serde-kv` is a [serde](https://github.com/serde-rs/serde) data format implementation
for a basic key/value string.

The format is a space-separated list of `key=value` pairs. Values are bare tokens,
or double-quoted when they contain spaces or special characters:

```text
foo=bar baz=42 cool=true message="hello world"
```

There is **no type inference at the format level** — the type comes from the target
struct field. A raw token like `42` becomes the string `"42"` for a `String` field,
or the integer `42` for a `u64` field. Quotes are purely for parsing, so
`foo=42` and `foo="42"` decode to the same value.

## Example

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Record {
    foo: String,
    baz: u64,
    cool: bool,
    message: String,
}

fn main() {
    let line = r#"foo=bar baz=42 cool=true message="hello world""#;

    // Deserialize a line into a struct.
    let record: Record = serde_kv::from_str(line).unwrap();
    assert_eq!(record.baz, 42);
    assert_eq!(record.message, "hello world");

    // Serialize it back to a line.
    let out = serde_kv::to_string(&record).unwrap();
    assert_eq!(out, line);
}
```

You can also deserialize into a map, where every value is a string:

```rust
use std::collections::HashMap;

let map: HashMap<String, String> =
    serde_kv::from_str(r#"foo=bar baz=42 msg="hello world""#).unwrap();
assert_eq!(map["baz"], "42");
```

## Supported types

- Flat scalars: `String`, all integer/float widths, `bool`, `char`.
- `Option<T>`: a missing key deserializes to `None`; `None` is skipped when serializing.
- Unknown keys are ignored, or collected with `#[serde(flatten)] extra: HashMap<String, String>`.

Nested structs and sequences are not supported.
