use std::error::Error;

use serde::{
    ser::{
        SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant,
    },
    Serialize,
};

use crate::{
    needed_bytes_usize,
    types::{LeadByte, NormalRionType, RionFieldType, ShortRionType},
    RionField,
};

pub struct Serializer {
    output: Vec<u8>,
}

impl Default for Serializer {
    fn default() -> Self {
        Self::new()
    }
}

impl Serializer {
    pub fn new() -> Self {
        Self { output: Vec::new() }
    }

    pub fn serialize_key(&mut self, key: &[u8]) -> Result<(), SerializeError> {
        let field = RionField::key(key);
        field.encode(&mut self.output).unwrap();
        Ok(())
    }

    pub fn serialize_entry<T: ?Sized + Serialize>(
        &mut self,
        key: &str,
        value: &T,
    ) -> Result<(), SerializeError> {
        let mut sized = SizedSerializer::new(self);
        sized.serialize_key(key)?;
        value.serialize(&mut sized.temp)?;
        sized.finish(0xC)
    }
}

pub struct SizedSerializer<'a> {
    output: &'a mut Serializer,
    temp: Serializer,
}

pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>, SerializeError>
where
    T: RionSerialize,
{
    let mut serializer = Serializer::new();
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

pub trait RionSerialize {
    fn serialize(&self, serializer: &mut Serializer) -> Result<(), SerializeError>;
}

#[cfg(feature = "specialization")]
impl<T: Serialize> RionSerialize for T {
    default fn serialize(&self, serializer: &mut Serializer) -> Result<(), SerializeError> {
        self.serialize(serializer)
    }
}
#[cfg(not(feature = "specialization"))]
impl<T: Serialize> RionSerialize for T {
    fn serialize(&self, serializer: &mut Serializer) -> Result<(), SerializeError> {
        self.serialize(serializer)
    }
}

#[cfg(feature = "specialization")]
macro_rules! impl_rion_serialize_const_array {
  ($($len:expr), +) => {
      $(
        impl RionSerialize for [u8; $len] {
            fn serialize(&self, serializer: &mut Serializer) -> Result<(), SerializeError> {
                // println!("Serializing array of length {}", $len);
                let bytes = RionField::bytes(self);
                bytes.encode(&mut serializer.output).unwrap();
                Ok(())
            }
        }
      )+
    };
}

#[cfg(feature = "specialization")]
impl_rion_serialize_const_array!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);

#[cfg(feature = "specialization")]
impl RionSerialize for &[u8] {
    fn serialize(&self, serializer: &mut Serializer) -> Result<(), SerializeError> {
        let bytes = RionField::bytes(self);
        bytes.encode(&mut serializer.output).unwrap();
        Ok(())
    }
}

#[cfg(feature = "specialization")]
impl RionSerialize for Vec<u8> {
    fn serialize(&self, serializer: &mut Serializer) -> Result<(), SerializeError> {
        let bytes = RionField::bytes(self);
        bytes.encode(&mut serializer.output).unwrap();
        Ok(())
    }
}

impl std::fmt::Display for SerializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SerializeError::Custom(msg) => write!(f, "{}", msg),
            SerializeError::InvalidType(rion_field_type) => {
                write!(f, "Invalid type: {:?}", rion_field_type)
            }
            SerializeError::LengthOverflow(len) => {
                write!(f, "Length overflow: {}", len)
            }
            SerializeError::IoError(err) => write!(f, "IO Error: {}", err),
        }
    }
}
impl Error for SerializeError {}
impl serde::ser::Error for SerializeError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        SerializeError::Custom(msg.to_string())
    }
}
impl From<Box<dyn Error>> for SerializeError {
    fn from(err: Box<dyn Error>) -> Self {
        SerializeError::Custom(err.to_string())
    }
}
impl From<std::io::Error> for SerializeError {
    fn from(err: std::io::Error) -> Self {
        SerializeError::IoError(err)
    }
}

#[derive(Debug)]
pub enum SerializeError {
    Custom(String),
    InvalidType(RionFieldType),
    LengthOverflow(usize),
    IoError(std::io::Error),
}

impl<'a> serde::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = SerializeError;
    type SerializeSeq = SizedSerializer<'a>;
    type SerializeTuple = SizedSerializer<'a>;
    type SerializeTupleStruct = SizedSerializer<'a>;
    type SerializeTupleVariant = SizedSerializer<'a>;
    type SerializeMap = SizedSerializer<'a>;
    type SerializeStruct = SizedSerializer<'a>;
    type SerializeStructVariant = SizedSerializer<'a>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        let field = RionField::bool(v);
        field.encode(&mut self.output).unwrap();
        Ok(())
    }
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        let field = RionField::int64(v);
        field.encode(&mut self.output).unwrap();
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        let field = RionField::uint64(v);
        field.encode(&mut self.output).unwrap();
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        let field = RionField::f32(v);
        field.encode(&mut self.output).unwrap();
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        let field = RionField::f64(v);
        field.encode(&mut self.output).unwrap();
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        let field = RionField::from_str(v);
        field.encode(&mut self.output).unwrap();
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let field = RionField::bytes(v);
        field.encode(&mut self.output).unwrap();
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.output.push(0x00); // Null Bytes field
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        // let mut sized = SizedSerializer {
        //     output: self,
        //     temp: Serializer::new(),
        // };
        // sized.serialize_key(variant)?;
        // value.serialize(&mut sized.temp)?;
        // sized.finish(0xC)
        self.serialize_entry(variant, value)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SizedSerializer::new(self))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    // todo this is not correct
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        let mut sized = SizedSerializer::new(self);
        sized.serialize_key(variant)?;
        Ok(sized)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(SizedSerializer::new(self))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    // Todo this is not correct, does not handle key
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.serialize_map(Some(len))
    }
}

impl<'a> SerializeTuple for SizedSerializer<'a> {
    type Ok = ();
    type Error = SerializeError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut self.temp)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        // Array type serialization
        self.finish(0xA)
    }
}

impl<'a> SerializeSeq for SizedSerializer<'a> {
    type Ok = ();
    type Error = SerializeError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut self.temp)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // Array type serialization
        self.finish(0xA)
    }
}

impl SerializeMap for SizedSerializer<'_> {
    type Ok = ();
    type Error = SerializeError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        // key.serialize(&mut self.temp)
        let initial_len = self.temp.output.len();
        key.serialize(&mut self.temp)?;
        assert!(self.temp.output.len() > initial_len);
        let lead = self.temp.output[initial_len]; // Guaranteed to have at least one byte written
        let lead_byte = LeadByte::try_from(lead)?;
        // If the first byte is not a Key field, throw an error
        let ft = lead_byte.field_type();
        let target = &mut self.temp.output[initial_len];
        match ft {
            ft if ft.is_key() => {}
            RionFieldType::Normal(NormalRionType::UTF8) => {
                *target &= 0x0F;
                *target |= NormalRionType::Key.to_byte() << 4;
            }
            RionFieldType::Short(ShortRionType::UTF8) => {
                *target &= 0x0F;
                *target |= ShortRionType::Key.to_byte() << 4;
            }
            _ => return Err(SerializeError::InvalidType(ft)),
        }
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut self.temp)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // Object type serialization
        self.finish(0xC)
    }
}

impl<'a> SizedSerializer<'a> {
    fn new(output: &'a mut Serializer) -> Self {
        Self {
            output,
            temp: Serializer::new(),
        }
    }

    fn finish(self, type_byte: u8) -> Result<(), SerializeError> {
        let total_len = self.temp.output.len();
        let length_length = needed_bytes_usize(total_len);
        if length_length > 15 {
            return Err(SerializeError::LengthOverflow(length_length)); // TODO handle error
        }
        self.output
            .output
            .push(type_byte << 4 | length_length as u8);
        let ll = total_len as u64;
        let orig = self.output.output.len();
        crate::int_to_bytes(&ll, &mut self.output.output)?;
        assert_eq!(self.output.output.len() - orig, length_length);
        self.output.output.extend(self.temp.output);
        Ok(())
    }
}

impl SerializeTupleStruct for SizedSerializer<'_> {
    type Ok = ();
    type Error = SerializeError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut self.temp)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.finish(0xA)
    }
}

impl SerializeStruct for SizedSerializer<'_> {
    type Ok = ();
    type Error = SerializeError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        // let key = RionField::key(key.as_bytes());
        // key.encode(&mut self.temp.output).unwrap();
        self.serialize_key(key)?;
        value.serialize(&mut self.temp)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.finish(0xC)
    }
}

impl SerializeStructVariant for SizedSerializer<'_> {
    type Ok = ();
    type Error = SerializeError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.serialize_key(key)?;
        value.serialize(&mut self.temp)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.finish(0xC)
    }
}

impl SerializeTupleVariant for SizedSerializer<'_> {
    type Ok = ();
    type Error = SerializeError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut self.temp)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.finish(0xA)
    }
}
