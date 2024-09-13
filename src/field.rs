use crate::{get_lead_byte, types::*, Result};
use chrono::{DateTime, Datelike, Timelike, Utc};
use core::str;
use num_bigint::BigUint;
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShortField<'a> {
    pub(crate) field_type: ShortRionType,
    pub(crate) data_len: u8,
    data: Cow<'a, [u8]>,
}

impl<'a> ShortField<'a> {
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

    pub fn parse(
        input: &'a [u8],
        data_len: usize,
        field_type: ShortRionType,
    ) -> Result<(Self, &'a [u8])> {
        if data_len > 15 {
            return Err("Data too large for short field".into());
        }
        let data = &input[..data_len];
        Ok((
            ShortField {
                field_type,
                data_len: data_len as u8,
                data: data.into(),
            },
            &input[data_len..],
        ))
    }

    pub fn extend(&self, data: &mut impl std::io::Write) -> std::io::Result<()> {
        data.write(&[self.field_type.to_byte() | self.data_len])?;
        data.write(&self.data)?;
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
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalField<'a> {
    pub(crate) field_type: NormalRionType,
    pub(crate) length_length: u8,
    pub(crate) data: Cow<'a, [u8]>,
}

impl<'a> NormalField<'a> {
    fn read_with_lead(
        mut buffer: Vec<u8>,
        field_type: NormalRionType,
        length_length: usize,
        buf: &mut impl std::io::Read,
    ) -> Result<Self> {
        if length_length > 15 {
            return Err("Length too large for normal field".into());
        }
        buffer.resize(length_length, 0);
        buf.read_exact(&mut buffer)?;
        // The next length_length bytes (0..15) are the number of bytes (as a number) in the data
        let data_len = BigUint::from_bytes_be(&buffer);
        let data_len: usize = data_len
            .try_into()
            .map_err(|_| "Data too large for this system!")?;
        buffer.resize(data_len, 0);
        buf.read_exact(&mut buffer)?;
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

    pub fn parse(
        input: &'a [u8],
        length_length: usize,
        field_type: NormalRionType,
    ) -> Result<(Self, &'a [u8])> {
        if length_length > 15 {
            return Err("Length too large for normal field".into());
        }
        let data_len = BigUint::from_bytes_be(&input[..length_length]);
        let data_len: usize = data_len
            .try_into()
            .map_err(|_| "Data too large for this system!")?;
        let data = &input[length_length..length_length + data_len];
        Ok((
            NormalField {
                field_type,
                length_length: length_length as u8,
                data: data.into(),
            },
            &input[length_length + data_len..],
        ))
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

    pub fn bytes(data: &'a [u8]) -> Self {
        RionField::Normal(NormalField {
            field_type: NormalRionType::Bytes,
            length_length: data.len().div_ceil(64) as u8 & 0x0F,
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

    pub fn from_str(value: &'a str) -> Self {
        value.into()
    }

    pub fn parse(data: &'a [u8]) -> Result<(RionField<'_>, &'a [u8])> {
        let (lead, rest) = get_lead_byte(data)?;
        let length = lead.length() as usize;
        let (parsed, rest) = match lead.field_type() {
            RionFieldType::Short(short) => {
                let (short, rest) = ShortField::parse(rest, length, short)?;
                (RionField::Short(short), rest)
            }
            RionFieldType::Normal(normal) => {
                let (normal, rest) = NormalField::parse(rest, length, normal)?;
                (RionField::Normal(normal), rest)
            }
            RionFieldType::Tiny(lead) => (RionField::Tiny(lead), rest),
            RionFieldType::Extended => todo!(),
        };
        Ok((parsed, rest))
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
                ShortField::read_with_lead(buffer, short, data_len, buf).map(RionField::Short)
            }
            RionFieldType::Normal(normal) => {
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
