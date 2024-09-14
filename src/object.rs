use std::{borrow::Cow, collections::HashMap};

use crate::{
    field::NormalField,
    get_normal_header,
    types::{NormalRionType, RionFieldType},
    Result, RionField,
};

// Struct to represent a RION object
#[derive(Debug, Clone, PartialEq)]
pub struct RionObject<'a> {
    // pub data: Cow<'a, [u8]>,
    pub fields: HashMap<Cow<'a, [u8]>, RionField<'a>>,
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
        }
    }

    fn parse(data: &'a [u8]) -> Result<(Self, &[u8])> {
        let (lead, data_len, mut data) = get_normal_header(data)?;
        let RionFieldType::Normal(NormalRionType::Object) = lead.field_type() else {
            return Err("Expected a RION object".into());
        };
        let total = data.len();
        let mut fields = HashMap::new();
        while total - data.len() < data_len {
            let (key, rest) = RionField::parse(data)?;
            if !key.is_key() {
                return Err(format!("Expected a key, found {key:?} in {data:x?}").into());
            }
            let (value, rest) = RionField::parse(rest)?;
            data = rest;
            fields.insert(key.to_data().unwrap(), value);
        }
        Ok((RionObject { fields }, data))
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
        self.fields.insert(key.into(), field.into());
    }

    pub fn add_field(&mut self, key: &'a str, field: impl Into<RionField<'a>>) {
        self.add_field_bytes(key.as_bytes(), field);
    }

    // Encode the RION object to its binary representation
    pub fn encode(&self) -> Vec<u8> {
        let mut content = Vec::new();
        for (key, field) in &self.fields {
            // Encode key
            let key_field = RionField::key(key);
            key_field.encode(&mut content).unwrap();
            // Encode field
            field.encode(&mut content).unwrap();
        }
        let content_len = content.len();
        let length_length = content_len.div_ceil(64);
        if length_length > 15 {
            println!("Warning: Object length field is too long, truncating to 15 bytes");
        }
        println!("Content length: {content_len} - Num Bytes {length_length}");
        let length_bytes = content_len.to_be_bytes();
        let mut encoded = Vec::with_capacity(1 + content_len + length_length);
        encoded.push(0xC0 | length_length as u8 & 0x0F);
        // Add only the necessary bytes
        encoded.extend_from_slice(&length_bytes[8 - length_length..]);
        encoded.extend(content);
        encoded
    }

    // // Decode a RION object from its binary representation
}

impl<'a> From<RionObject<'a>> for RionField<'a> {
    fn from(obj: RionObject) -> Self {
        let mut content = Vec::new();
        for (key, field) in &obj.fields {
            let key_field = RionField::key(key);
            key_field.encode(&mut content).unwrap();
            field.encode(&mut content).unwrap();
        }
        RionField::Normal(NormalField {
            field_type: NormalRionType::Object,
            length_length: content.len().div_ceil(64) as u8,
            data: content.into(),
        })
    }
}
