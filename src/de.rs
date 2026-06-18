use serde::de::{
    self, Deserialize, DeserializeSeed, IntoDeserializer, MapAccess, Visitor,
};

use crate::error::{Error, Result};

/// Deserialize a value of type `T` from a key/value line.
///
/// ```
/// use serde::Deserialize;
///
/// #[derive(Deserialize, PartialEq, Debug)]
/// struct Record {
///     foo: String,
///     baz: u64,
///     cool: bool,
/// }
///
/// let got: Record = serde_kv::from_str("foo=bar baz=42 cool=true").unwrap();
/// assert_eq!(got, Record { foo: "bar".into(), baz: 42, cool: true });
/// ```
pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    let pairs = parse_line(s)?;
    T::deserialize(Deserializer { pairs })
}

/// Split a key/value line into decoded `(key, value)` pairs.
///
/// Bare values run until the next space; quoted values may contain spaces and
/// support `\"` and `\\` escapes. Escapes are decoded here so the rest of the
/// deserializer works on clean owned strings.
fn parse_line(input: &str) -> Result<Vec<(String, String)>> {
    let mut pairs = Vec::new();
    let mut chars = input.chars().peekable();

    loop {
        // Skip whitespace separating pairs.
        while let Some(&c) = chars.peek() {
            if c == ' ' {
                chars.next();
            } else {
                break;
            }
        }
        if chars.peek().is_none() {
            break;
        }

        // Read the key up to `=`.
        let mut key = String::new();
        loop {
            match chars.next() {
                Some('=') => break,
                Some(c) => key.push(c),
                None => return Err(Error::ExpectedEquals),
            }
        }

        // Read the value: quoted or bare.
        let mut value = String::new();
        if chars.peek() == Some(&'"') {
            chars.next(); // opening quote
            loop {
                match chars.next() {
                    Some('"') => break,
                    Some('\\') => match chars.next() {
                        Some('"') => value.push('"'),
                        Some('\\') => value.push('\\'),
                        Some(c) => return Err(Error::InvalidEscape(c)),
                        None => return Err(Error::UnterminatedQuote),
                    },
                    Some(c) => value.push(c),
                    None => return Err(Error::UnterminatedQuote),
                }
            }
        } else {
            while let Some(&c) = chars.peek() {
                if c == ' ' {
                    break;
                }
                value.push(c);
                chars.next();
            }
        }

        pairs.push((key, value));
    }

    Ok(pairs)
}

/// Top-level deserializer: the whole line is a map of key/value pairs.
struct Deserializer {
    pairs: Vec<(String, String)>,
}

/// Generate top-level `deserialize_*` methods that reject scalar/sequence
/// targets with [`Error::TopLevelNotMap`].
macro_rules! top_level_not_map {
    ($($method:ident)*) => {
        $(
            fn $method<V>(self, _visitor: V) -> Result<V::Value>
            where
                V: Visitor<'de>,
            {
                Err(Error::TopLevelNotMap)
            }
        )*
    };
}

impl<'de> de::Deserializer<'de> for Deserializer {
    type Error = Error;

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(KvMap {
            iter: self.pairs.into_iter(),
            value: None,
        })
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // The top level is always a map of pairs.
        self.deserialize_map(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    // A whole line cannot become a scalar/sequence: reject these directly so the
    // caller gets a clear `TopLevelNotMap` rather than a visitor type mismatch.
    top_level_not_map! {
        deserialize_bool deserialize_i8 deserialize_i16 deserialize_i32
        deserialize_i64 deserialize_i128 deserialize_u8 deserialize_u16
        deserialize_u32 deserialize_u64 deserialize_u128 deserialize_f32
        deserialize_f64 deserialize_char deserialize_str deserialize_string
        deserialize_bytes deserialize_byte_buf deserialize_option deserialize_unit
        deserialize_identifier
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::TopLevelNotMap)
    }

    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::TopLevelNotMap)
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::TopLevelNotMap)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::TopLevelNotMap)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::TopLevelNotMap)
    }
}

/// `MapAccess` over the parsed pairs.
struct KvMap {
    iter: std::vec::IntoIter<(String, String)>,
    value: Option<String>,
}

impl<'de> MapAccess<'de> for KvMap {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                // Keys are always strings/identifiers.
                seed.deserialize(key.into_deserializer()).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        let token = self
            .value
            .take()
            .expect("next_value_seed called before next_key_seed");
        seed.deserialize(ValueDeserializer { token })
    }
}

/// Deserializer for a single decoded value token.
///
/// The token is interpreted according to the *requested* type: typed methods
/// (`deserialize_u64`, `deserialize_bool`, ...) parse it, while `deserialize_any`
/// and `deserialize_str` fall back to the raw string. This is what makes
/// `baz=42` become `"42"` for a `String` field and `42` for a `u64` field.
struct ValueDeserializer {
    token: String,
}

/// Generate `deserialize_<int/float>` methods that parse the token.
macro_rules! deserialize_parsed {
    ($($method:ident => $visit:ident : $ty:ty, $err:ident);* $(;)?) => {
        $(
            fn $method<V>(self, visitor: V) -> Result<V::Value>
            where
                V: Visitor<'de>,
            {
                let n: $ty = self
                    .token
                    .parse()
                    .map_err(|_| Error::$err(self.token.clone()))?;
                visitor.$visit(n)
            }
        )*
    };
}

impl<'de> de::Deserializer<'de> for ValueDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(&self.token)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // Tolerate unknown keys by treating their value as a string.
        self.deserialize_any(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(&self.token)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.token)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.token.as_str() {
            "true" => visitor.visit_bool(true),
            "false" => visitor.visit_bool(false),
            _ => Err(Error::ParseBool(self.token)),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let mut chars = self.token.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => visitor.visit_char(c),
            _ => Err(Error::ParseChar(self.token)),
        }
    }

    deserialize_parsed! {
        deserialize_i8   => visit_i8   : i8,   ParseInt;
        deserialize_i16  => visit_i16  : i16,  ParseInt;
        deserialize_i32  => visit_i32  : i32,  ParseInt;
        deserialize_i64  => visit_i64  : i64,  ParseInt;
        deserialize_i128 => visit_i128 : i128, ParseInt;
        deserialize_u8   => visit_u8   : u8,   ParseInt;
        deserialize_u16  => visit_u16  : u16,  ParseInt;
        deserialize_u32  => visit_u32  : u32,  ParseInt;
        deserialize_u64  => visit_u64  : u64,  ParseInt;
        deserialize_u128 => visit_u128 : u128, ParseInt;
        deserialize_f32  => visit_f32  : f32,  ParseFloat;
        deserialize_f64  => visit_f64  : f64,  ParseFloat;
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // A present key is always `Some`; missing keys never reach us (the struct
        // derive supplies `None` for absent `Option` fields).
        visitor.visit_some(self)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    // Nested/aggregate values are out of scope.
    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::Unsupported("sequence"))
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::Unsupported("tuple"))
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::Unsupported("tuple struct"))
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::Unsupported("nested map"))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::Unsupported("nested struct"))
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::Unsupported("enum"))
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::Unsupported("bytes"))
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::Unsupported("bytes"))
    }
}

#[cfg(test)]
mod tests {
    use super::parse_line;

    #[test]
    fn parses_bare_and_quoted() {
        let pairs = parse_line(r#"foo=bar baz=42 message="hello world""#).unwrap();
        assert_eq!(
            pairs,
            vec![
                ("foo".to_string(), "bar".to_string()),
                ("baz".to_string(), "42".to_string()),
                ("message".to_string(), "hello world".to_string()),
            ]
        );
    }

    #[test]
    fn parses_empty_and_escapes() {
        let pairs = parse_line(r#"a= b="he said \"hi\" \\ ok""#).unwrap();
        assert_eq!(
            pairs,
            vec![
                ("a".to_string(), String::new()),
                ("b".to_string(), r#"he said "hi" \ ok"#.to_string()),
            ]
        );
    }

    #[test]
    fn parse_errors() {
        use crate::error::Error;
        assert_eq!(parse_line("foo"), Err(Error::ExpectedEquals));
        assert_eq!(parse_line(r#"a="oops"#), Err(Error::UnterminatedQuote));
        assert_eq!(parse_line(r#"a="\x""#), Err(Error::InvalidEscape('x')));
    }
}
