use std::{borrow::Cow, collections::HashMap};

use crate::{
    field::NormalField,
    get_normal_header, num_needed_length,
    types::{NormalRionType, RionFieldType},
    Result, RionField,
};

// Struct to represent a RION object
#[derive(Debug, Clone, PartialEq)]
pub struct RionObject<'a> {
    // pub data: Cow<'a, [u8]>,
    pub fields: HashMap<Cow<'a, [u8]>, RionField<'a>>,
    byte_len: usize,
}

impl<'a> Default for RionObject<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> RionObject<'a> {
    // Create a new RION object
    pub fn new() -> Self {
        RionObject {
            fields: HashMap::new(),
            byte_len: 0,
        }
    }

    pub(crate) fn parse_rest(mut data: &'a [u8]) -> Result<Self> {
        let byte_len = data.len();
        let mut fields = HashMap::new();
        while data.len() > 0 {
            let (key, rest) = RionField::parse(data)?;
            if !key.is_key() {
                return Err(format!("Expected a key, found {key:?} in {data:x?}").into());
            }
            let (value, rest) = RionField::parse(rest)?;
            data = rest;
            fields.insert(key.to_data().unwrap(), value);
        }
        Ok(RionObject { fields, byte_len })
    }

    fn parse(data: &'a [u8]) -> Result<(Self, &[u8])> {
        let (lead, data_len, data) = get_normal_header(data)?;
        let RionFieldType::Normal(NormalRionType::Object) = lead.field_type() else {
            return Err("Expected a RION object".into());
        };
        let out = Self::parse_rest(&data[..data_len])?;
        Ok((out, &data[data_len..]))
    }

    pub fn from_slice(data: &'a [u8]) -> Result<Self> {
        let (object, rest) = Self::parse(data)?;
        if !rest.is_empty() {
            return Err("Extra data after object".into());
        }
        Ok(object)
    }

    // Add a field to the RION object
    pub fn add_field_bytes(&mut self, key: &'a [u8], field: impl Into<RionField<'a>>) {
        let field = field.into();
        self.byte_len += field.needed_bytes()
            + 1
            + if key.len() > 15 {
                num_needed_length(key.len())
            } else {
                0
            };
        self.fields.insert(key.into(), field);
    }

    pub fn add_field(&mut self, key: &'a str, field: impl Into<RionField<'a>>) {
        self.add_field_bytes(key.as_bytes(), field);
    }

    pub(crate) fn write_header(&self, writer: &mut impl std::io::Write) -> Result<()> {
        let length_length = num_needed_length(self.byte_len);
        if length_length > 15 {
            return Err("Object length field is too long".into());
        }
        writer.write(&[0xC0 | length_length as u8 & 0x0F])?;
        let len_bytes = self.byte_len.to_be_bytes();
        writer.write(&len_bytes[8 - length_length..])?;
        Ok(())
    }

    pub(crate) fn write_body(&self, writer: &mut impl std::io::Write) -> Result<()> {
        let mut fields = self.fields.iter().collect::<Vec<_>>();
        fields.sort_unstable_by_key(|f| f.0);
        for (key, field) in &self.fields {
            RionField::key(key).write(writer)?;
            field.write(writer)?;
        }
        Ok(())
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        self.write(&mut out).unwrap();
        out
    }

    // Encode the RION object to its binary representation
    pub fn write(&self, writer: &mut impl std::io::Write) -> Result<()> {
        // let mut content = Vec::new();
        self.write_header(writer)?;
        self.write_body(writer)?;
        // let content_len = content.len();
        // let length_length = content_len.div_ceil(64);
        // if length_length > 15 {
        //     println!("Warning: Object length field is too long, truncating to 15 bytes");
        // }
        // println!("Content length: {content_len} - Num Bytes {length_length}");
        // let length_bytes = content_len.to_be_bytes();
        // let mut encoded = Vec::with_capacity(1 + content_len + length_length);

        
        // encoded
        Ok(())
    }

    // // Decode a RION object from its binary representation
}

impl<'a> From<RionObject<'a>> for RionField<'a> {
    fn from(obj: RionObject) -> Self {
        let mut content = Vec::new();
        for (key, field) in &obj.fields {
            let key_field = RionField::key(key);
            key_field.write(&mut content).unwrap();
            field.write(&mut content).unwrap();
        }
        RionField::Normal(NormalField {
            field_type: NormalRionType::Object,
            // length_length: content.len().div_ceil(64) as u8,
            data: content.into(),
        })
    }
}
