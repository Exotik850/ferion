use std::fmt::Error;

use serde::{
    ser::{
        SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant,
    },
    Serialize,
};

use crate::{
    types::{LeadByte, NormalRionType, RionFieldType, ShortRionType},
    RionField,
};

pub struct Serializer {
    output: Vec<u8>,
}

impl Serializer {
    pub fn new() -> Self {
        Self { output: Vec::new() }
    }

    pub fn serialize_key(&mut self, key: &[u8]) -> Result<(), Error> {
        let field = RionField::key(key);
        Ok(field.encode(&mut self.output).unwrap())
    }
}

pub struct SizedSerializer<'a> {
    output: &'a mut Serializer,
    temp: Serializer,
    initial_len: usize,
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::*;
    use crate::{RionField, RionObject};

    #[test]
    fn test_serialize_bool() {
        let value = true;
        let serialized = to_bytes(&value).unwrap();
        assert_eq!(serialized, vec![0x12]);
    }

    #[test]
    fn test_serialize_object() {
        let mut obj = HashMap::new();
        obj.insert("name", "Alice");
        obj.insert("age", "30");
        let serialized = to_bytes(&obj).unwrap();
        let object = RionObject::from_slice(&serialized).unwrap();

        let mut test_object = RionObject::new();
        test_object.add_field("name", "Alice");
        test_object.add_field("age", "30");

        assert_eq!(object, test_object);
        // println!("{:?}", object);
    }
}

pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>, Error>
where
    T: Serialize,
{
    let mut serializer = Serializer { output: Vec::new() };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

impl<'a> serde::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = SizedSerializer<'a>;
    type SerializeTuple = SizedSerializer<'a>;
    type SerializeTupleStruct = SizedSerializer<'a>;
    type SerializeTupleVariant = SizedSerializer<'a>;
    type SerializeMap = SizedSerializer<'a>;
    type SerializeStruct = SizedSerializer<'a>;
    type SerializeStructVariant = SizedSerializer<'a>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        let field = RionField::bool(v);
        Ok(field.encode(&mut self.output).unwrap())
    }
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        let field = RionField::int64(v);
        Ok(field.encode(&mut self.output).unwrap())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        let field = RionField::uint64(v);
        Ok(field.encode(&mut self.output).unwrap())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        let field = RionField::f32(v);
        Ok(field.encode(&mut self.output).unwrap())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        let field = RionField::f64(v);
        Ok(field.encode(&mut self.output).unwrap())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        let field = RionField::from_str(v);
        Ok(field.encode(&mut self.output).unwrap())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let field = RionField::bytes(v);
        Ok(field.encode(&mut self.output).unwrap())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.output.push(0x00); // Null Bytes field
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SizedSerializer {
            initial_len: self.output.len(),
            temp: Serializer::new(),
            output: self,
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        // if let Some(len) = len {
        //     let num_bytes = len.div_ceil(64);
        //     if num_bytes > 15 {
        //         return Err(Error); // TODO handle error
        //     }
        //     self.output.push(0xC << 4 | num_bytes as u8); // Object lead byte
        //     let ll = len as u64;
        //     let len_bytes = ll.to_be_bytes();
        //     let zeros = len.trailing_zeros() as usize / 8;
        //     self.output.extend_from_slice(dbg!(&len_bytes[zeros..]));
        // }
        Ok(SizedSerializer {
            initial_len: self.output.len(),
            temp: Serializer::new(),
            output: self,
        })
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.serialize_map(Some(len))
    }
}

impl<'a> SerializeTuple for SizedSerializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut self.temp)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        // Array type serialization
        self.finish(0xA)
    }
}

impl<'a> SerializeSeq for SizedSerializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut self.temp)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // Array type serialization
        self.finish(0xA)
    }
}

impl SerializeMap for SizedSerializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        // key.serialize(&mut self.temp)
        let initial_len = self.temp.output.len();
        key.serialize(&mut self.temp)?;
        let lead = self.temp.output[initial_len];
        let lead_byte = LeadByte::try_from(lead).map_err(|_| Error)?;
        // If the first byte is not a Key field, throw an error
        let ft = lead_byte.field_type();
        match ft {
            ft if ft.is_key() => {}
            RionFieldType::Normal(NormalRionType::UTF8) => {
                self.temp.output[initial_len] &= 0x0F;
                self.temp.output[initial_len] |= NormalRionType::Key.to_byte() << 4;
            }
            RionFieldType::Short(ShortRionType::UTF8) => {
                self.temp.output[initial_len] &= 0x0F;
                self.temp.output[initial_len] |= ShortRionType::Key.to_byte() << 4;
            }
            _ => return Err(Error),
        }
        return Ok(());
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut self.temp)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // Object type serialization
        self.finish(0xC)
    }
}

impl<'a> SizedSerializer<'a> {
    fn finish(self, type_byte: u8) -> Result<(), Error> {
        let total_len = self.temp.output.len();
        let len_bytes = total_len.div_ceil(64);
        if len_bytes > 15 {
            return Err(Error); // TODO handle error
        }
        self.output
            .output
            .insert(self.initial_len, type_byte << 4 | len_bytes as u8);
        let ll = total_len as u64;
        let len_bytes = ll.to_be_bytes();
        let zeros = total_len.trailing_zeros().div_ceil(8);
        self.output
            .output
            .extend_from_slice(&len_bytes[8 - zeros as usize..]);
        self.output.output.extend(self.temp.output);
        Ok(())
    }
}

impl SerializeTupleStruct for SizedSerializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut self.temp)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.finish(0xC)
    }
}

impl SerializeStruct for SizedSerializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut self.temp)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.finish(0xC)
    }
}

impl SerializeStructVariant for SizedSerializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut self.temp)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.finish(0xC)
    }
}

impl SerializeTupleVariant for SizedSerializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut self.temp)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.finish(0xA)
    }
}
