use serde::{
    ser::{
        SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant,
    },
    Serialize,
};

use crate::{
    num_needed_length,
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

    pub fn serialize_key(&mut self, key: &[u8]) -> Result<(), SerializeError> {
        let field = RionField::key(key);
        Ok(field.write(&mut self.output).unwrap())
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

    use serde::Deserialize;

    use super::*;
    use crate::{from_bytes, RionObject};

//     #[test]
//     fn test_serialize_bool() {
//         let value = true;
//         let serialized = to_bytes(&value).unwrap();
//         assert_eq!(serialized, vec![0x12]);
//     }

//     #[test]
//     fn test_serialize_object() {
//         let mut obj = HashMap::new();
//         obj.insert("name", "Alice");
//         obj.insert("age", "30");
//         let serialized = to_bytes(&obj).unwrap();
//         println!("{:?}", serialized);
//         let object = RionObject::from_slice(&serialized).unwrap();

//         let mut test_object = RionObject::new();
//         test_object.add_field("name", "Alice");
//         test_object.add_field("age", "30");

//         assert_eq!(object, test_object);
//         // println!("{:?}", object);
//     }

//     #[test]
//     fn test_serialize_numbers() {
//         assert_eq!(to_bytes(&0i8).unwrap(), vec![0x31, 0]);
//         assert_eq!(to_bytes(&127i8).unwrap(), vec![0x31, 127]);
//         assert_eq!(to_bytes(&-128i8).unwrap(), vec![0x41, 127]);
//         assert_eq!(to_bytes(&32767i16).unwrap(), vec![0x32, 127, 255]);
//         assert_eq!(to_bytes(&-32768i16).unwrap(), vec![0x42, 127, 255]);
//         assert_eq!(to_bytes(&2147483647i32).unwrap(), vec![0x34, 127, 255, 255, 255]);
//         assert_eq!(to_bytes(&-2147483648i32).unwrap(), vec![0x44, 127, 255, 255, 255]);
//         assert_eq!(to_bytes(&9223372036854775807i64).unwrap(), vec![0x38, 127, 255, 255, 255, 255, 255, 255, 255]);
//         assert_eq!(to_bytes(&-9223372036854775808i64).unwrap(), vec![0x48, 127, 255, 255, 255, 255, 255, 255, 255]);
//     }

//     #[test]
//     fn test_serialize_floats() {
//         assert_eq!(to_bytes(&0.0f32).unwrap(), vec![0x54, 0, 0, 0, 0]);
//         assert_eq!(to_bytes(&1.0f32).unwrap(), vec![0x54, 63, 128, 0, 0]);
//         assert_eq!(to_bytes(&-1.0f32).unwrap(), vec![0x54, 191, 128, 0, 0]);
//         assert_eq!(to_bytes(&f32::INFINITY).unwrap(), vec![0x54, 127, 128, 0, 0]);
//         assert_eq!(to_bytes(&f32::NEG_INFINITY).unwrap(), vec![0x54, 255, 128, 0, 0]);
//         assert_eq!(to_bytes(&f32::NAN).unwrap(), vec![0x54, 127, 192, 0, 0]);

//         assert_eq!(to_bytes(&0.0f64).unwrap(), vec![0x58, 0, 0, 0, 0, 0, 0, 0, 0]);
//         assert_eq!(to_bytes(&1.0f64).unwrap(), vec![0x58, 63, 240, 0, 0, 0, 0, 0, 0]);
//         assert_eq!(to_bytes(&-1.0f64).unwrap(), vec![0x58, 191, 240, 0, 0, 0, 0, 0, 0]);
//         assert_eq!(to_bytes(&f64::INFINITY).unwrap(), vec![0x58, 127, 240, 0, 0, 0, 0, 0, 0]);
//         assert_eq!(to_bytes(&f64::NEG_INFINITY).unwrap(), vec![0x58, 255, 240, 0, 0, 0, 0, 0, 0]);
//         assert_eq!(to_bytes(&f64::NAN).unwrap(), vec![0x58, 127, 248, 0, 0, 0, 0, 0, 0]);
//     }

//     #[test]
//     fn test_serialize_strings() {
//         assert_eq!(to_bytes(&"").unwrap(), vec![0x60]);
//         assert_eq!(to_bytes(&"Hello").unwrap(), vec![0x65, b'H', b'e', b'l', b'l', b'o']);
//         assert_eq!(to_bytes(&"こんにちは").unwrap(), vec![0x6F, 227, 129, 147, 227, 130, 147, 227, 129, 171, 227, 129, 161, 227, 129, 175]);
//     }

//     #[test]
//     fn test_serialize_bytes() {

//         assert_eq!(to_bytes(&vec![0u8; 0]).unwrap(), vec![0x01, 0x00]);
//         assert_eq!(to_bytes(&vec![1u8, 2u8, 3u8]).unwrap(), vec![0x01, 0x03, 1, 2, 3]);
//         assert_eq!(to_bytes(&vec![0u8; 256]).unwrap(), [vec![0x02, 0x01, 0xFF], vec![0u8; 256]].concat());
//     }

//     #[test]
//     fn test_serialize_option() {
//         assert_eq!(to_bytes(&Option::<u32>::None).unwrap(), vec![0x00]);
//         assert_eq!(to_bytes(&Some(42u32)).unwrap(), vec![0x34, 42, 0, 0, 0]);
//     }

//     #[test]
//     fn test_serialize_unit_and_unit_struct() {
//         #[derive(Serialize)]
//         struct UnitStruct;

//         assert_eq!(to_bytes(&()).unwrap(), vec![0x00]);
//         assert_eq!(to_bytes(&UnitStruct).unwrap(), vec![0x00]);
//     }

//     #[test]
//     fn test_serialize_newtype_struct() {
//         #[derive(Serialize)]
//         struct Wrapper(u32);

//         assert_eq!(to_bytes(&Wrapper(42)).unwrap(), vec![0x34, 42, 0, 0, 0]);
//     }

//     #[test]
//     fn test_serialize_tuple() {
//         assert_eq!(to_bytes(&(1u8, 2u16, 3u32)).unwrap(), vec![0xA3, 0x31, 1, 0x32, 2, 0, 0x34, 3, 0, 0, 0]);
//     }

    #[test]
    fn test_serialize_struct() {
        #[derive(Serialize)]
        struct TestStruct {
            a: u8,
            b: String,
        }

        let test = TestStruct { a: 42, b: "test".to_string() };
        assert_eq!(to_bytes(&test).unwrap(), vec![0xC1, 0x0B, 0xE1, b'a', 0x21, 42, 0xE1, b'b', 0x64, b't', b'e', b's', b't']);
    }

//     #[test]
//     fn test_serialize_enum() {
//         #[derive(Serialize)]
//         enum TestEnum {
//             A,
//             B(u32),
//             C { x: u8, y: u8 },
//         }

//         assert_eq!(to_bytes(&TestEnum::A).unwrap(), vec![0x61, b'A']);
//         assert_eq!(to_bytes(&TestEnum::B(42)).unwrap(), vec![0xC1, 0x61, b'B', 0x34, 42, 0, 0, 0]);
//         assert_eq!(to_bytes(&TestEnum::C { x: 1, y: 2 }).unwrap(),
//                    vec![0xC1, 0x61, b'C', 0xC2, 0x61, b'x', 0x31, 1, 0x61, b'y', 0x31, 2]);
//     }

    #[test]
    fn test_serialize_nested_structures() {
        let mut map = HashMap::new();
        map.insert("key1".to_string(), vec![1, 2, 3]);
        map.insert("key2".to_string(), vec![4, 5, 6]);

        let complex = vec![
            Some(42u32),
            None,
            Some(0u32),
        ];

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct NestedStruct {
            map: HashMap<String, Vec<u8>>,
            complex: Vec<Option<u32>>,
        }

        let nested = NestedStruct { map, complex };
        let serialized = to_bytes(&nested).unwrap();

        println!("{:X?}", serialized);
        // We don't check the exact byte representation due to HashMap's non-deterministic order
        assert!(serialized.len() > 20);
        assert_eq!(serialized[0], 0xC1); // Object with a 1-byte length

        let decoded: NestedStruct = from_bytes(&serialized).unwrap();
        assert_eq!(decoded, nested);
    }

    #[test]
    fn test_serialize_large_data() {
        let large_string = "a".repeat(1000000);
        let serialized = to_bytes(&large_string).unwrap();
        assert_eq!(serialized[0], 0x53); // String with 3-byte length
        assert_eq!(&serialized[1..4], &[0x0F, 0x42, 0x40]); // 1000000 in big-endian
        assert_eq!(serialized.len(), 1000004);
    }

    #[test]
    fn test_serialize_empty_collections() {
        assert_eq!(to_bytes(&Vec::<u8>::new()).unwrap(), vec![0x00]);
        assert_eq!(to_bytes(&HashMap::<String, u32>::new()).unwrap(), vec![0xC0]);
        assert_eq!(to_bytes(&std::collections::BTreeMap::<String,u32>::new()).unwrap(), vec![0xC0]);
    }
}

pub trait RionSerialize {
    fn serialize(&self, serializer: &mut Serializer) -> Result<(), SerializeError>;
}

impl<T: Serialize> RionSerialize for T {
    default fn serialize(&self, serializer: &mut Serializer) -> Result<(), SerializeError> {
        self.serialize(serializer)
    }
}

impl RionSerialize for Vec<u8> {
    fn serialize(&self, serializer: &mut Serializer) -> Result<(), SerializeError> {
        let field = RionField::bytes(self);
        Ok(field.write(&mut serializer.output).unwrap())
    }
}

pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>, SerializeError>
where
    T: RionSerialize,
{
    let mut serializer = Serializer { output: Vec::new() };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

#[derive(Debug)]
pub enum SerializeError {
    IoError(std::io::Error),
    LengthOverflow(usize),
    InvalidType(RionFieldType),
    Custom(String),
}

impl std::fmt::Display for SerializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SerializeError::Custom(msg) => write!(f, "{}", msg),
            SerializeError::InvalidType(ty) => write!(f, "Invalid type: {:?}", ty),
            SerializeError::LengthOverflow(len) => write!(f, "Length overflow: {}", len),
            SerializeError::IoError(err) => write!(f, "IO error: {}", err),
        }
    }
}
impl std::error::Error for SerializeError {}

impl serde::ser::Error for SerializeError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Custom(msg.to_string())
    }
}

impl From<String> for SerializeError {
    fn from(msg: String) -> Self {
        SerializeError::Custom(msg)
    }
}
impl From<Box<dyn std::error::Error>> for SerializeError {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        SerializeError::Custom(value.to_string())
    }
}
impl From<std::io::Error> for SerializeError {
    fn from(err: std::io::Error) -> Self {
        SerializeError::IoError(err)
    }
}

impl<'a> serde::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = SerializeError;
    type SerializeSeq = SizedSerializer<'a>;
    type SerializeTuple = SizedSerializer<'a>;
    type SerializeTupleStruct = SizedSerializer<'a>;
    type SerializeTupleVariant = SizedSerializer<'a>;
    type SerializeMap = SizedSerializer<'a>;
    type SerializeStruct = SizedSerializer<'a>;
    type SerializeStructVariant = SizedSerializer<'a>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        let field = RionField::bool(v);
        Ok(field.write(&mut self.output).unwrap())
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
        Ok(field.write(&mut self.output).unwrap())
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
        Ok(field.write(&mut self.output).unwrap())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        let field = RionField::f32(v);
        Ok(field.write(&mut self.output).unwrap())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        let field = RionField::f64(v);
        Ok(field.write(&mut self.output).unwrap())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        let field = RionField::from_str(v);
        Ok(field.write(&mut self.output).unwrap())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let field = RionField::bytes(v);
        Ok(field.write(&mut self.output).unwrap())
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
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
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
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(SizedSerializer {
            initial_len: self.output.len(),
            temp: Serializer::new(),
            output: self,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.serialize_map(Some(len))
    }
}

impl<'a> SerializeTuple for SizedSerializer<'a> {
    type Ok = ();
    type Error = SerializeError;

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
    type Error = SerializeError;

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
    type Error = SerializeError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        // key.serialize(&mut self.temp)
        let initial_len = self.temp.output.len();
        key.serialize(&mut self.temp)?;
        let lead = self.temp.output[initial_len];
        let lead_byte = LeadByte::try_from(lead)?;
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
            _ => return Err(SerializeError::InvalidType(ft)),
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
    fn finish(self, type_byte: u8) -> Result<(), SerializeError> {
        let total_len = self.temp.output.len();
        let len_bytes = num_needed_length(total_len);
        if len_bytes > 15 {
            return Err(SerializeError::LengthOverflow(len_bytes)); // TODO handle error
        }
        self.output
            .output
            .insert(self.initial_len, type_byte << 4 | len_bytes as u8);
        // let zeros = num_needed_length(total_len);
        let zeros = 8 - len_bytes;
        let len_bytes = (total_len as u64).to_be_bytes();
        self.output
            .output
            .extend_from_slice(&len_bytes[zeros..]);
        self.output.output.extend(self.temp.output);
        Ok(())
    }
    // fn finish(self, type_byte: u8) -> Result<(), SerializeError> {
    //   let total_len = self.temp.output.len();

    //   // Calculate the number of bytes needed to represent the length
    //   let len_bytes = num_needed_length(total_len);
    //   if len_bytes > 15 {
    //       return Err(SerializeError::LengthOverflow(total_len));
    //   }

    //   // Write type byte and length of length
    //   self.output.output.push(type_byte << 4 | len_bytes as u8);

    //   // Write length bytes
    //   if len_bytes > 0 {
    //       self.output.output.extend_from_slice(&(total_len as u64).to_be_bytes()[8 - len_bytes..]);
    //   } else {
    //     println!("No length bytes to write");
    //   }

    //   // Write the actual data
    //   self.output.output.extend(self.temp.output);
    //   Ok(())
    // }
}

impl SerializeTupleStruct for SizedSerializer<'_> {
    type Ok = ();
    type Error = SerializeError;

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
    type Error = SerializeError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.serialize_key(key)?;
        value.serialize(&mut self.temp)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.finish(0xC)
    }
}

impl SerializeStructVariant for SizedSerializer<'_> {
    type Ok = ();
    type Error = SerializeError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.serialize_key(key)?;
        value.serialize(&mut self.temp)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.finish(0xC)
    }
}

impl SerializeTupleVariant for SizedSerializer<'_> {
    type Ok = ();
    type Error = SerializeError;

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
