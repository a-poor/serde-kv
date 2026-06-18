//! Property-based round-trip tests, focused on the quote/escape logic.
//!
//! Keys are restricted to identifier-like strings (the format cannot represent a
//! key containing `=`, a space, or leading/trailing whitespace), but values are
//! arbitrary unicode strings — including spaces, quotes, backslashes, newlines,
//! and the empty string. The invariant under test is
//! `from_str(to_string(map)) == map`.

use std::collections::BTreeMap;

use proptest::prelude::*;

/// An identifier-like key: starts with a letter, no `=`/whitespace.
fn key_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9_]{0,12}"
}

/// An arbitrary value string, biased to include the characters that exercise
/// quoting and escaping.
fn value_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(String::new()),
        r#"[a-zA-Z0-9 "\\\t\n=]*"#.prop_map(|s| s),
        any::<String>(),
    ]
}

proptest! {
    /// Serializing a string map and parsing it back yields the original map.
    #[test]
    fn string_map_round_trips(
        map in prop::collection::btree_map(key_strategy(), value_strategy(), 0..8)
    ) {
        let line = serde_kv::to_string(&map).unwrap();
        let back: BTreeMap<String, String> = serde_kv::from_str(&line).unwrap();
        prop_assert_eq!(back, map);
    }

    /// `parse_line` (via `from_str`) never panics on arbitrary input.
    #[test]
    fn parsing_arbitrary_input_never_panics(input in any::<String>()) {
        let _ = serde_kv::from_str::<BTreeMap<String, String>>(&input);
    }

    /// A single value survives a serialize/parse round-trip regardless of content.
    #[test]
    fn single_value_round_trips(value in value_strategy()) {
        let mut map = BTreeMap::new();
        map.insert("k".to_string(), value.clone());
        let line = serde_kv::to_string(&map).unwrap();
        let back: BTreeMap<String, String> = serde_kv::from_str(&line).unwrap();
        prop_assert_eq!(back.get("k"), Some(&value));
    }
}
