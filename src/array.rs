use crate::{
    num_needed_length, types::{LeadByte, NormalRionType, RionFieldType}, Result, RionField
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
    byte_len: usize,
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
            byte_len: 0,
        }
    }

    pub(crate) fn parse_rest(mut data: &'a [u8]) -> Result<Self> {
        let mut elements = vec![];
        let byte_len = data.len();
        while data.len() > 0 {
            let (element, rest) = RionField::parse(data)?;
            data = rest;
            elements.push(element);
        }
        Ok(RionArray { elements, byte_len  })
    }

    pub fn from_slice(data: &'a [u8]) -> Result<Self> {
        let (array, rest) = Self::parse(data)?;
        if !rest.is_empty() {
            return Err("Extra data after array".into());
        }
        Ok(array)
    }

    fn parse(data: &'a [u8]) -> Result<(Self, &[u8])> {
        let (lead, length, rest) = crate::get_normal_header(data)?;
        let RionFieldType::Normal(NormalRionType::Array) = lead.field_type() else {
            return Err("Expected a RION array".into());
        };
        // let total = rest.len();
        // let mut elements = Vec::with_capacity(length);
        // while total - rest.len() < length {
        //     let (element, new_rest) = RionField::parse(rest)?;
        //     rest = new_rest;
        //     elements.push(element);
        // }
        let elements = Self::parse_rest(&rest[..length])?;
        Ok((elements, &rest[length..]))

        // Ok((RionArray { elements }, rest))
    }

    pub fn add_element(&mut self, element: impl Into<RionField<'a>>) {
        let element = element.into();
        self.byte_len += element.needed_bytes();
        self.elements.push(element);
    }

    pub(crate) fn write_header(&self, writer: &mut impl std::io::Write) -> Result<()> {
        let length_length = num_needed_length(self.byte_len);
        if length_length > 15 {
            println!("Warning: Array length field is too long, truncating to 15 bytes");
        }
        let lead_byte = LeadByte::from_type(RionFieldType::Normal(NormalRionType::Array), length_length as u8);
        writer.write_all(&[lead_byte.byte()])?;
        writer.write_all(&self.byte_len.to_be_bytes()[usize::BITS as usize / 8 - length_length..])?;
        Ok(())
    }
    // writer.write(&[]);

    pub fn write(&self, writer: &mut impl std::io::Write) -> Result<()> {
        self.write_header(writer)?;
        self.write_body(writer)
    }

    pub(crate) fn write_body(&self, writer: &mut impl std::io::Write) -> Result<()>{
        for element in &self.elements {
            element.write(writer)?;
        }
        Ok(())
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut encoded = Vec::new();
        self.write(&mut encoded).unwrap();
        encoded
    }
}
