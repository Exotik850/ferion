use crate::Result;
use chrono::{DateTime, Datelike, Timelike, Utc};
use core::str;
use num_bigint::BigUint;
use std::borrow::Cow;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct LeadByte(u8); // (field type, length)

impl LeadByte {
    pub fn from_type(field_type: RionFieldType, length: u8) -> Self {
        LeadByte(field_type.to_byte() | length)
    }

    pub fn field_type(self) -> RionFieldType {
        RionFieldType::try_from(self.0).unwrap()
    }

    pub fn length(self) -> u8 {
        self.0 & 0x0F
    }

    pub fn is_null(self) -> bool {
        self.length() == 0
    }

    pub fn is_short(self) -> bool {
        self.length() < 15
    }

    pub fn byte(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for LeadByte {
    type Error = &'static str;
    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        RionFieldType::try_from(value)?;
        Ok(LeadByte(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShortField<'a> {
    pub(crate) field_type: ShortRionType,
    pub(crate) data_len: u8,
    data: Cow<'a, [u8]>,
}

impl ShortField<'_> {
    pub fn read_with_lead(
        mut buffer: Vec<u8>,
        field_type: ShortRionType,
        data_len: usize,
        buf: &mut impl std::io::Read,
    ) -> Result<Self> {
        if data_len > 15 {
            return Err("Data too large for short field".into());
        }
        buffer.resize(data_len, 0);
        buf.read_exact(&mut buffer)?;
        Ok(ShortField {
            field_type,
            data_len: data_len as u8,
            data: buffer.into(),
        })
    }

    pub fn extend(&self, data: &mut impl std::io::Write) -> std::io::Result<()> {
        data.write(&[self.field_type.to_byte() | self.data_len])?;
        data.write(&self.data)?;
        Ok(())
    }

    pub fn as_str(&self) -> Option<&str> {
        match self.field_type {
            ShortRionType::UTF8 => unsafe { Some(str::from_utf8_unchecked(&self.data)) },
            ShortRionType::Key => std::str::from_utf8(&self.data).ok(),
            _ => None,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalField<'a> {
    pub(crate) field_type: NormalRionType,
    pub(crate) length_length: u8,
    pub(crate) data: Cow<'a, [u8]>,
}

impl NormalField<'_> {
    fn read_with_lead(
        mut buffer: Vec<u8>,
        field_type: NormalRionType,
        length_length: usize,
        buf: &mut impl std::io::Read,
    ) -> Result<Self> {
        if length_length > 15 {
            return Err("Length too large for normal field".into());
        }
        println!("Length length: {}", length_length);
        buffer.resize(length_length, 0);
        buf.read_exact(&mut buffer)?;
        // The next length_length bytes (0..15) are the number of bytes (as a number) in the data

        println!("Buffer: {:?}", buffer);
        let data_len = BigUint::from_bytes_be(&buffer);
        let data_len: usize = data_len
        .try_into()
        .map_err(|_| "Data too large for this system!")?;
      println!("DataLen: {data_len}", );
      
      buffer.resize(data_len, 0);
      buf.read_exact(&mut buffer)?;
      println!("Buffer: {:?}", buffer);
        Ok(NormalField {
            field_type,
            length_length: length_length as u8,
            data: buffer.into(),
        })
    }

    pub fn read_from(buf: &mut impl std::io::Read) -> Result<Self> {
        let mut buffer = vec![0; 1];
        buf.read_exact(&mut buffer)?;
        let lead_byte = LeadByte::try_from(buffer[0])?;
        let RionFieldType::Normal(field_type) = lead_byte.field_type() else {
            return Err("Invalid field type".into());
        };
        let length_length = lead_byte.length() as usize;
        Self::read_with_lead(buffer, field_type, length_length, buf)
    }

    pub fn extend(&self, data: &mut impl std::io::Write) -> std::io::Result<()> {
        data.write(&[self.field_type.to_byte() | self.length_length])?;
        // lead_byte.length() == bytes needed to represent d_len
        let d_len = self.data.len();
        let num_bytes = d_len.div_ceil(64);
        if num_bytes > 15 {
            println!("Warning: Field length field is too long, truncating to 15 bytes");
        }
        data.write(&d_len.to_be_bytes()[8 - num_bytes..])?;
        data.write(&self.data)?;
        Ok(())
    }

    pub fn as_str(&self) -> Option<&str> {
        match self.field_type {
            NormalRionType::UTF8 => unsafe { Some(str::from_utf8_unchecked(&self.data)) },
            NormalRionType::Key => std::str::from_utf8(&self.data).ok(),
            _ => None,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum RionField<'a> {
    Tiny(LeadByte), // Has field type and 4 bits of data
    Short(ShortField<'a>),
    Normal(NormalField<'a>), // Short encoding also included
                             // TODO Extended
}

impl<'a> RionField<'a> {
    pub fn key(key: &'a [u8]) -> Self {
        if key.len() < 16 {
            RionField::Short(ShortField {
                field_type: ShortRionType::Key,
                data_len: key.len() as u8,
                data: key.into(),
            })
        } else {
            RionField::Normal(NormalField {
                field_type: NormalRionType::Key,
                length_length: key.len().div_ceil(64) as u8 & 0x0F,
                data: key.into(),
            })
        }
    }

    pub fn key_str(key: &'a str) -> Self {
        Self::key(key.as_bytes())
    }

    pub fn encode(&self, data: &mut impl std::io::Write) -> std::io::Result<()> {
        match self {
            RionField::Tiny(lead) => {
                data.write(&[lead.byte()])?;
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

    pub fn from_slice(buf: &[u8]) -> Result<Self> {
        let mut buf = std::io::Cursor::new(buf);
        Self::read_from(&mut buf)
    }

    pub fn read_from(buf: &mut impl std::io::Read) -> Result<Self> {
        let mut buffer = vec![0; 1];
        buf.read_exact(&mut buffer)?;
        let lead_byte = LeadByte::try_from(buffer[0])?;
        println!(
            "Lead byte: {:?} - {:?} - Length {}",
            lead_byte,
            lead_byte.field_type(),
            lead_byte.length()
        );
        let data_len = lead_byte.length() as usize;
        match lead_byte.field_type() {
            RionFieldType::Tiny(lead) => Ok(RionField::Tiny(lead)),
            RionFieldType::Short(short) => {
                println!("Short encoding");
                ShortField::read_with_lead(buffer, short, data_len, buf).map(RionField::Short)
            }
            RionFieldType::Normal(normal) => {
                println!("Normal encoding");
                NormalField::read_with_lead(buffer, normal, data_len, buf).map(RionField::Normal)
            }
            RionFieldType::Extended => unimplemented!(),
        }
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
            RionField::Tiny(lead) => lead.is_null(),
            RionField::Short(short) => short.data_len == 0,
            RionField::Normal(normal) => normal.data.is_empty(),
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            RionField::Short(short) => short.as_str(),
            RionField::Normal(normal) => normal.as_str(),
            _ => None,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            RionField::Short(short) => short.as_bytes(),
            RionField::Normal(normal) => normal.as_bytes(),
            _ => &[],
        }
    }

    pub fn to_data(self) -> Option<Cow<'a, [u8]>> {
        match self {
            RionField::Short(short) => Some(short.data),
            RionField::Normal(normal) => Some(normal.data),
            _ => None,
        }
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
        let zeros = value.leading_zeros() / 8;
        let num_bytes = 8 - zeros;
        RionField::Short(ShortField {
            field_type,
            data_len: num_bytes as u8,
            data: bytes[zeros as usize..].to_vec().into(),
        })
    }
}
impl From<u64> for RionField<'_> {
    fn from(value: u64) -> Self {
        let bytes = value.to_be_bytes();
        let zeros = value.leading_zeros() / 8;
        let num_bytes = 8 - zeros;
        RionField::Short(ShortField {
            field_type: ShortRionType::Int64Positive,
            data_len: num_bytes as u8,
            data: bytes[zeros as usize..].to_vec().into(),
        })
    }
}

impl From<DateTime<Utc>> for RionField<'_> {
    fn from(value: DateTime<Utc>) -> Self {
        let year = value.year();
        if year > 65535 {
            println!("Year is too large, truncating to 65535");
        }
        let mut data = Vec::with_capacity(11);
        data.extend_from_slice(&(year as u16).to_be_bytes());
        let bytes = [
            value.month(),
            value.day(),
            value.hour(),
            value.minute(),
            value.second(),
        ]
        .map(|v| v as u8); // TODO Compress unnecessary bytes
        data.extend_from_slice(&bytes);
        data.extend_from_slice(&value.nanosecond().to_be_bytes());
        RionField::Short(ShortField {
            // lead_byte: LeadByte::from_type(RionFieldType::UTCDateTime, 11),
            field_type: ShortRionType::UTCDateTime,
            data_len: 11,
            data: data.into(),
        })
    }
}

impl From<bool> for RionField<'_> {
    fn from(value: bool) -> Self {
        // add one since 0 is reserved for null
        RionField::Tiny(LeadByte(0x10 | (value as u8 + 1)))
    }
}

impl<'a> From<&'a str> for RionField<'a> {
    fn from(value: &'a str) -> Self {
        let value_len = value.len();
        match value_len {
            0..=15 => RionField::Short(ShortField {
                field_type: ShortRionType::UTF8,
                data_len: value_len as u8,
                data: value.as_bytes().into(),
            }),
            _ => {
                // let data = value.as_bytes().to_vec();
                let num_bytes = value_len.div_ceil(64);
                if num_bytes > 15 {
                    println!("Warning: UTF-8 length field is too long, truncating to 15 bytes");
                } // TODO handle this
                RionField::Normal(NormalField {
                    field_type: NormalRionType::UTF8,
                    length_length: num_bytes as u8 & 0x0F,
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
                data_len: value_len as u8,
                data: value.into_bytes().into(),
            }),
            _ => {
                let num_bytes = value_len.div_ceil(64);
                if num_bytes > 15 {
                    println!("Warning: UTF-8 length field is too long, truncating to 15 bytes");
                } // TODO handle this
                RionField::Normal(NormalField {
                    field_type: NormalRionType::UTF8,
                    length_length: num_bytes as u8 & 0x0F,
                    data: value.into_bytes().into(),
                })
            }
        }
    }
}

impl From<f32> for RionField<'_> {
    fn from(value: f32) -> Self {
        let bytes = value.to_be_bytes();
        let zeros = value.to_bits().leading_zeros() / 8;
        let needed = 4 - zeros;
        RionField::Short(ShortField {
            field_type: ShortRionType::Float,
            data_len: needed as u8,
            data: bytes[zeros as usize..].to_vec().into(),
        })
    }
}

impl From<f64> for RionField<'_> {
    fn from(value: f64) -> Self {
        let bytes = value.to_be_bytes();
        let zeros = value.to_bits().leading_zeros() / 8;
        let needed = 8 - zeros;
        RionField::Short(ShortField {
            field_type: ShortRionType::Float,
            data_len: needed as u8,
            data: bytes[zeros as usize..].to_vec().into(),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RionFieldType {
    Short(ShortRionType),
    Normal(NormalRionType),
    Extended,
    Tiny(LeadByte),
}

#[derive(Debug, Clone, PartialEq, Copy, Eq, Hash)]
pub enum ShortRionType {
    Int64Positive,
    Int64Negative,
    UTF8,
    UTCDateTime,
    Float,
    Key,
}

impl ShortRionType {
    fn to_byte(self) -> u8 {
        match self {
            ShortRionType::Int64Positive => 0x20,
            ShortRionType::Int64Negative => 0x30,
            ShortRionType::UTF8 => 0x50,
            ShortRionType::UTCDateTime => 0x70,
            ShortRionType::Float => 0x40,
            ShortRionType::Key => 0xD0,
        }
    }
}

impl TryFrom<u8> for ShortRionType {
    type Error = &'static str;
    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        let out = match value {
            0x20 => ShortRionType::Int64Positive,
            0x30 => ShortRionType::Int64Negative,
            0x50 => ShortRionType::UTF8,
            0x70 => ShortRionType::UTCDateTime,
            0x40 => ShortRionType::Float,
            0xD0 => ShortRionType::Key,
            _ => return Err("Invalid short field type"),
        };
        Ok(out)
    }
}

#[derive(Debug, Clone, PartialEq, Copy, Eq, Hash)]
pub enum NormalRionType {
    Bytes,
    UTF8,
    Array,
    Table,
    Object,
    Key,
}

impl TryFrom<u8> for NormalRionType {
    type Error = &'static str;
    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        let out = match value & 0xF0 {
            0x00 => NormalRionType::Bytes,
            0x50 => NormalRionType::UTF8,
            0xA0 => NormalRionType::Array,
            0xB0 => NormalRionType::Table,
            0xC0 => NormalRionType::Object,
            0xD0 => NormalRionType::Key,
            _ => return Err("Invalid normal field type"),
        };
        Ok(out)
    }
}

impl NormalRionType {
    fn to_byte(self) -> u8 {
        match self {
            NormalRionType::Bytes => 0x00,
            NormalRionType::UTF8 => 0x50,
            NormalRionType::Array => 0xA0,
            NormalRionType::Table => 0xB0,
            NormalRionType::Object => 0xC0,
            NormalRionType::Key => 0xD0,
        }
    }
}

impl RionFieldType {
    pub fn to_byte(self) -> u8 {
        match self {
            Self::Short(short) => short.to_byte(),
            Self::Normal(normal) => normal.to_byte(),
            Self::Extended => 0xF0,
            Self::Tiny(lead) => lead.byte(),
        }
    }
}

impl TryFrom<u8> for RionFieldType {
    type Error = &'static str;
    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        let type_bits = value & 0xF0;
        match type_bits {
            0xF0 => Ok(RionFieldType::Extended),
            0x10 => Ok(RionFieldType::Tiny(LeadByte(value))),
            0x80..=0xC0 => Ok(RionFieldType::Normal(NormalRionType::try_from(type_bits)?)),
            0x00..=0x70 | 0xD0 => Ok(RionFieldType::Short(ShortRionType::try_from(type_bits)?)),
            _ => Err("Invalid field type"),
        }
    }
}
