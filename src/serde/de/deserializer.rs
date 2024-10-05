use std::{
    error::Error,
    fmt::{Debug, Display},
};

use serde::{
    de::{value::StrDeserializer, IntoDeserializer, SeqAccess, Visitor},
    forward_to_deserialize_any,
};

use crate::{
    bytes_to_uint, get_header, get_normal_header,
    types::{LeadByte, NormalRionType, RionFieldType, ShortRionType},
    RionField,
};

impl serde::de::Error for DeserializeError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        let msg = msg.to_string();
        DeserializeError::Custom(msg)
    }
}

impl std::error::Error for DeserializeError {}
impl Debug for DeserializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}
impl Display for DeserializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeserializeError::Eod => write!(f, "end of available data!")?,
            DeserializeError::InvalidData(data) => write!(f, "invalid data! {data:?}")?,
            DeserializeError::Custom(msg) => write!(f, "{}", msg)?,
            DeserializeError::ExpectedNull => write!(f, "expected null")?,
            DeserializeError::DataLength(expected, actual, data) => write!(
                f,
                "expected data length {expected}, but got {actual} from {data:?}"
            )?,
            DeserializeError::InvalidType(expected, actual) => {
                write!(f, "expected type {expected:?}, but got {actual:?}")?
            }
            DeserializeError::ExtraData => write!(f, "extra data found")?,
        }
        Ok(())
    }
}

impl From<String> for DeserializeError {
    fn from(err: String) -> Self {
        DeserializeError::Custom(err)
    }
}

impl From<Box<dyn Error>> for DeserializeError {
    fn from(err: Box<dyn Error>) -> Self {
        DeserializeError::Custom(err.to_string())
    }
}

// impl<T: Display> From<T> for DeserializeError {
//     fn from(err: T) -> Self {
//         DeserializeError::Custom(err.to_string())
//     }
// }

pub fn from_bytes<'de, T>(data: &'de [u8]) -> Result<T, DeserializeError>
where
    T: serde::de::Deserialize<'de>,
{
    let mut deserializer = Deserializer::new(data);
    T::deserialize(&mut deserializer)
}

// #[derive(Debug)]
#[derive(PartialEq)]
pub enum DeserializeError {
    Eod,
    DataLength(usize, usize, Vec<u8>),         // Expected, Actual
    InvalidType(RionFieldType, RionFieldType), // Expected, Actual
    ExpectedNull,
    ExtraData,
    InvalidData(Vec<u8>),
    Custom(String),
}

pub struct Deserializer<'de> {
    data: &'de [u8],
}

pub struct BytesDeserializer<'de> {
    data: &'de [u8],
}

impl<'a, 'de> serde::Deserializer<'de> for &'a mut BytesDeserializer<'de> {
    type Error = DeserializeError;

    fn deserialize_any<V>(self, _: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unreachable!("Only intended for bytes deserialization")
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.data.is_empty() {
            return Err(DeserializeError::Eod);
        }
        let value = self.data[0];
        self.data = &self.data[1..];
        visitor.visit_u8(value)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(self)
    }

    forward_to_deserialize_any! {
      u64 u32 u16 i64 i32 i16 i8 bool f32 f64 char string str bytes byte_buf option unit unit_struct newtype_struct tuple tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de> SeqAccess<'de> for BytesDeserializer<'de> {
    type Error = DeserializeError;
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        // if std::mem::size_of::<T>() != 1 {
        //     return Err(DeserializeError::Custom(
        //         "Expected bytes deserializer to only deserialize bytes".into(),
        //     ));
        // }
        match self.data.first() {
            Some(_) => Ok(Some(seed.deserialize(self)?)),
            None => Ok(None),
        }
    }
    fn size_hint(&self) -> Option<usize> {
        Some(self.data.len())
    }
}

fn deserialize_string<'de, V: Visitor<'de>>(
    data: &'de [u8],
    visitor: V,
) -> Result<V::Value, DeserializeError> {
    match std::str::from_utf8(data) {
        Ok(data) => visitor.visit_str(data),
        Err(_) => visitor.visit_borrowed_bytes(data),
    }
}
fn deserialize_normal<'de, V: Visitor<'de>>(
    normal: NormalRionType,
    data: &'de [u8],
    visitor: V,
) -> Result<V::Value, DeserializeError> {
    match normal {
        NormalRionType::Array => visitor.visit_seq(&mut Deserializer::new(data)),
        NormalRionType::Object => visitor.visit_map(&mut Deserializer::new(data)),
        NormalRionType::UTF8 | NormalRionType::Key => deserialize_string(data, visitor),
        NormalRionType::Bytes => visitor.visit_seq(BytesDeserializer { data }),
        NormalRionType::Table => todo!(),
    }
}

fn deserialize_short<'de, V: Visitor<'de>>(
    short: ShortRionType,
    length: &'de [u8],
    visitor: V,
) -> Result<V::Value, DeserializeError> {
    // println!("Short: {short:?} ({length:X?})");
    match short {
        ShortRionType::Key | ShortRionType::UTF8 => deserialize_string(length, visitor),
        ShortRionType::Int64Positive => {
            let val = bytes_to_uint(length)?;
            visitor.visit_u64(val)
        }
        ShortRionType::Int64Negative => {
            let val = bytes_to_uint(length)?;
            let val = -(val as i64) - 1;
            visitor.visit_i64(val)
        }
        ShortRionType::Float => match length.len() {
            4 => visitor.visit_f32(f32::from_be_bytes(length.try_into().unwrap())),
            8 => visitor.visit_f64(f64::from_be_bytes(length.try_into().unwrap())),
            _ => Err(DeserializeError::DataLength(
                8,
                length.len(),
                length.to_vec(),
            )),
        },
        ShortRionType::UTCDateTime => todo!(),
    }
}

impl<'de> Deserializer<'de> {
    pub fn new(data: &'de [u8]) -> Self {
        Self { data }
    }

    pub fn next_byte(&mut self) -> Option<u8> {
        if self.data.is_empty() {
            return None;
        }
        let val = self.data[0];
        self.data = &self.data[1..];
        Some(val)
    }

    pub fn next_lead(&mut self) -> Option<LeadByte> {
        let lead = self.peek_lead()?;
        self.data = &self.data[1..];
        Some(lead)
    }

    // Returns none if there is no more data or it is an invalid lead byte
    pub fn peek_lead(&self) -> Option<LeadByte> {
        self.data
            .first()
            .copied()
            .map(LeadByte::try_from)
            .and_then(Result::ok)
    }

    fn deserialize_field<V>(&mut self, visitor: V) -> Result<V::Value, DeserializeError>
    where
        V: Visitor<'de>,
    {
        let (lead, length, rest) = get_header(self.data)?;
        self.data = rest;
        if lead.is_null() {
            return visitor.visit_none();
        }
        match lead.field_type() {
            RionFieldType::Bool(lead) => visitor.visit_bool(lead.unwrap()),
            RionFieldType::Short(short) => deserialize_short(short, length, visitor),
            RionFieldType::Normal(normal) => {
                let length_length = bytes_to_uint(length)? as usize;
                if length_length > self.data.len() {
                    return Err(DeserializeError::DataLength(
                        length_length,
                        self.data.len(),
                        self.data.to_vec(),
                    ));
                }
                let (data, rest) = self.data.split_at(length_length);
                self.data = rest;
                deserialize_normal(normal, data, visitor)
            }
            _ => Err(DeserializeError::InvalidData(self.data.to_vec())),
        }
    }

    fn parse_next_field(&mut self) -> Result<RionField<'de>, DeserializeError> {
        let (field, rest) = RionField::parse(self.data)?;
        self.data = rest;
        Ok(field)
    }

    fn parse_field<'a, T>(&'a mut self) -> Result<T, DeserializeError>
    where
        T: TryFrom<RionField<'a>, Error: Display>,
    {
        let field = self.parse_next_field()?;
        // println!("{:?}", field);
        field
            .try_into()
            .map_err(|e: T::Error| DeserializeError::Custom(e.to_string()))
    }
}

impl<'de, 'a> serde::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = DeserializeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_field(visitor)
    }

    forward_to_deserialize_any! {
      bool i64 u64 f32 f64 str ignored_any seq identifier map bytes string unit unit_struct newtype_struct
      tuple struct
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (lead, length, rest) = get_normal_header(self.data)?;
        if !lead.field_type().is_normal_type(NormalRionType::Object) {
            return Err(DeserializeError::InvalidType(
                RionFieldType::Normal(NormalRionType::Object),
                lead.field_type(),
            ));
        }
        let obj_data = &rest[..length];
        self.data = &rest[length..];
        let (lead, length, rest) = get_normal_header(obj_data)?;
        let ft = lead.field_type();
        if !ft.is_normal_type(NormalRionType::Array) {
            return Err(DeserializeError::InvalidType(
                RionFieldType::Normal(NormalRionType::Array),
                ft,
            ));
        }
        visitor.visit_seq(&mut Deserializer::new(&rest[..length]))
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let value: _ = self.parse_field()?;
        visitor.visit_i8(value)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let value: _ = self.parse_field()?;
        visitor.visit_i16(value)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let value: _ = self.parse_field()?;
        visitor.visit_i32(value)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let value: _ = self.parse_field()?;
        visitor.visit_u8(value)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let value: _ = self.parse_field()?;
        visitor.visit_u16(value)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let value: _ = self.parse_field()?;
        visitor.visit_u32(value)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_char(self.parse_field()?)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let field = self.parse_next_field()?;
        let field_type = field.field_type();
        let RionFieldType::Normal(NormalRionType::Bytes) = field_type else {
            return Err(DeserializeError::InvalidType(
                RionFieldType::Normal(NormalRionType::Bytes),
                field_type,
            ));
        };
        visitor.visit_byte_buf(field.as_bytes().to_vec())
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let Some(first) = self.data.first() else {
            return Err(DeserializeError::Eod);
        };
        let lead = LeadByte::try_from(*first)?;
        if lead.is_null() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        // Can either be a string or an object with a name, then a single field, object, or array

        let (lead, length, rest) = get_header(self.data)?;
        let field_type = lead.field_type();
        match field_type {
            ft if ft.is_label() => {
                let field = self.parse_next_field()?;
                visitor.visit_enum(field.as_str().unwrap().into_deserializer())
            }
            RionFieldType::Normal(NormalRionType::Object) => {
                let length = bytes_to_uint(length)? as usize;
                self.data = &rest[length..];
                visitor.visit_enum(&mut Deserializer::new(&rest[..length]))
            }
            _ => Err(DeserializeError::InvalidType(
                RionFieldType::Normal(NormalRionType::Object),
                field_type,
            )),
        }
    }
}

impl<'de> serde::de::SeqAccess<'de> for Deserializer<'de> {
    type Error = DeserializeError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if self.data.is_empty() {
            return Ok(None);
        }
        let value = seed.deserialize(self)?;
        Ok(Some(value))
    }
}

impl<'de> serde::de::MapAccess<'de> for Deserializer<'de> {
    type Error = DeserializeError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        // If the next byte is not a key, return None
        let Some(lead) = self.peek_lead() else {
            return Ok(None);
        };
        if !lead.field_type().is_key() {
            return Ok(None);
        }
        seed.deserialize(self).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(self)
    }
}

impl<'de> serde::de::VariantAccess<'de> for &'_ mut Deserializer<'de> {
    type Error = DeserializeError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        unreachable!("Should be handled by deserialize_enum")
    }
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(self)
    }
    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (lead, length, rest) = get_normal_header(self.data)?;
        if !lead.field_type().is_normal_type(NormalRionType::Array) {
            return Err(DeserializeError::InvalidType(
                RionFieldType::Normal(NormalRionType::Array),
                lead.field_type(),
            ));
        }
        self.data = &rest[length..];
        visitor.visit_seq(&mut Deserializer::new(&rest[..length]))
    }
    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // println!("Struct Variant: {:?}", fields);
        let (lead, length, rest) = get_normal_header(self.data)?;
        if !lead.field_type().is_normal_type(NormalRionType::Object) {
            return Err(DeserializeError::InvalidType(
                RionFieldType::Normal(NormalRionType::Object),
                lead.field_type(),
            ));
        }
        self.data = &rest[length..];
        visitor.visit_map(&mut Deserializer::new(&rest[..length]))
    }
}

impl<'de> serde::de::EnumAccess<'de> for &'_ mut Deserializer<'de> {
    type Error = DeserializeError;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let lead = self.peek_lead().ok_or(DeserializeError::Eod)?;
        let ft = lead.field_type();
        if !ft.is_label() {
            return Err(DeserializeError::InvalidType(
                RionFieldType::Normal(NormalRionType::Key),
                ft,
            ));
        }

        let (key, rest) = RionField::parse(self.data)?;

        let variant = {
            let name = key.as_str().unwrap();
            let name_de: StrDeserializer<'_, DeserializeError> = name.into_deserializer();
            seed.deserialize(name_de)?
        };

        self.data = rest;
        Ok((variant, self))
    }
}

// struct Enum<'a, 'de>(&'a mut Deserializer<'de>, usize);

// impl<'a, 'de> serde::de::VariantAccess<'de> for Enum<'a, 'de> {
//     type Error = DeserializeError;

//     fn unit_variant(self) -> Result<(), Self::Error> {
//         // let lead = self.next_lead().ok_or(DeserializeError::Eod)?;
//         // if lead.is_null() {
//         //     Ok(())
//         // } else {
//         //     Err(DeserializeError::InvalidData(self.data.to_vec()))
//         // }
//         Err(DeserializeError::InvalidData(self.0.data.to_vec())) // Should not be called
//     }

//     fn newtype_variant_seed<T>(mut self, seed: T) -> Result<T::Value, Self::Error>
//     where
//         T: serde::de::DeserializeSeed<'de>,
//     {
//         let orig = self.0.data.len();
//         let res = seed.deserialize(&mut *self.0);
//         self.1 -= orig - self.0.data.len();
//         res
//     }

//     fn tuple_variant<V>(mut self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: Visitor<'de>,
//     {
//         let orig = self.0.data.len();
//         let res = visitor.visit_seq(&mut *self.0);
//         self.1 -= orig - self.0.data.len();
//         // assert_eq!(self.1, 0);
//         res
//     }

//     fn struct_variant<V>(
//         mut self,
//         fields: &'static [&'static str],
//         visitor: V,
//     ) -> Result<V::Value, Self::Error>
//     where
//         V: Visitor<'de>,
//     {
//         let orig = self.0.data.len();
//         let res = visitor.visit_map(&mut *self.0);
//         self.1 -= orig - self.0.data.len();
//         // assert_eq!(self.1, 0);
//         res
//     }
// }

// impl<'a, 'de> serde::de::EnumAccess<'de> for Enum<'a, 'de> {
//     type Error = DeserializeError;
//     type Variant = Self;
//     fn variant_seed<V>(mut self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
//     where
//         V: serde::de::DeserializeSeed<'de>,
//     {
//         let (key, rest) = RionField::parse(self.0.data)?;

//         if !key.field_type().is_label() {
//             return Err(DeserializeError::InvalidType(
//                 RionFieldType::Normal(NormalRionType::Key),
//                 key.field_type(),
//             ));
//         }

//         let variant = {
//             let name_data = key.as_bytes();
//             let name = std::str::from_utf8(name_data)
//                 .map_err(|_| DeserializeError::InvalidData(name_data.to_vec()))?;
//             println!("Variant: {}", name);
//             let name_de: StrDeserializer<'_, DeserializeError> = StrDeserializer::new(name);
//             seed.deserialize(name_de)?
//         };
//         self.1 -= key.needed_bytes();
//         self.0.data = rest;
//         Ok((variant, self))
//     }
// }
