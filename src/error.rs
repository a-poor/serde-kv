use std::fmt::{self, Display};

/// The error type for serializing and deserializing the key/value format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// A custom message produced by serde (missing field, unknown variant, etc.).
    Message(String),
    /// A token was missing its `=` separator.
    ExpectedEquals,
    /// A quoted value opened with `"` but never closed.
    UnterminatedQuote,
    /// A backslash was followed by something other than `"` or `\`.
    InvalidEscape(char),
    /// A token could not be parsed as the requested integer type.
    ParseInt(String),
    /// A token could not be parsed as the requested float type.
    ParseFloat(String),
    /// A token was not `true` or `false` for a bool field.
    ParseBool(String),
    /// A token was not exactly one character for a char field.
    ParseChar(String),
    /// A scalar/sequence was (de)serialized at the top level; only structs/maps are allowed.
    TopLevelNotMap,
    /// A nested struct, sequence, map, or enum value was encountered (unsupported).
    Unsupported(&'static str),
}

/// A specialized [`Result`] for this crate.
pub type Result<T> = std::result::Result<T, Error>;

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Message(msg) => f.write_str(msg),
            Error::ExpectedEquals => f.write_str("expected `=` separator in key/value pair"),
            Error::UnterminatedQuote => f.write_str("unterminated quoted value"),
            Error::InvalidEscape(c) => write!(f, "invalid escape sequence `\\{c}`"),
            Error::ParseInt(t) => write!(f, "invalid integer `{t}`"),
            Error::ParseFloat(t) => write!(f, "invalid float `{t}`"),
            Error::ParseBool(t) => write!(f, "invalid bool `{t}` (expected `true` or `false`)"),
            Error::ParseChar(t) => write!(f, "invalid char `{t}` (expected a single character)"),
            Error::TopLevelNotMap => {
                f.write_str("top level value must be a struct or map of key/value pairs")
            }
            Error::Unsupported(what) => write!(f, "unsupported value type: {what}"),
        }
    }
}

impl std::error::Error for Error {}

impl serde::ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl serde::de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}
