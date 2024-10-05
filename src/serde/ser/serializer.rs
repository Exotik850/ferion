use core::panic;
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
    pub(crate) output: Vec<u8>,
    pub(crate) stack: Vec<usize>, // Positions of object starts
}

impl Default for Serializer {
    fn default() -> Self {
        Self::new()
    }
}

impl Serializer {
    pub fn new() -> Self {
        Self {
            output: Vec::new(),
            stack: Vec::new(),
        }
    }

    pub fn serialize_key(&mut self, key: &[u8]) -> Result<(), SerializeError> {
        let field = RionField::key(key);
        Ok(field.encode(&mut self.output)?)
    }

    fn start_container(&mut self, type_byte: u8) {
        self.stack.push(self.output.len());
        self.output.extend(&[type_byte << 4, 0]);
    }

    fn end_container(&mut self) {
        let Some(start) = self.stack.pop() else {
            panic!("No object to end");
        };
        let end = self.output.len();
        let len = end - start - 2; // Subtract the type byte and the length byte
        let length_length = needed_bytes_usize(len);
        if length_length == 0 {
            self.output.remove(start + 1);
            return;
        }
        self.output[start] |= length_length as u8;
        if length_length > 1 {
            // Have to resize the vec to make room for the length
            self.output.splice(
                start + 1..start + 1,
                std::iter::repeat(0).take(length_length - 1),
            );
        }
        crate::int_to_bytes(
            &(len as u64),
            &mut &mut self.output[start + 1..start + length_length + 1],
        )
        .expect("Failed to write length");
    }

    pub fn serialize_entry<T: ?Sized + Serialize>(
        &mut self,
        key: &str,
        value: &T,
    ) -> Result<(), SerializeError> {
        self.start_container(RionFieldType::OBJECT);
        self.serialize_key(key.as_bytes())?;
        value.serialize(&mut *self)?;
        self.end_container();
        Ok(())
        // let mut sized = SizedSerializer::new(self);
        // sized.serialize_key(key)?;
        // value.serialize(&mut sized.temp)?;
        // sized.finish(0xC)
    }
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
                let bytes = RionField::bytes(self);
                Ok(bytes.encode(&mut serializer.output)?)
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
        Ok(bytes.encode(&mut serializer.output)?)
    }
}

#[cfg(feature = "specialization")]
impl RionSerialize for Vec<u8> {
    fn serialize(&self, serializer: &mut Serializer) -> Result<(), SerializeError> {
        let bytes = RionField::bytes(self);
        Ok(bytes.encode(&mut serializer.output)?)
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
    // type SerializeSeq = SizedSerializer<'a>;
    // type SerializeTuple = SizedSerializer<'a>;
    // type SerializeTupleStruct = SizedSerializer<'a>;
    // type SerializeTupleVariant = SizedSerializer<'a>;
    // type SerializeMap = SizedSerializer<'a>;
    // type SerializeStruct = SizedSerializer<'a>;
    // type SerializeStructVariant = SizedSerializer<'a>;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        let field = RionField::bool(v);
        Ok(field.encode(&mut self.output)?)
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
        Ok(field.encode(&mut self.output)?)
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
        Ok(field.encode(&mut self.output)?)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        let field = RionField::f32(v);
        Ok(field.encode(&mut self.output)?)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        let field = RionField::f64(v);
        Ok(field.encode(&mut self.output)?)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        let field = RionField::from_str(v);
        Ok(field.encode(&mut self.output)?)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let field = RionField::bytes(v);
        Ok(field.encode(&mut self.output)?)
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
        self.serialize_entry(variant, value)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        // Ok(SizedSerializer::new(self))
        self.start_container(RionFieldType::ARRAY);
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        // self.serialize_seq(Some(len))
        self.start_container(RionFieldType::OBJECT);
        self.serialize_key(name.as_bytes())?;
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        // let mut sized = SizedSerializer::new(self);
        // sized.serialize_key(variant)?;
        // Ok(sized)
        self.start_container(RionFieldType::OBJECT);
        self.serialize_key(variant.as_bytes())?;
        self.start_container(RionFieldType::ARRAY);
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        // Ok(SizedSerializer::new(self))
        self.start_container(RionFieldType::OBJECT);
        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.start_container(RionFieldType::OBJECT);
        self.serialize_key(variant.as_bytes())?;
        self.start_container(RionFieldType::OBJECT);
        Ok(self)
    }
}

// pub struct SizedSerializer<'a> {
//     output: &'a mut Serializer,
//     temp: Serializer,
// }

// impl<'a> SizedSerializer<'a> {
//     fn new(output: &'a mut Serializer) -> Self {
//         Self {
//             output,
//             temp: Serializer::new(),
//         }
//     }

//     fn finish(self, type_byte: u8) -> Result<(), SerializeError> {
//         let total_len = self.temp.output.len();
//         let length_length = needed_bytes_usize(total_len);
//         if length_length > 15 {
//             return Err(SerializeError::LengthOverflow(length_length)); // TODO handle error
//         }
//         self.output
//             .output
//             .push(type_byte << 4 | length_length as u8);
//         let ll = total_len as u64;
//         let orig = self.output.output.len();
//         crate::int_to_bytes(&ll, &mut self.output.output)?;
//         assert_eq!(self.output.output.len() - orig, length_length);
//         self.output.output.extend(self.temp.output);
//         Ok(())
//     }
// }

impl<'a> SerializeTuple for &'a mut Serializer {
    // impl<'a> SerializeTuple for SizedSerializer<'a> {
    type Ok = ();
    type Error = SerializeError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        // Array type serialization
        // self.finish(0xA)
        self.end_container();
        Ok(())
    }
}

impl<'a> SerializeSeq for &'a mut Serializer {
    // impl<'a> SerializeSeq for SizedSerializer<'a> {
    type Ok = ();
    type Error = SerializeError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // Array type serialization
        // self.finish(0xA)
        self.end_container();
        Ok(())
    }
}

// impl SerializeMap for SizedSerializer<'_> {
impl SerializeMap for &'_ mut Serializer {
    type Ok = ();
    type Error = SerializeError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        // key.serialize(&mut self.temp)
        let initial_len = self.output.len();
        key.serialize(&mut **self)?;
        // assert!(self.output.len() > initial_len);
        let lead = self.output[initial_len]; // Guaranteed to have at least one byte written
        let lead_byte = LeadByte::try_from(lead)?;
        // If the first byte is not a Key field, throw an error
        let ft = lead_byte.field_type();
        let target = &mut self.output[initial_len];
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
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // Object type serialization
        // self.finish(0xC)
        self.end_container();
        Ok(())
    }
}

impl SerializeTupleStruct for &'_ mut Serializer {
    // impl SerializeTupleStruct for SizedSerializer<'_> {
    type Ok = ();
    type Error = SerializeError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // self.finish(0xA)
        self.end_container();
        self.end_container();
        Ok(())
    }
}

impl SerializeStruct for &'_ mut Serializer {
    // impl SerializeStruct for SizedSerializer<'_> {
    type Ok = ();
    type Error = SerializeError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.serialize_key(key)?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // self.finish(0xC)
        self.end_container();
        Ok(())
    }
}

impl SerializeStructVariant for &'_ mut Serializer {
    // impl SerializeStructVariant for SizedSerializer<'_> {
    type Ok = ();
    type Error = SerializeError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.serialize_key(key)?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // self.finish(0xC)
        self.end_container(); // End the inner object
        self.end_container(); // End the outer object
        Ok(())
    }
}

impl SerializeTupleVariant for &'_ mut Serializer {
    // impl SerializeTupleVariant for SizedSerializer<'_> {
    type Ok = ();
    type Error = SerializeError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // self.finish(0xA)
        self.end_container(); // End the array
        self.end_container(); // End the object
        Ok(())
    }
}
