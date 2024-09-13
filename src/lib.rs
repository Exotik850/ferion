use std::collections::HashMap;
use std::error::Error;
use std::{borrow::Cow, io::Cursor};
mod field;

#[cfg(test)]
mod test;
pub use field::RionField;
use field::{LeadByte, NormalField, NormalRionType, RionFieldType};
use num_bigint::BigUint;

type Result<T> = std::result::Result<T, Box<dyn Error>>;
// Struct to represent a RION field
// pub struct RionField {
//     field_type: RionFieldType,
//     value: Vec<u8>,
// }

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

fn get_lead_byte(data: &[u8]) -> Result<(LeadByte, &[u8])> {
    let Some(lead) = data.get(0) else {
        return Err("Data is empty".into());
    };
    Ok((LeadByte::try_from(*lead)?, &data[1..]))
}

/// Get the header of a RION object
/// Returns the lead byte, the length of the data, and the remaining data
fn get_header(data: &[u8]) -> Result<(LeadByte, usize, &[u8])> {
    let (lead, rest) = get_lead_byte(data)?;
    let length_length = lead.length() as usize;
    let length = BigUint::from_bytes_be(&rest[..length_length]);
    let data_len: usize = length
        .try_into()
        .map_err(|_| "Data too large for this system!")?;
    Ok((lead, data_len, &rest[length_length..]))
}

impl<'a> RionObject<'a> {
    // Create a new RION object
    pub fn new() -> Self {
        RionObject {
            fields: HashMap::new(),
        }
    }

    pub fn from_slice(data: &'a [u8]) -> Result<Self> {
        let (lead, data_len, mut data) = get_header(data)?;
        let RionFieldType::Normal(NormalRionType::Object) = lead.field_type() else {
            return Err("Expected a RION object".into());
        };
        let total = data.len();
        let mut fields = HashMap::new();
        while total - data.len() < data_len {
            let (key, rest) = RionField::parse(data)?;
            if !key.is_key() {
                return Err("Expected a key field".into());
            }
            let (value, rest) = RionField::parse(rest)?;
            data = rest;
            fields.insert(key.to_data().unwrap(), value);
        }
        Ok(RionObject { fields })
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
        // number of bytes needed to encode the length
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

impl From<RionObject<'_>> for RionField<'_> {
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
