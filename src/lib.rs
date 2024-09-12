use std::collections::HashMap;
use std::error::Error;
use std::{borrow::Cow, io::Cursor};
mod field;
pub use field::RionField;
use field::{NormalRionType, RionFieldType};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
// Struct to represent a RION field
// pub struct RionField {
//     field_type: RionFieldType,
//     value: Vec<u8>,
// }

// Struct to represent a RION object
#[derive(Debug, Clone, PartialEq)]
pub struct RionObject<'a> {
    pub fields: HashMap<RionField<'a>, RionField<'a>>,
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

    // Add a field to the RION object
    pub fn add_field(&mut self, key: impl Into<RionField<'a>>, field: impl Into<RionField<'a>>) {
        self.fields.insert(key.into(), field.into());
    }

    // Encode the RION object to its binary representation
    pub fn encode(&self) -> Vec<u8> {
        let mut content = Vec::new();
        for (key, field) in &self.fields {
            // Encode key
            key.encode(&mut content).unwrap();
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

impl<'a> TryFrom<RionField<'a>> for RionObject<'a> {
    type Error = Box<dyn Error>;
    fn try_from(value: RionField<'a>) -> std::result::Result<Self, Self::Error> {
        let RionField::Normal(obj) = value else {
            return Err("Invalid RionField type".into());
        };
        let NormalRionType::Object = obj.field_type else {
            return Err("Invalid RionField type".into());
        };
        let data = obj.data;
        let mut fields = HashMap::new();
        let data_len = data.len() as u64;
        let mut cursor = Cursor::new(data);
        while cursor.position() < data_len {
            let key = RionField::read_from(&mut cursor)?;
            if !key.is_key() {
                return Err("Invalid key field".into());
            }
            let field = RionField::read_from(&mut cursor)?;
            fields.insert(key, field);
        }
        Ok(RionObject { fields })
    }
}
