//! Table-driven coverage for value interpretation, quoting, whitespace, and
//! numeric boundaries.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct S {
    v: String,
}

/// String values: each input string must serialize to the expected line and
/// round-trip back to the same string.
#[test]
fn quoting_matrix() {
    let cases: &[(&str, &str)] = &[
        ("bar", "v=bar"),
        ("42", "v=42"),                     // numeric-looking string stays bare
        ("with=equals", "v=with=equals"),   // '=' needs no quoting
        ("hello world", r#"v="hello world""#),
        ("", r#"v="""#),                    // empty must be quoted
        (r#"a"b"#, r#"v="a\"b""#),          // embedded quote
        (r#"a\b"#, r#"v="a\\b""#),          // embedded backslash
        ("  spaces  ", r#"v="  spaces  ""#), // leading/trailing spaces in value
        ("tab\tok", "v=tab\tok"),           // tab is not a separator -> bare
    ];
    for (raw, expected) in cases {
        let line = serde_kv::to_string(&S { v: (*raw).into() }).unwrap();
        assert_eq!(line, *expected, "serialize {raw:?}");
        let back: S = serde_kv::from_str(&line).unwrap();
        assert_eq!(back.v, *raw, "round-trip {raw:?}");
    }
}

/// The same token is interpreted differently per target type.
#[test]
fn value_interpretation() {
    #[derive(Deserialize)]
    struct AsStr { v: String }
    #[derive(Deserialize)]
    struct AsI64 { v: i64 }
    #[derive(Deserialize)]
    struct AsF64 { v: f64 }
    #[derive(Deserialize)]
    struct AsBool { v: bool }

    assert_eq!(serde_kv::from_str::<AsStr>("v=42").unwrap().v, "42");
    assert_eq!(serde_kv::from_str::<AsI64>("v=-42").unwrap().v, -42);
    assert_eq!(serde_kv::from_str::<AsF64>("v=2.5").unwrap().v, 2.5);
    assert_eq!(serde_kv::from_str::<AsF64>("v=1e3").unwrap().v, 1000.0);
    assert!(serde_kv::from_str::<AsBool>("v=true").unwrap().v);
    // Quotes never change interpretation.
    assert_eq!(serde_kv::from_str::<AsI64>(r#"v="-42""#).unwrap().v, -42);
}

/// Whitespace handling around and between pairs.
#[test]
fn whitespace_handling() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct AB { a: u32, b: u32 }

    let cases: &[&str] = &[
        "a=1 b=2",
        "a=1   b=2",   // multiple separating spaces
        "  a=1 b=2",   // leading
        "a=1 b=2   ",  // trailing
        "  a=1   b=2  ",
    ];
    for input in cases {
        assert_eq!(
            serde_kv::from_str::<AB>(input).unwrap(),
            AB { a: 1, b: 2 },
            "input {input:?}"
        );
    }
}

/// Empty input deserializes to an empty map / all-optional struct.
#[test]
fn empty_input() {
    let map: BTreeMap<String, String> = serde_kv::from_str("").unwrap();
    assert!(map.is_empty());
    assert!(serde_kv::from_str::<BTreeMap<String, String>>("   ").unwrap().is_empty());

    #[derive(Deserialize, PartialEq, Debug)]
    struct AllOpt { a: Option<u32> }
    assert_eq!(serde_kv::from_str::<AllOpt>("").unwrap(), AllOpt { a: None });
}

/// Numeric parse failures: overflow, sign mismatch, and non-numeric tokens.
#[test]
fn numeric_boundaries() {
    use serde_kv::Error;

    #[derive(Deserialize, Debug)]
    struct U8 { #[allow(dead_code)] v: u8 }
    #[derive(Deserialize, Debug)]
    struct I32 { #[allow(dead_code)] v: i32 }

    let fails: &[&str] = &[
        "v=256",   // u8 overflow
        "v=-1",    // negative into unsigned
        "v=3.5",   // float into integer
        "v=abc",   // not a number
        "v=",      // empty token into integer
    ];
    for input in fails {
        let err = serde_kv::from_str::<U8>(input).unwrap_err();
        assert!(matches!(err, Error::ParseInt(_)), "{input:?} -> {err:?}");
    }
    // i32 at the limits parses fine; one past overflows.
    assert_eq!(serde_kv::from_str::<I32>("v=2147483647").unwrap().v, i32::MAX);
    assert!(matches!(
        serde_kv::from_str::<I32>("v=2147483648").unwrap_err(),
        Error::ParseInt(_)
    ));
}

/// Unicode keys, values, and a single multi-byte `char`.
#[test]
fn unicode() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct U {
        emoji: String,
        c: char,
    }
    let line = r#"emoji="héllo 🌍" c=🦀"#;
    let got: U = serde_kv::from_str(line).unwrap();
    assert_eq!(got, U { emoji: "héllo 🌍".into(), c: '🦀' });
    assert_eq!(serde_kv::to_string(&got).unwrap(), line);
}

/// Duplicate keys: a struct rejects them; a map keeps the last.
#[test]
fn duplicate_keys() {
    #[derive(Deserialize, Debug)]
    struct One { #[allow(dead_code)] a: u32 }
    assert!(serde_kv::from_str::<One>("a=1 a=2").is_err());

    let map: BTreeMap<String, String> = serde_kv::from_str("a=1 a=2").unwrap();
    assert_eq!(map["a"], "2");
}
