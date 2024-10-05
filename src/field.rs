use crate::{bytes_to_uint, get_header, int_to_bytes, needed_bytes_usize, types::*, Result};
use chrono::{DateTime, Datelike, Timelike, Utc};
use core::str;
use std::{borrow::Cow, error::Error};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShortField<'a> {
    pub(crate) field_type: ShortRionType,
    data: Cow<'a, [u8]>,
}

impl<'a> ShortField<'a> {
    pub fn new<D>(field_type: ShortRionType, data: D) -> Self
    where
        D: Into<Cow<'a, [u8]>> + ?Sized,
    {
        let data = data.into();
        let data_len = data.len() as u8;
        if data_len > 15 {
            panic!("Data too large for short field");
        }
        ShortField { field_type, data }
    }

    pub fn null(field_type: ShortRionType) -> Self {
        ShortField {
            field_type,
            data: (&[]).into(),
        }
    }

    pub fn parse(
        input: &'a [u8],
        // field_type: ShortRionType,
    ) -> Result<(Self, &'a [u8])> {
        let (lead_byte, data, input) = get_header(input)?;
        let field_type = lead_byte.field_type();
        let RionFieldType::Short(field_type) = field_type else {
            return Err("Field type is not short".into());
        };
        let data_len = data.len();
        if data_len > 15 {
            return Err("Data length too large for short field".into());
        }
        Ok((
            ShortField {
                field_type,
                data: data.into(),
            },
            input,
        ))
    }

    pub fn extend(&self, data: &mut impl std::io::Write) -> std::io::Result<()> {
        data.write_all(&[self.field_type.to_byte() << 4 | self.data.len() as u8])?;
        data.write_all(&self.data)?;
        Ok(())
    }

    pub fn as_str(&self) -> Option<&str> {
        match self.field_type {
            ShortRionType::Key | ShortRionType::UTF8 => std::str::from_utf8(&self.data).ok(),
            _ => None,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn as_pos_int(&self) -> Option<u64> {
        if self.data.len() > 8 || self.field_type != ShortRionType::Int64Positive {
            return None;
        }
        let mut bytes = [0; 8];
        bytes[8 - self.data.len()..].copy_from_slice(&self.data);
        Some(u64::from_be_bytes(bytes))
    }

    pub fn as_neg_int(&self) -> Option<i64> {
        if self.data.len() > 8 || self.field_type != ShortRionType::Int64Negative {
            return None;
        }
        let mut bytes = [0; 8];
        bytes[8 - self.data.len()..].copy_from_slice(&self.data);
        Some(-(i64::from_be_bytes(bytes) + 1))
    }

    pub fn as_f32(&self) -> Option<f32> {
        if self.data.len() > 4 || self.field_type != ShortRionType::Float {
            return None;
        }
        let mut bytes = [0; 4];
        bytes[4 - self.data.len()..].copy_from_slice(&self.data);
        Some(f32::from_be_bytes(bytes))
    }

    pub fn as_f64(&self) -> Option<f64> {
        if self.data.len() > 8 || self.field_type != ShortRionType::Float {
            return None;
        }
        let mut bytes = [0; 8];
        bytes[8 - self.data.len()..].copy_from_slice(&self.data);
        Some(f64::from_be_bytes(bytes))
    }

    pub fn is_null(&self) -> bool {
        self.data.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalField<'a> {
    pub(crate) field_type: NormalRionType,
    // Length in bytes of the length field
    pub(crate) data: Cow<'a, [u8]>,
}

impl<'a> NormalField<'a> {
    pub fn new<D>(field_type: NormalRionType, data: D) -> Self
    where
        D: Into<Cow<'a, [u8]>> + ?Sized,
    {
        let data = data.into();
        if needed_bytes_usize(data.len()) > 15 {
            panic!("Data too large for normal field");
        }
        NormalField { field_type, data }
    }

    pub fn null(field_type: NormalRionType) -> Self {
        NormalField {
            field_type,
            data: (&[]).into(),
        }
    }

    pub fn parse(
        input: &'a [u8],
        length_length: usize,
        field_type: NormalRionType,
    ) -> Result<(Self, &'a [u8])> {
        match length_length {
            16.. => return Err("Length too large for normal field".into()),
            l if l > input.len() => return Err("Input too short for length field".into()),
            0 => return Ok((NormalField::null(field_type), input)),
            _ => {}
        }
        let data_len = bytes_to_uint(&input[..length_length])? as usize;
        if data_len > input.len() {
            return Err(format!(
                "Input too short for data field ({}), expected {data_len}",
                input.len()
            )
            .into());
        }
        let input = &input[length_length..];
        let data = (&input[..data_len]).into();
        Ok((NormalField { field_type, data }, &input[data_len..]))
    }

    pub fn extend(&self, data: &mut impl std::io::Write) -> Result<()> {
        let length_length = needed_bytes_usize(self.data.len());
        if length_length > 15 {
            return Err("Data length too large for normal field".into());
        }
        data.write_all(&[self.field_type.to_byte() << 4 | length_length as u8])?;
        // lead_byte.length() == bytes needed to represent d_len
        // write the length of the data
        int_to_bytes(&(self.data.len() as u64), data)?;
        // let length_bytes = &self.data.len().to_be_bytes()[8 - length_length..];
        // println!("Length bytes: {:?}", length_bytes);
        // data.write_all(length_bytes)?;
        data.write_all(&self.data)?;
        Ok(())
    }

    pub fn is_null(&self) -> bool {
        self.data.is_empty()
    }

    pub fn as_str(&self) -> Option<&str> {
        match self.field_type {
            NormalRionType::Key | NormalRionType::UTF8 => std::str::from_utf8(&self.data).ok(),
            _ => None,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum RionField<'a> {
    // Tiny(LeadByte), // Has field type and 4 bits of data
    Bool(Option<bool>), // Has field type and 4 bits of data
    Short(ShortField<'a>),
    Normal(NormalField<'a>), // Short encoding also included
                             // TODO Extended
}

impl<'a> RionField<'a> {
    pub fn expect<T: From<Self>>(self) -> T {
        self.into()
    }

    pub fn key(key: &'a [u8]) -> Self {
        if key.len() < 16 {
            RionField::Short(ShortField {
                field_type: ShortRionType::Key,
                data: key.into(),
            })
        } else {
            RionField::Normal(NormalField {
                field_type: NormalRionType::Key,
                data: key.into(),
            })
        }
    }

    pub fn key_str(key: &'a str) -> Self {
        Self::key(key.as_bytes())
    }

    pub fn bytes(data: &'a [u8]) -> Self {
        RionField::Normal(NormalField {
            field_type: NormalRionType::Bytes,
            data: data.into(),
        })
    }

    pub fn f32(value: f32) -> Self {
        value.into()
    }

    pub fn f64(value: f64) -> Self {
        value.into()
    }

    pub fn int64(value: i64) -> Self {
        value.into()
    }

    pub fn uint64(value: u64) -> Self {
        value.into()
    }

    pub fn bool(value: bool) -> Self {
        value.into()
    }

    pub fn from_str(value: &'a str) -> Self {
        value.into()
    }

    pub fn parse(data: &'a [u8]) -> Result<(RionField<'a>, &'a [u8])> {
        let (lead, length, mut rest) = get_header(data)?;
        let parsed = match lead.field_type() {
            RionFieldType::Short(short) => ShortField::new(short, length).into(),
            RionFieldType::Normal(normal) => {
                // let (normal, rest) = NormalField::parse(rest, length, normal)?;
                // (RionField::Normal(normal), rest)
                let length = bytes_to_uint(length)? as usize;
                let field = NormalField::new(normal, &rest[..length]);
                rest = &rest[length..];
                field.into()
            }
            RionFieldType::Bool(lead) => RionField::Bool(lead),
            RionFieldType::Extended => todo!(),
        };
        Ok((parsed, rest))
    }

    pub fn encode(&self, data: &mut impl std::io::Write) -> Result<()> {
        match self {
            RionField::Bool(lead) => {
                data.write_all(&[RionFieldType::BOOL << 4
                    | match lead {
                        Some(v) => *v as u8 + 1,
                        None => 0,
                    }])?;
            }
            RionField::Short(short) => {
                short.extend(data)?;
            }
            RionField::Normal(normal) => {
                normal.extend(data)?;
            }
        }
        Ok(())
    }

    pub fn from_slice(buf: &'a [u8]) -> Result<Self> {
        let (field, rest) = Self::parse(buf)?;
        if !rest.is_empty() {
            return Err("Extra data after field".into());
        }
        Ok(field)
    }

    pub fn is_key(&self) -> bool {
        match self {
            RionField::Short(short) => short.field_type == ShortRionType::Key,
            RionField::Normal(normal) => normal.field_type == NormalRionType::Key,
            _ => false,
        }
    }

    pub fn is_null(&self) -> bool {
        match self {
            RionField::Bool(lead) => lead.is_none(),
            RionField::Short(short) => short.is_null(),
            RionField::Normal(normal) => normal.is_null(),
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            RionField::Short(short) => short.as_str(),
            RionField::Normal(normal) => normal.as_str(),
            _ => None,
        }
    }

    pub fn as_bytes(&'a self) -> &'a [u8] {
        match self {
            RionField::Short(short) => short.as_bytes(),
            RionField::Normal(normal) => normal.as_bytes(),
            _ => &[],
        }
    }

    // Bytes needed to encode this field
    pub fn needed_bytes(&self) -> usize {
        1 + match self {
            RionField::Short(short) => short.data.len(),
            RionField::Normal(normal) => {
                let data_len = normal.data.len();
                data_len + needed_bytes_usize(data_len)
            }
            _ => 0,
        }
    }

    pub fn to_data(self) -> Option<Cow<'a, [u8]>> {
        // pub fn to_data(self) -> Option<&'a [u8]> {
        match self {
            RionField::Short(short) => Some(short.data),
            RionField::Normal(normal) => Some(normal.data),
            _ => None,
        }
    }

    pub fn is_normal_type(&self, field_type: NormalRionType) -> bool {
        match self {
            RionField::Normal(normal) => normal.field_type == field_type,
            _ => false,
        }
    }

    pub fn is_short_type(&self, field_type: ShortRionType) -> bool {
        match self {
            RionField::Short(short) => short.field_type == field_type,
            _ => false,
        }
    }

    pub fn field_type(&self) -> RionFieldType {
        match self {
            RionField::Bool(lead) => RionFieldType::Bool(*lead),
            RionField::Short(short) => RionFieldType::Short(short.field_type),
            RionField::Normal(normal) => RionFieldType::Normal(normal.field_type),
        }
    }
}

impl<'a> From<NormalField<'a>> for RionField<'a> {
    fn from(value: NormalField<'a>) -> Self {
        RionField::Normal(value)
    }
}

impl<'a> From<ShortField<'a>> for RionField<'a> {
    fn from(value: ShortField<'a>) -> Self {
        RionField::Short(value)
    }
}

impl From<i64> for RionField<'_> {
    fn from(value: i64) -> Self {
        let field_type = if value < 0 {
            ShortRionType::Int64Negative
        } else {
            ShortRionType::Int64Positive
        };
        let value = if value < 0 { -(value + 1) } else { value };
        let bytes = value.to_be_bytes();
        let zeros = if value == 0 {
            7
        } else {
            value.leading_zeros() / 8
        };
        RionField::Short(ShortField {
            field_type,
            data: bytes[zeros as usize..].to_vec().into(),
        })
    }
}
impl From<u64> for RionField<'_> {
    fn from(value: u64) -> Self {
        let bytes = value.to_be_bytes();
        let zeros = if value == 0 {
            7
        } else {
            value.leading_zeros() / 8
        };
        RionField::Short(ShortField {
            field_type: ShortRionType::Int64Positive,
            data: bytes[zeros as usize..].to_vec().into(),
        })
    }
}

impl From<DateTime<Utc>> for RionField<'_> {
    fn from(dt: DateTime<Utc>) -> Self {
        let year = dt.year();
        if year > 0xFFFF {
            println!("Year is too large, truncating to 2^16-1");
        }
        let mut data = Vec::with_capacity(11);
        let components = [
            dt.month() as u8,
            dt.day() as u8,
            dt.hour() as u8,
            dt.minute() as u8,
            dt.second() as u8,
        ];
        // Find the last non-zero component
        let last_non_zero = components.iter().rposition(|&x| x != 0).unwrap_or(0);
        // Add all components up to and including the last non-zero one
        data.extend_from_slice(&components[..=last_non_zero]);
        let nanos = dt.nanosecond();
        if nanos > 0 {
            if nanos % 1_000_000 == 0 {
                // Milliseconds (2 bytes)
                data.extend_from_slice(&((nanos / 1_000_000) as u16).to_be_bytes());
            } else if nanos % 1_000 == 0 {
                // Microseconds (3 bytes)
                let micros = nanos / 1_000;
                data.extend_from_slice(&[(micros >> 16) as u8, (micros >> 8) as u8, micros as u8]);
            } else {
                // Nanoseconds (4 bytes)
                data.extend_from_slice(&nanos.to_be_bytes());
            }
        }
        RionField::Short(ShortField {
            // lead_byte: LeadByte::from_type(RionFieldType::UTCDateTime, 11),
            field_type: ShortRionType::UTCDateTime,
            data: data.into(),
        })
    }
}

impl From<bool> for RionField<'_> {
    fn from(value: bool) -> Self {
        // add one since 0 is reserved for null
        RionField::Bool(Some(value))
    }
}

impl<'a> From<&'a str> for RionField<'a> {
    fn from(value: &'a str) -> Self {
        let value_len = value.len();
        match value_len {
            0 => RionField::Normal(NormalField {
                field_type: NormalRionType::UTF8,
                data: (&[]).into(),
            }),
            1..=15 => RionField::Short(ShortField {
                field_type: ShortRionType::UTF8,
                data: value.as_bytes().into(),
            }),
            _ => {
                // let data = value.as_bytes().to_vec();
                let num_bytes = needed_bytes_usize(value_len);
                if num_bytes > 15 {
                    println!("Warning: UTF-8 length field is too long, truncating to 15 bytes");
                } // TODO handle this
                RionField::Normal(NormalField {
                    field_type: NormalRionType::UTF8,
                    data: value.as_bytes().into(),
                })
            }
        }
    }
}

impl From<String> for RionField<'static> {
    fn from(value: String) -> Self {
        let value_len = value.len();
        match value_len {
            0..=15 => RionField::Short(ShortField {
                field_type: ShortRionType::UTF8,
                data: value.into_bytes().into(),
            }),
            _ => {
                let num_bytes = needed_bytes_usize(value_len);
                if num_bytes > 15 {
                    println!("Warning: UTF-8 length field is too long, truncating to 15 bytes");
                } // TODO handle this
                RionField::Normal(NormalField {
                    field_type: NormalRionType::UTF8,
                    data: value.into_bytes().into(),
                })
            }
        }
    }
}

impl From<f32> for RionField<'_> {
    fn from(value: f32) -> Self {
        let bytes = value.to_be_bytes();
        RionField::Short(ShortField {
            field_type: ShortRionType::Float,
            data: bytes.to_vec().into(),
        })
    }
}

impl From<f64> for RionField<'_> {
    fn from(value: f64) -> Self {
        let bytes = value.to_be_bytes();
        // let zeros = (value.to_bits().leading_zeros() / 8).min(7);
        RionField::Short(ShortField {
            field_type: ShortRionType::Float,
            data: bytes.to_vec().into(),
        })
    }
}

impl TryFrom<RionField<'_>> for i64 {
    type Error = Box<dyn std::error::Error>;
    fn try_from(value: RionField<'_>) -> Result<Self> {
        let out = match value {
            RionField::Short(short) => match short.field_type {
                ShortRionType::Int64Positive => short.as_pos_int().unwrap().try_into()?,
                ShortRionType::Int64Negative => short.as_neg_int().unwrap(),
                _ => return Err("Field is not an integer".into()),
            },
            _ => return Err("Field is not an integer".into()),
        };
        Ok(out)
    }
}
impl TryFrom<RionField<'_>> for u64 {
    type Error = Box<dyn std::error::Error>;
    fn try_from(value: RionField<'_>) -> Result<Self> {
        match value {
            RionField::Short(short) => short
                .as_pos_int()
                .ok_or_else(|| format!("Field is not a positive integer: {:?}", short).into()),
            _ => Err("Field is not a positive integer".into()),
        }
    }
}
impl TryFrom<RionField<'_>> for u32 {
    type Error = Box<dyn Error>;
    fn try_from(value: RionField<'_>) -> std::result::Result<Self, Self::Error> {
        let value = u64::try_from(value)?;
        if value > u32::MAX as u64 {
            return Err(format!("Value ({value:?}) is too large for u32").into());
        }
        Ok(value as u32)
    }
}
impl TryFrom<RionField<'_>> for u16 {
    type Error = Box<dyn Error>;
    fn try_from(value: RionField<'_>) -> std::result::Result<Self, Self::Error> {
        let value = u64::try_from(value)?;
        if value > u16::MAX as u64 {
            return Err(format!("Value ({value:?}) is too large for u16").into());
        }
        Ok(value as u16)
    }
}
impl TryFrom<RionField<'_>> for u8 {
    type Error = Box<dyn Error>;
    fn try_from(value: RionField<'_>) -> std::result::Result<Self, Self::Error> {
        let value = u64::try_from(value)?;
        if value > u8::MAX as u64 {
            return Err(format!("Value ({value:?}) is too large for u8").into());
        }
        Ok(value as u8)
    }
}
impl TryFrom<RionField<'_>> for i32 {
    type Error = Box<dyn Error>;
    fn try_from(value: RionField<'_>) -> std::result::Result<Self, Self::Error> {
        let value = i64::try_from(value)?;
        if value < i32::MIN as i64 || value > i32::MAX as i64 {
            return Err(format!("Value ({value:?}) is too large for i32").into());
        }
        Ok(value as i32)
    }
}
impl TryFrom<RionField<'_>> for i16 {
    type Error = Box<dyn Error>;
    fn try_from(value: RionField<'_>) -> std::result::Result<Self, Self::Error> {
        let value = i64::try_from(value)?;
        if value < i16::MIN as i64 || value > i16::MAX as i64 {
            return Err(format!("Value ({value:?}) is too large for i16").into());
        }
        Ok(value as i16)
    }
}
impl TryFrom<RionField<'_>> for i8 {
    type Error = Box<dyn Error>;
    fn try_from(value: RionField<'_>) -> std::result::Result<Self, Self::Error> {
        let value = i64::try_from(value)?;
        if value < i8::MIN as i64 || value > i8::MAX as i64 {
            return Err(format!("Value ({value:?}) is too large for i8").into());
        }
        Ok(value as i8)
    }
}

impl TryFrom<RionField<'_>> for f32 {
    type Error = Box<dyn Error>;
    fn try_from(value: RionField<'_>) -> Result<Self> {
        match value {
            RionField::Short(short) => short
                .as_f32()
                .ok_or_else(|| format!("Field is not a f32: {:?}", short).into()),
            _ => Err("Field is not a f32".into()),
        }
    }
}

impl TryFrom<RionField<'_>> for f64 {
    type Error = Box<dyn std::error::Error>;
    fn try_from(value: RionField<'_>) -> Result<Self> {
        match value {
            RionField::Short(short) => short
                .as_f64()
                .ok_or_else(|| format!("Field is not a f64: {:?}", short).into()),
            _ => Err("Field is not a f64".into()),
        }
    }
}

// impl<'a> TryFrom<RionField<'a>> for &'a str {
//     type Error = Box<dyn std::error::Error>;

//     fn try_from(value: RionField<'a>) -> std::result::Result<Self, Self::Error> {
//         match value {
//             RionField::Short(short) => {
//                 let str = short
//                     .as_str()
//                     .ok_or_else(|| format!("Field is not a string: {:?}", short))?;
//                 Ok(str)
//             }
//             RionField::Normal(normal) => {
//                 let str = normal
//                     .as_str()
//                     .ok_or_else(|| format!("Field is not a string: {:?}", normal))?;
//                 Ok(str)
//             }
//             _ => Err("Field is not a string".into()),
//         }
//     }
// }

impl TryFrom<RionField<'_>> for String {
    type Error = Box<dyn std::error::Error>;
    fn try_from(value: RionField<'_>) -> Result<Self> {
        match value {
            RionField::Short(short) => {
                let str = short
                    .as_str()
                    .ok_or_else(|| format!("Field is not a string: {:?}", short))?;
                Ok(str.to_string())
            }
            RionField::Normal(normal) => {
                let str = normal
                    .as_str()
                    .ok_or_else(|| format!("Field is not a string: {:?}", normal))?;
                Ok(str.to_string())
            }
            _ => Err("Field is not a string".into()),
        }
    }
}

impl TryFrom<RionField<'_>> for char {
    type Error = Box<dyn std::error::Error>;
    fn try_from(value: RionField<'_>) -> Result<Self> {
        let string: String = value.try_into()?;
        string
            .chars()
            .next()
            .ok_or_else(|| "String is empty".into())
    }
}

impl TryFrom<RionField<'_>> for bool {
    type Error = Box<dyn std::error::Error>;
    fn try_from(value: RionField<'_>) -> Result<Self> {
        match value {
            RionField::Bool(lead) => match lead {
                Some(v) => Ok(v),
                None => Err("Field is null".into()),
            },
            _ => Err("Field is not a boolean".into()),
        }
    }
}
// TODO Datetime into impl
