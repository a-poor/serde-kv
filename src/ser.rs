use serde::ser::{self, Impossible, Serialize};

use crate::error::{Error, Result};

/// Serialize a value into a key/value line.
///
/// ```
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Record {
///     foo: String,
///     baz: u64,
///     cool: bool,
/// }
///
/// let line = serde_kv::to_string(&Record { foo: "bar".into(), baz: 42, cool: true }).unwrap();
/// assert_eq!(line, "foo=bar baz=42 cool=true");
/// ```
pub fn to_string<T>(value: &T) -> Result<String>
where
    T: Serialize,
{
    let mut serializer = Serializer { out: String::new() };
    value.serialize(&mut serializer)?;
    Ok(serializer.out)
}

/// Quote and escape a value if it cannot be represented as a bare token.
///
/// This is the exact inverse of the parser's decoding: bare tokens are returned
/// verbatim, while empty strings or strings containing a space, `"`, or `\` are
/// wrapped in quotes with `"` and `\` escaped.
fn quote_if_needed(s: &str) -> String {
    let needs_quotes =
        s.is_empty() || s.contains(' ') || s.contains('"') || s.contains('\\');
    if !needs_quotes {
        return s.to_owned();
    }
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Top-level serializer: only structs and maps are accepted.
struct Serializer {
    out: String,
}

impl ser::Serializer for &mut Serializer {
    type Ok = ();
    type Error = Error;

    type SerializeStruct = Self;
    type SerializeMap = Self;

    type SerializeSeq = Impossible<(), Error>;
    type SerializeTuple = Impossible<(), Error>;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeStructVariant = Impossible<(), Error>;

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(self)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // Every scalar/aggregate at the top level is rejected.
    fn serialize_bool(self, _v: bool) -> Result<()> {
        Err(Error::TopLevelNotMap)
    }
    fn serialize_i8(self, _v: i8) -> Result<()> {
        Err(Error::TopLevelNotMap)
    }
    fn serialize_i16(self, _v: i16) -> Result<()> {
        Err(Error::TopLevelNotMap)
    }
    fn serialize_i32(self, _v: i32) -> Result<()> {
        Err(Error::TopLevelNotMap)
    }
    fn serialize_i64(self, _v: i64) -> Result<()> {
        Err(Error::TopLevelNotMap)
    }
    fn serialize_i128(self, _v: i128) -> Result<()> {
        Err(Error::TopLevelNotMap)
    }
    fn serialize_u8(self, _v: u8) -> Result<()> {
        Err(Error::TopLevelNotMap)
    }
    fn serialize_u16(self, _v: u16) -> Result<()> {
        Err(Error::TopLevelNotMap)
    }
    fn serialize_u32(self, _v: u32) -> Result<()> {
        Err(Error::TopLevelNotMap)
    }
    fn serialize_u64(self, _v: u64) -> Result<()> {
        Err(Error::TopLevelNotMap)
    }
    fn serialize_u128(self, _v: u128) -> Result<()> {
        Err(Error::TopLevelNotMap)
    }
    fn serialize_f32(self, _v: f32) -> Result<()> {
        Err(Error::TopLevelNotMap)
    }
    fn serialize_f64(self, _v: f64) -> Result<()> {
        Err(Error::TopLevelNotMap)
    }
    fn serialize_char(self, _v: char) -> Result<()> {
        Err(Error::TopLevelNotMap)
    }
    fn serialize_str(self, _v: &str) -> Result<()> {
        Err(Error::TopLevelNotMap)
    }
    fn serialize_bytes(self, _v: &[u8]) -> Result<()> {
        Err(Error::TopLevelNotMap)
    }
    fn serialize_none(self) -> Result<()> {
        Err(Error::TopLevelNotMap)
    }
    fn serialize_some<T>(self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::TopLevelNotMap)
    }
    fn serialize_unit(self) -> Result<()> {
        Err(Error::TopLevelNotMap)
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        Err(Error::TopLevelNotMap)
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        Err(Error::TopLevelNotMap)
    }
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::TopLevelNotMap)
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(Error::TopLevelNotMap)
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(Error::TopLevelNotMap)
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::TopLevelNotMap)
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::TopLevelNotMap)
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::TopLevelNotMap)
    }
}

impl Serializer {
    /// Append `key=value` to the output, inserting a space separator as needed.
    fn push_pair(&mut self, key: &str, value: &str) {
        if !self.out.is_empty() {
            self.out.push(' ');
        }
        self.out.push_str(key);
        self.out.push('=');
        self.out.push_str(value);
    }
}

impl ser::SerializeStruct for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if let Some(token) = value.serialize(ValueSerializer)? {
            self.push_pair(key, &token);
        }
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl ser::SerializeMap for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, _key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        // Keys are buffered via serialize_entry; see below.
        unreachable!("serialize_entry is used instead")
    }

    fn serialize_value<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unreachable!("serialize_entry is used instead")
    }

    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<()>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize,
    {
        let key = key.serialize(KeySerializer)?;
        if let Some(token) = value.serialize(ValueSerializer)? {
            self.push_pair(&key, &token);
        }
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

/// Serializer for a single value, producing the formatted token (or `None` to
/// skip the field, e.g. for `Option::None`).
struct ValueSerializer;

impl ser::Serializer for ValueSerializer {
    type Ok = Option<String>;
    type Error = Error;

    type SerializeSeq = Impossible<Option<String>, Error>;
    type SerializeTuple = Impossible<Option<String>, Error>;
    type SerializeTupleStruct = Impossible<Option<String>, Error>;
    type SerializeTupleVariant = Impossible<Option<String>, Error>;
    type SerializeMap = Impossible<Option<String>, Error>;
    type SerializeStruct = Impossible<Option<String>, Error>;
    type SerializeStructVariant = Impossible<Option<String>, Error>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        Ok(Some(if v { "true".into() } else { "false".into() }))
    }
    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        Ok(Some(v.to_string()))
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        Ok(Some(v.to_string()))
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        Ok(Some(v.to_string()))
    }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        Ok(Some(v.to_string()))
    }
    fn serialize_i128(self, v: i128) -> Result<Self::Ok> {
        Ok(Some(v.to_string()))
    }
    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        Ok(Some(v.to_string()))
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        Ok(Some(v.to_string()))
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        Ok(Some(v.to_string()))
    }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        Ok(Some(v.to_string()))
    }
    fn serialize_u128(self, v: u128) -> Result<Self::Ok> {
        Ok(Some(v.to_string()))
    }
    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        Ok(Some(v.to_string()))
    }
    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        Ok(Some(v.to_string()))
    }
    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        Ok(Some(quote_if_needed(&v.to_string())))
    }
    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        Ok(Some(quote_if_needed(v)))
    }
    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok> {
        Err(Error::Unsupported("bytes"))
    }
    fn serialize_none(self) -> Result<Self::Ok> {
        Ok(None)
    }
    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(None)
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        Ok(None)
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        Ok(Some(quote_if_needed(variant)))
    }
    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::Unsupported("enum"))
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(Error::Unsupported("sequence"))
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(Error::Unsupported("tuple"))
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::Unsupported("tuple struct"))
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::Unsupported("enum"))
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::Unsupported("nested map"))
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct> {
        Err(Error::Unsupported("nested struct"))
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::Unsupported("enum"))
    }
}

/// Serializer that accepts only string-like map keys.
struct KeySerializer;

impl ser::Serializer for KeySerializer {
    type Ok = String;
    type Error = Error;

    type SerializeSeq = Impossible<String, Error>;
    type SerializeTuple = Impossible<String, Error>;
    type SerializeTupleStruct = Impossible<String, Error>;
    type SerializeTupleVariant = Impossible<String, Error>;
    type SerializeMap = Impossible<String, Error>;
    type SerializeStruct = Impossible<String, Error>;
    type SerializeStructVariant = Impossible<String, Error>;

    fn serialize_str(self, v: &str) -> Result<String> {
        Ok(v.to_owned())
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<String> {
        Ok(variant.to_owned())
    }
    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<String>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_bool(self, _v: bool) -> Result<String> {
        Err(Error::Unsupported("non-string map key"))
    }
    fn serialize_i8(self, _v: i8) -> Result<String> {
        Err(Error::Unsupported("non-string map key"))
    }
    fn serialize_i16(self, _v: i16) -> Result<String> {
        Err(Error::Unsupported("non-string map key"))
    }
    fn serialize_i32(self, _v: i32) -> Result<String> {
        Err(Error::Unsupported("non-string map key"))
    }
    fn serialize_i64(self, _v: i64) -> Result<String> {
        Err(Error::Unsupported("non-string map key"))
    }
    fn serialize_i128(self, _v: i128) -> Result<String> {
        Err(Error::Unsupported("non-string map key"))
    }
    fn serialize_u8(self, _v: u8) -> Result<String> {
        Err(Error::Unsupported("non-string map key"))
    }
    fn serialize_u16(self, _v: u16) -> Result<String> {
        Err(Error::Unsupported("non-string map key"))
    }
    fn serialize_u32(self, _v: u32) -> Result<String> {
        Err(Error::Unsupported("non-string map key"))
    }
    fn serialize_u64(self, _v: u64) -> Result<String> {
        Err(Error::Unsupported("non-string map key"))
    }
    fn serialize_u128(self, _v: u128) -> Result<String> {
        Err(Error::Unsupported("non-string map key"))
    }
    fn serialize_f32(self, _v: f32) -> Result<String> {
        Err(Error::Unsupported("non-string map key"))
    }
    fn serialize_f64(self, _v: f64) -> Result<String> {
        Err(Error::Unsupported("non-string map key"))
    }
    fn serialize_char(self, v: char) -> Result<String> {
        Ok(v.to_string())
    }
    fn serialize_bytes(self, _v: &[u8]) -> Result<String> {
        Err(Error::Unsupported("non-string map key"))
    }
    fn serialize_none(self) -> Result<String> {
        Err(Error::Unsupported("non-string map key"))
    }
    fn serialize_some<T>(self, _value: &T) -> Result<String>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::Unsupported("non-string map key"))
    }
    fn serialize_unit(self) -> Result<String> {
        Err(Error::Unsupported("non-string map key"))
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<String> {
        Err(Error::Unsupported("non-string map key"))
    }
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<String>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::Unsupported("non-string map key"))
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(Error::Unsupported("non-string map key"))
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(Error::Unsupported("non-string map key"))
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::Unsupported("non-string map key"))
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::Unsupported("non-string map key"))
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::Unsupported("non-string map key"))
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct> {
        Err(Error::Unsupported("non-string map key"))
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::Unsupported("non-string map key"))
    }
}
