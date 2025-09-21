use std::collections::HashMap;
use serde::Serialize;
use crate::error::PgBouncerSerdeError;

/// Serialize a Rust value into a minimal PgBouncer-like INI string.
///
/// # Parameters
/// - value: The serializable value (typically a struct or map-like configuration).
///
/// # Returns
/// A String containing INI-like lines (with optional [section] headers and `key = value` pairs).
///
/// # Errors
/// Returns an error when a complex key or unsupported structure is encountered.
///
/// # Examples
/// ```rust
/// use pgbouncer_config_serde::ser;
/// #[derive(serde::Serialize)]
/// struct S { a: String }
/// let s = S { a: "v".into() };
/// let out = ser::to_string(&s)?;
/// # let _ = out;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Notes
/// - Suspected issue: `current_section` is never set, so no [section] headers are emitted.
/// - Keys from nested maps/structs are not combined into dotted paths; deeper levels likely overwrite `current_key_prefix`.
/// - Output order is based on `HashMap` iteration and is non-deterministic across runs.
/// - Strings with commas or equals are not escaped/quoted; lists are emitted as comma-joined values without quoting.
pub fn to_string<T>(value: &T) -> crate::error::Result<String>
where
    T: serde::Serialize
{
    let mut serializer = Serializer::new();
    value.serialize(&mut serializer)?;
    Ok(serializer.into_output())
}

/// Minimal serializer that collects keys and values per section.
///
/// # Fields
/// - output: Accumulates section -> (key -> value) pairs.
/// - current_section: Name of the section currently targeted. Not set by the current implementation.
/// - current_key_prefix: Current key being written; used for simple fields.
pub struct Serializer {
    output: HashMap<String, HashMap<String, String>>,
    current_section: String,
    current_key_prefix: String,
}

impl Serializer {
    fn new() -> Self {
        Serializer {
            output: HashMap::new(),
            current_section: String::new(),
            current_key_prefix: String::new(),
        }
    }

    fn into_output(self) -> String {
        let mut result = String::new();
        for (section_name, map) in self.output {
            if !section_name.is_empty() {
                result.push_str(&format!("[{}]\n", section_name));
            }
            for (key, value) in map {
                result.push_str(&format!("{} = {}\n", key, value));
            }
            result.push_str("\n");
        }
        result
    }
}

impl<'a> serde::ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = PgBouncerSerdeError;

    type SerializeSeq = CommaSeparated<'a>;
    type SerializeTuple = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, value: bool) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&value.to_string())
    }

    fn serialize_i8(self, value: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&value.to_string())
    }

    fn serialize_i16(self, value: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&value.to_string())
    }

    fn serialize_i32(self, value: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&value.to_string())
    }

    fn serialize_i64(self, value: i64) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&value.to_string())
    }
    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_u8(self, value: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&value.to_string())
    }

    fn serialize_u16(self, value: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&value.to_string())
    }

    fn serialize_u32(self, value: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&value.to_string())
    }

    fn serialize_u64(self, value: u64) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&value.to_string())
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_f32(self, value: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&value.to_string())
    }

    fn serialize_f64(self, value: f64) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&value.to_string())
    }

    fn serialize_char(self, value: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&value.to_string())
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
        let key = self.current_key_prefix.trim_end_matches(".").to_string();
        self.output
            .entry(self.current_section.clone())
            .or_default()
            .insert(key, value.to_string());
        Ok(())
    }

    fn serialize_bytes(self, _: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_some<T: ?Sized + Serialize>(self, _: &T) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_variant(self, _: &'static str, _: u32, _: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(self, _: &'static str, _: &T) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_newtype_variant<T>(self, _: &'static str, _: u32, _: &'static str, _: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Ok(())
    }

    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(CommaSeparated {
            serializer: self,
            buffer: String::new(),
        })
    }

    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(PgBouncerSerdeError::ComplexKey)
    }

    fn serialize_tuple_struct(self, _: &'static str, _: usize) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(PgBouncerSerdeError::ComplexKey)
    }

    fn serialize_tuple_variant(self, _: &'static str, _: u32, _: &'static str, _: usize) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(PgBouncerSerdeError::ComplexKey)
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(self)
    }

    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_struct_variant(self, _: &'static str, _: u32, _: &'static str, _: usize) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(PgBouncerSerdeError::ComplexKey)
    }
}

/// Implements map serialization by converting keys to strings and serializing values.
///
/// # Notes
/// - Suspected issue: Only a single-level `current_key_prefix` is tracked; nested maps/structs will not form dotted keys.
impl<'a> serde::ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = PgBouncerSerdeError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.current_key_prefix = to_string_key(key)?;
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.current_key_prefix.clear();
        Ok(())
    }
}

impl<'a> serde::ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = PgBouncerSerdeError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.current_key_prefix = key.to_string();
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.current_key_prefix.clear();
        Ok(())
    }
}

/// Convert a serializable key into a string for use in INI keys.
///
/// # Parameters
/// - value: Any serializable value intended to be used as a key.
///
/// # Returns
/// The string representation of the key.
///
/// # Errors
/// Returns `ComplexKey` if the value is not representable as a simple string (e.g., map, struct, seq).
///
/// # Notes
/// - Suspected issue: Lack of quoting/escaping means keys containing `.` or spaces may be ambiguous
///   when combined with nested structures.
pub fn to_string_key<T>(value: &T) -> crate::error::Result<String>
where
    T: ?Sized + Serialize,
{
    value.serialize(StrSerializer)
}

/// Internal serializer used to stringify simple values for keys.
struct StrSerializer;

impl serde::ser::Serializer for StrSerializer {
    type Ok = String;
    type Error = PgBouncerSerdeError;

    type SerializeSeq = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeMap = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }

    fn serialize_bytes(self, _: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(PgBouncerSerdeError::ComplexKey)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(PgBouncerSerdeError::ComplexKey)
    }

    fn serialize_some<T>(self, _: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize
    {
        Err(PgBouncerSerdeError::ComplexKey)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(PgBouncerSerdeError::ComplexKey)
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(PgBouncerSerdeError::ComplexKey)
    }

    fn serialize_unit_variant(self, _: &'static str, _: u32, _: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(PgBouncerSerdeError::ComplexKey)
    }

    fn serialize_newtype_struct<T>(self, _: &'static str, _: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize
    {
        Err(PgBouncerSerdeError::ComplexKey)
    }

    fn serialize_newtype_variant<T>(self, _: &'static str, _: u32, _: &'static str, _: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize
    {
        Err(PgBouncerSerdeError::ComplexKey)
    }

    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(PgBouncerSerdeError::ComplexKey)
    }

    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(PgBouncerSerdeError::ComplexKey)
    }

    fn serialize_tuple_struct(self, _: &'static str, _: usize) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(PgBouncerSerdeError::ComplexKey)
    }

    fn serialize_tuple_variant(self, _: &'static str, _: u32, _: &'static str, _: usize) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(PgBouncerSerdeError::ComplexKey)
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(PgBouncerSerdeError::ComplexKey)
    }

    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Self::SerializeStruct, Self::Error> {
        Err(PgBouncerSerdeError::ComplexKey)
    }

    fn serialize_struct_variant(self, _: &'static str, _: u32, _: &'static str, _: usize) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(PgBouncerSerdeError::ComplexKey)
    }
}

/// Helper to emit comma-separated lists for sequences.
///
/// # Fields
/// - serializer: Reference to the outer serializer used to write back the final value.
/// - buffer: Accumulates the comma-joined representation.
pub struct CommaSeparated<'a> {
    serializer: &'a mut Serializer,
    buffer: String,
}

impl<'a> serde::ser::SerializeSeq for CommaSeparated<'a> {
    type Ok = ();
    type Error = PgBouncerSerdeError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        if !self.buffer.is_empty() {
            self.buffer.push(',');
        }

        let s = value.serialize(StrSerializer)?;
        self.buffer.push_str(&s);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let key = self.serializer.current_key_prefix.trim_end_matches('.').to_string();
        self.serializer.output
            .entry(self.serializer.current_section.clone())
            .or_default()
            .insert(key, self.buffer);

        Ok(())
    }
}