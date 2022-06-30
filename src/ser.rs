use crate::{Cell, Row};
use anyhow::anyhow;
use serde::{ser, Serialize};
use std::fmt;

type Result<T, E = Error> = std::result::Result<T, E>;
pub struct Error(anyhow::Error);

impl Error {
    fn from(e: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self(anyhow::Error::from(e))
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Error(anyhow!("{}", msg))
    }
}

pub fn serialize_row(value: impl Serialize) -> Result<Row<'static>> {
    let mut serializer = Serializer {
        row: Row::new(),
        level: 0,
    };
    value.serialize(&mut serializer)?;
    Ok(serializer.row)
}

pub struct Serializer {
    row: Row<'static>,
    level: usize,
}

impl Serializer {
    fn serialize_static_str(&mut self, s: &'static str) -> Result<()> {
        self.row.add_cell(Cell::from(s));
        Ok(())
    }

    fn serialize_string(&mut self, s: String) -> Result<()> {
        self.row.add_cell(Cell::from(s));
        Ok(())
    }
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = SerializerSeq<'a>;
    type SerializeTuple = SerializerSeq<'a>;
    type SerializeTupleStruct = SerializerSeq<'a>;
    type SerializeTupleVariant = SerializerSeq<'a>;
    type SerializeMap = SerializerSeq<'a>;
    type SerializeStruct = SerializerSeq<'a>;
    type SerializeStructVariant = SerializerSeq<'a>;

    // Here we go with the simple methods. The following 12 methods receive one
    // of the primitive types of the data model and map it to JSON by appending
    // into the output string.
    fn serialize_bool(self, v: bool) -> Result<()> {
        self.serialize_static_str(if v { "true" } else { "false" })
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_string(v.to_string())
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_string(v.to_string())
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_string(v.to_string())
    }

    // Not particularly efficient but this is example code anyway. A more
    // performant approach would be to use the `itoa` crate.
    fn serialize_i64(self, v: i64) -> Result<()> {
        self.serialize_string(v.to_string())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_string(v.to_string())
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_string(v.to_string())
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_string(v.to_string())
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.serialize_string(v.to_string())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.serialize_string(v.to_string())
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.serialize_string(v.to_string())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_string(v.to_string())
    }

    // We can't rely on this being a static string
    fn serialize_str(self, v: &str) -> Result<()> {
        self.serialize_string(v.to_string())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        let v = format!("{:?}", v);
        self.serialize_string(v)
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        // Don't serialize anything
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_static_str(variant)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let value = serde_json::to_string(value).map_err(|e| Error(e.into()))?;
        let output = format!("{}: {}", variant, value);
        self.serialize_string(output)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SerializerSeq::new("", BracketTy::Square, self))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(SerializerSeq::new("", BracketTy::Round, self))
    }

    // Tuple structs look just like sequences in JSON.
    fn serialize_tuple_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(SerializerSeq::new(name, BracketTy::Round, self))
    }

    // Tuple variants are represented in JSON as `{ NAME: [DATA...] }`. Again
    // this method is only responsible for the externally tagged representation.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(SerializerSeq::new(variant, BracketTy::Round, self))
    }

    // Maps are represented in JSON as `{ K: V, K: V, ... }`.
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(SerializerSeq::new("", BracketTy::Curly, self))
    }

    // Structs look just like maps in JSON. In particular, JSON requires that we
    // serialize the field names of the struct. Other formats may be able to
    // omit the field names when serializing structs because the corresponding
    // Deserialize implementation is required to know what the keys are without
    // looking at the serialized data.
    fn serialize_struct(self, name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(SerializerSeq::new(name, BracketTy::Curly, self))
    }

    // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }`.
    // This is the externally tagged representation.
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(SerializerSeq::new(variant, BracketTy::Curly, self))
    }
}

#[derive(Copy, Clone)]
enum BracketTy {
    Round,
    Curly,
    Square,
}

impl BracketTy {
    fn start(self) -> char {
        use BracketTy::*;
        match self {
            Round => '(',
            Curly => '{',
            Square => '[',
        }
    }

    fn end(self) -> char {
        use BracketTy::*;
        match self {
            Round => ')',
            Curly => '}',
            Square => ']',
        }
    }
}

pub struct SerializerSeq<'a> {
    ty: BracketTy,
    parent: &'a mut Serializer,
    output: String,
}

impl<'a> SerializerSeq<'a> {
    fn new(prefix: &'static str, ty: BracketTy, parent: &'a mut Serializer) -> Self {
        parent.level += 1;
        Self {
            ty,
            parent,
            output: if prefix.is_empty() {
                format!("{}", ty.start())
            } else {
                format!("{} {}", prefix, ty.start())
            },
        }
    }

    fn serialize_element<T>(&mut self, key: Option<&'static str>, value: &T) -> Result<()>
    where
        T: Serialize + ?Sized,
    {
        if self.parent.level > 1 {
            if !self.output.ends_with(self.ty.start()) {
                self.output += ", ";
            }
            if let Some(key) = key {
                self.output += &serde_json::to_string(key).map_err(Error::from)?;
                self.output += ": ";
            }
            self.output += &serde_json::to_string(value).map_err(Error::from)?;
        } else {
            value.serialize(&mut *self.parent)?;
        }
        Ok(())
    }

    fn end(mut self) -> Result<()> {
        self.parent.level -= 1;
        if self.parent.level > 0 {
            self.output.push(self.ty.end());
            self.parent.serialize_string(self.output)?;
        }
        Ok(())
    }
}

impl<'a> ser::SerializeSeq for SerializerSeq<'a> {
    type Ok = ();
    type Error = Error;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_element(None, value)
    }

    // Close the sequence.
    fn end(self) -> Result<()> {
        self.end()
    }
}

impl<'a> ser::SerializeTuple for SerializerSeq<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_element(None, value)
    }

    fn end(self) -> Result<()> {
        self.end()
    }
}

impl<'a> ser::SerializeTupleStruct for SerializerSeq<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_element(None, value)
    }

    fn end(self) -> Result<()> {
        self.end()
    }
}

impl<'a> ser::SerializeTupleVariant for SerializerSeq<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_element(None, value)
    }

    fn end(self) -> Result<()> {
        self.end()
    }
}

impl<'a> ser::SerializeMap for SerializerSeq<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if self.parent.level > 1 {
            if !self.output.ends_with(self.ty.start()) {
                self.output += ", ";
            }
            self.output += &serde_json::to_string(key).map_err(Error::from)?;
            self.output += ": ";
        } else {
            // serialize nothing
        }
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if self.parent.level > 1 {
            self.output += &serde_json::to_string(value).map_err(Error::from)?;
        } else {
            value.serialize(&mut *self.parent)?;
        }
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.end()
    }
}

// Structs are like maps in which the keys are constrained to be compile-time
// constant strings.
impl<'a> ser::SerializeStruct for SerializerSeq<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_element(Some(key), value)
    }

    fn end(self) -> Result<()> {
        self.end()
    }
}

// Similar to `SerializeTupleVariant`, here the `end` method is responsible for
// closing both of the curly braces opened by `serialize_struct_variant`.
impl<'a> ser::SerializeStructVariant for SerializerSeq<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_element(Some(key), value)
    }

    fn end(self) -> Result<()> {
        self.end()
    }
}
