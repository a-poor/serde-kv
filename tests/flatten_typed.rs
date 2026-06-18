//! Documents a known limitation: `#[serde(flatten)]` into a map with *typed*
//! (non-string) values is not supported.
//!
//! Flatten makes serde buffer fields into its self-describing `Content` enum via
//! `deserialize_any`. Because this format carries no type information, every
//! buffered value becomes a string, and serde will not coerce that string into a
//! numeric flattened field. Flatten catch-alls must therefore use string values
//! (`HashMap<String, String>`), which is tested in `tests/flatten.rs`.

use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct TypedExtra {
    foo: String,
    #[serde(flatten)]
    extra: HashMap<String, u64>,
}

#[test]
fn flatten_into_typed_map_is_unsupported() {
    let result: Result<TypedExtra, _> = serde_kv::from_str("foo=bar a=1 b=2");
    assert!(
        result.is_err(),
        "flatten into a typed map unexpectedly succeeded: {result:?}"
    );
}
