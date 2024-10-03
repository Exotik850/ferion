use std::{
    error::Error,
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
};

use serde::{
    de::{SeqAccess, Visitor},
    forward_to_deserialize_any,
};

use crate::{
    bytes_to_int, get_header,
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

impl<'de> Deserializer<'de> {
    fn deserialize_string<V: Visitor<'de>>(
        &mut self,
        data: &'de [u8],
        visitor: V,
    ) -> Result<V::Value, DeserializeError> {
        match std::str::from_utf8(data) {
            Ok(data) => visitor.visit_str(data),
            Err(_) => visitor.visit_borrowed_bytes(data),
        }
    }

    fn deserialize_normal<V: Visitor<'de>>(
        &mut self,
        normal: NormalRionType,
        // length: &[u8],
        data: &'de [u8],
        visitor: V,
    ) -> Result<V::Value, DeserializeError> {
        // let len_data: usize = bytes_to_num(length)?;
        // if len_data > self.data.len() {
        //     return Err(DeserializeError::DataLength(len_data, self.data.len()));
        // }
        // let field_data = &self.data[..len_data];
        // self.data = &self.data[len_data..];
        // println!("Normal: {normal:?} ({data:X?})");
        match normal {
            // NormalRionType::Array => {
            //     visitor.visit_seq(SizedDeserializer::new(&mut Deserializer::new(data)))
            // }
            // NormalRionType::Object => {
            //     visitor.visit_map(SizedDeserializer::new(&mut Deserializer::new(data)))
            // }
            NormalRionType::Array => {
                let mut deserializer = Deserializer::new(data);
                visitor.visit_seq(SizedDeserializer::new(&mut deserializer))
            }
            NormalRionType::Object => {
                let mut deserializer = Deserializer::new(data);
                let result = visitor.visit_map(SizedDeserializer::new(&mut deserializer));
                if !deserializer.data.is_empty() {
                    return Err(DeserializeError::ExtraData);
                }
                result
            }
            NormalRionType::UTF8 | NormalRionType::Key => self.deserialize_string(data, visitor),
            NormalRionType::Bytes => visitor.visit_seq(BytesDeserializer { data }),
            NormalRionType::Table => todo!(),
        }
    }

    fn deserialize_short<V: Visitor<'de>>(
        &mut self,
        short: ShortRionType,
        length: &'de [u8],
        visitor: V,
    ) -> Result<V::Value, DeserializeError> {
        // println!("Short: {short:?} ({length:X?})");
        match short {
            ShortRionType::Key | ShortRionType::UTF8 => self.deserialize_string(length, visitor),
            ShortRionType::Int64Positive => {
                let val = bytes_to_int(length)?;
                visitor.visit_u64(val)
            }
            ShortRionType::Int64Negative => {
                let val = bytes_to_int(length)?;
                let val = -(val as i64 + 1);
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
        if lead.is_null() {
            return visitor.visit_none();
        }
        self.data = rest;
        match lead.field_type() {
            RionFieldType::Tiny(lead) => visitor.visit_bool(lead.as_bool().unwrap()),
            RionFieldType::Short(short) => self.deserialize_short(short, length, visitor),
            RionFieldType::Normal(normal) => {
                let length_length = bytes_to_int(length)? as usize;
                if length_length > self.data.len() {
                    return Err(DeserializeError::DataLength(
                        length_length,
                        self.data.len(),
                        self.data.to_vec(),
                    ));
                }
                let (data, rest) = self.data.split_at(length_length);
                self.data = rest;
                self.deserialize_normal(normal, data, visitor)
            }
            _ => Err(DeserializeError::InvalidData(self.data.to_vec())),
        }
    }

    fn parse_next_field(&mut self) -> Result<RionField<'de>, DeserializeError> {
        let (field, rest) = RionField::parse(self.data)
            .map_err(|_| DeserializeError::InvalidData(self.data.to_vec()))?;
        self.data = rest;
        Ok(field)
    }

    fn parse_field<T>(&mut self) -> Result<T, DeserializeError>
    where
        T: TryFrom<RionField<'de>, Error: Display>,
    {
        let field = self.parse_next_field()?;
        println!("{:?}", field);
        field
            .try_into()
            .map_err(|e: T::Error| DeserializeError::Custom(e.to_string()))
    }

    // fn visit_field<V>(
    //     &mut self,
    //     field: RionField<'de>,
    //     visitor: V,
    // ) -> Result<V::Value, DeserializeError>
    // where
    //     V: Visitor<'de>,
    // {
    //     match field.field_type() {
    //         RionFieldType::Normal(NormalRionType::Bytes) => todo!(),
    //         RionFieldType::Normal(NormalRionType::UTF8 | NormalRionType::Key)
    //         | RionFieldType::Short(ShortRionType::Key | ShortRionType::UTF8) => {
    //             visitor.visit_str(field.as_str().unwrap())
    //         }
    //         RionFieldType::Normal(NormalRionType::Array) => visitor.visit_seq(self),
    //         RionFieldType::Normal(NormalRionType::Object | NormalRionType::Table) => {
    //             visitor.visit_map(self) // TODO: Properly handle table
    //         }
    //         RionFieldType::Tiny(lead) => visitor.visit_bool(lead.length() == 2),
    //         RionFieldType::Short(ShortRionType::Int64Negative) => {
    //             visitor.visit_i64(field.try_into().unwrap())
    //         }
    //         RionFieldType::Short(ShortRionType::Int64Positive) => {
    //             visitor.visit_u64(field.try_into().unwrap())
    //         }
    //         RionFieldType::Short(ShortRionType::Float) => {
    //             let RionField::Short(short) = field else {
    //                 unreachable!()
    //             };
    //             match short.data_len {
    //                 ..=4 => visitor.visit_f32(short.as_f32().unwrap()),
    //                 ..=8 => visitor.visit_f64(short.as_f64().unwrap()),
    //                 _ => Err(DeserializeError::DataLength(8, short.data_len as usize)),
    //             }
    //         }
    //         RionFieldType::Short(ShortRionType::UTCDateTime) => {
    //             todo!()
    //         }
    //         _ => Err(DeserializeError::InvalidData(
    //             field.to_data().unwrap_or_default().to_vec(),
    //         )),
    //     }
    // }
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
      tuple tuple_struct struct
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
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }
}

impl<'de> Deref for SizedDeserializer<'_, 'de> {
    type Target = Deserializer<'de>;
    fn deref(&self) -> &Self::Target {
        self.serializer
    }
}

impl DerefMut for SizedDeserializer<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.serializer
    }
}

struct SizedDeserializer<'a, 'de> {
    serializer: &'a mut Deserializer<'de>,
}

impl<'a, 'de> SizedDeserializer<'a, 'de> {
    fn new(serializer: &'a mut Deserializer<'de>) -> Self {
        Self { serializer }
    }
}

impl<'de, 'a> serde::de::SeqAccess<'de> for SizedDeserializer<'a, 'de> {
    type Error = DeserializeError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if self.data.is_empty() {
            return Ok(None);
        }
        let value = seed.deserialize(&mut **self)?;
        Ok(Some(value))
    }
}

impl<'de, 'a> serde::de::MapAccess<'de> for SizedDeserializer<'a, 'de> {
    type Error = DeserializeError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        if self.data.is_empty() {
            return Ok(None);
        }
        // If the next byte is not a key, return None
        let field_val = self.data[0] & 0xF0;
        // let field_type = RionFieldType::try_from(field_val)?;
        // if !field_type.is_key() {
        //     return Ok(None);
        // }
        match RionFieldType::try_from(field_val) {
            Ok(field) if field.is_key() => field,
            _ => return Ok(None),
        };
        let key = seed.deserialize(&mut **self)?;
        Ok(Some(key))
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let value = seed.deserialize(&mut **self)?;
        Ok(value)
    }
}
