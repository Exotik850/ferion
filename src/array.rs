use crate::{
    needed_bytes_usize,
    types::{LeadByte, NormalRionType, RionFieldType},
    Result, RionField,
};

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_empty_array() {
        let array = RionArray::new();
        assert!(array.elements.is_empty());
    }

    #[test]
    fn test_add_element() {
        let mut array = RionArray::new();
        array.add_element("value");
        assert_eq!(array.elements.len(), 1);
    }

    #[test]
    fn test_encode_decode_array() {
        let mut array = RionArray::new();
        array.add_element("value1");
        array.add_element("value2");

        let encoded = array.encode();
        println!("{:?}", encoded);
        let decoded_array = RionArray::from_slice(&encoded).unwrap();

        assert_eq!(array, decoded_array);
    }

    #[test]
    fn test_empty_array_encoding() {
        let array = RionArray::new();
        let encoded = array.encode();
        let decoded_array = RionArray::from_slice(&encoded).unwrap();
        assert_eq!(array, decoded_array);
    }
}

#[derive(Debug, PartialEq)]
pub struct RionArray<'a> {
    pub elements: Vec<RionField<'a>>,
}

impl<'a> Default for RionArray<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> RionArray<'a> {
    pub fn new() -> Self {
        RionArray {
            elements: Vec::new(),
        }
    }

    pub fn from_slice(data: &'a [u8]) -> Result<Self> {
        let (array, rest) = Self::parse(data)?;
        if !rest.is_empty() {
            return Err("Extra data after array".into());
        }
        Ok(array)
    }

    fn parse(data: &'a [u8]) -> Result<(Self, &[u8])> {
        let (lead, length, mut rest) = crate::get_normal_header(data)?;
        let RionFieldType::Normal(NormalRionType::Array) = lead.field_type() else {
            return Err("Expected a RION array".into());
        };
        let total = rest.len();
        let mut elements = Vec::with_capacity(length);
        while total - rest.len() < length {
            let (element, new_rest) = RionField::parse(rest)?;
            rest = new_rest;
            elements.push(element);
        }

        Ok((RionArray { elements }, rest))
    }

    pub fn add_element(&mut self, element: impl Into<RionField<'a>>) {
        self.elements.push(element.into());
    }

    pub fn encode(&self) -> Vec<u8> {
        if self.elements.is_empty() {
            return vec![
                LeadByte::from_type(RionFieldType::Normal(NormalRionType::Array), 0).byte(),
            ];
        }

        let mut content = Vec::new();
        for element in &self.elements {
            element.encode(&mut content).unwrap();
        }
        let content_len = content.len();
        // number of bytes needed to encode the length
        let length_length = needed_bytes_usize(content_len);
        if length_length > 15 {
            println!("Warning: Object length field is too long, truncating to 15 bytes");
        }
        println!("Content length: {content_len} - Num Bytes {length_length}");
        let length_bytes = content_len.to_be_bytes();
        let mut encoded = Vec::with_capacity(1 + content_len + length_length);
        encoded.push(0xA0 | length_length as u8 & 0x0F);
        // Add only the necessary bytes
        encoded.extend_from_slice(&length_bytes[8 - length_length..]);
        encoded.extend(content);
        encoded
    }
}
