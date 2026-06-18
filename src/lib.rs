//! A [serde](https://serde.rs) data format for flat key/value lines.
//!
//! The format is a space-separated list of `key=value` pairs:
//!
//! ```text
//! foo=bar baz=42 cool=true message="hello world"
//! ```
//!
//! There is **no type inference at the format level**: a raw token like `42` is
//! interpreted according to the target field's type. A `String` field receives
//! `"42"`, while a `u64` field receives `42`. Quotes are only a transport
//! concern (needed when a value contains spaces or special characters), so
//! `foo=42` and `foo="42"` decode to the identical value.
//!
//! ```
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize, PartialEq, Debug)]
//! struct Record {
//!     foo: String,
//!     baz: u64,
//!     cool: bool,
//!     message: String,
//! }
//!
//! let line = r#"foo=bar baz=42 cool=true message="hello world""#;
//! let record: Record = serde_kv::from_str(line).unwrap();
//! assert_eq!(record.baz, 42);
//! assert_eq!(serde_kv::to_string(&record).unwrap(), line);
//! ```
//!
//! Scope: flat scalars (strings, integers, floats, `bool`, `char`), `Option<T>`
//! (a missing key deserializes to `None`; `None` is skipped on serialization),
//! and tolerant handling of unknown keys. Nested structs and sequences are not
//! supported.

mod de;
mod error;
mod ser;

pub use de::from_str;
pub use error::{Error, Result};
pub use ser::to_string;

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use crate::{Error, from_str, to_string};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Record {
        foo: String,
        baz: u64,
        cool: bool,
        message: String,
    }

    #[test]
    fn basic_deserialize() {
        let line = r#"foo=bar baz=42 cool=true message="hello world""#;
        let got: Record = from_str(line).unwrap();
        assert_eq!(
            got,
            Record {
                foo: "bar".into(),
                baz: 42,
                cool: true,
                message: "hello world".into(),
            }
        );
    }

    #[test]
    fn round_trip_is_canonical() {
        let record = Record {
            foo: "bar".into(),
            baz: 42,
            cool: true,
            message: "hello world".into(),
        };
        let line = to_string(&record).unwrap();
        assert_eq!(line, r#"foo=bar baz=42 cool=true message="hello world""#);
        assert_eq!(from_str::<Record>(&line).unwrap(), record);
    }

    #[test]
    fn type_decides_interpretation() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct AsString {
            x: String,
        }
        #[derive(Deserialize, PartialEq, Debug)]
        struct AsInt {
            x: u64,
        }

        assert_eq!(from_str::<AsString>("x=42").unwrap().x, "42");
        assert_eq!(from_str::<AsInt>("x=42").unwrap().x, 42);
        // Quotes do not change the type interpretation.
        assert_eq!(from_str::<AsInt>(r#"x="42""#).unwrap().x, 42);
        assert_eq!(from_str::<AsString>(r#"x="42""#).unwrap().x, "42");
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct WithOption {
        a: u32,
        b: Option<String>,
    }

    #[test]
    fn option_present_and_missing() {
        assert_eq!(
            from_str::<WithOption>("a=1 b=hi").unwrap(),
            WithOption { a: 1, b: Some("hi".into()) }
        );
        assert_eq!(
            from_str::<WithOption>("a=1").unwrap(),
            WithOption { a: 1, b: None }
        );
    }

    #[test]
    fn option_serialization_skips_none() {
        assert_eq!(to_string(&WithOption { a: 1, b: None }).unwrap(), "a=1");
        assert_eq!(
            to_string(&WithOption { a: 1, b: Some("x".into()) }).unwrap(),
            "a=1 b=x"
        );
    }

    #[test]
    fn unknown_keys_are_ignored() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct OnlyA {
            a: u32,
        }
        let got: OnlyA = from_str(r#"a=1 extra=zzz junk="a b""#).unwrap();
        assert_eq!(got, OnlyA { a: 1 });
    }

    #[test]
    fn quoting_and_escaping_round_trip() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct M {
            m: String,
        }
        let input = r#"m="he said \"hi\" \\ ok""#;
        let got: M = from_str(input).unwrap();
        assert_eq!(got.m, r#"he said "hi" \ ok"#);
        assert_eq!(to_string(&got).unwrap(), input);
    }

    #[test]
    fn empty_value_round_trips() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct E {
            a: String,
        }
        assert_eq!(from_str::<E>("a=").unwrap().a, "");
        assert_eq!(from_str::<E>(r#"a="""#).unwrap().a, "");
        assert_eq!(to_string(&E { a: String::new() }).unwrap(), r#"a="""#);
    }

    #[test]
    fn char_and_numbers() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct C {
            c: char,
            n: i32,
            f: f64,
        }
        let got: C = from_str("c=x n=-5 f=2.5").unwrap();
        assert_eq!(got, C { c: 'x', n: -5, f: 2.5 });
    }

    #[test]
    fn error_cases() {
        #[derive(Deserialize, Debug)]
        struct N {
            #[allow(dead_code)]
            n: u64,
        }
        #[derive(Deserialize, Debug)]
        struct B {
            #[allow(dead_code)]
            cool: bool,
        }
        #[derive(Deserialize, Debug)]
        struct Ch {
            #[allow(dead_code)]
            c: char,
        }

        assert_eq!(from_str::<N>("n=abc").unwrap_err(), Error::ParseInt("abc".into()));
        assert_eq!(
            from_str::<B>("cool=yes").unwrap_err(),
            Error::ParseBool("yes".into())
        );
        assert_eq!(
            from_str::<Ch>("c=xy").unwrap_err(),
            Error::ParseChar("xy".into())
        );
        assert_eq!(from_str::<N>("n").unwrap_err(), Error::ExpectedEquals);
    }

    #[test]
    fn top_level_guard() {
        assert_eq!(from_str::<u64>("a=1").unwrap_err(), Error::TopLevelNotMap);
        assert_eq!(to_string(&5u64).unwrap_err(), Error::TopLevelNotMap);
    }
}
