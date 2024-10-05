use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::to_bytes;
use crate::{RionObject, Serializer};

#[test]
fn test_serialize_zero() {
    let value = 0u8;
    let serialized = to_bytes(&value).unwrap();
    assert_eq!(serialized, vec![0x21, 0x00]);
}

#[test]
fn test_serialize_signed_zero() {
    let value = 0i8;
    let serialized = to_bytes(&value).unwrap();
    assert_eq!(serialized, vec![0x21, 0x00]);
}

#[test]
fn test_serialize_negative() {
    let value = -42i8;
    let serialized = to_bytes(&value).unwrap();
    assert_eq!(serialized, vec![0x31, 0x29]);
}

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

#[test]
fn test_nested_object_serialization() {
    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
    struct NestedObject {
        value: Option<Box<NestedObject>>,
        data: String,
    }

    fn create_nested(depth: usize) -> NestedObject {
        if depth == 0 {
            NestedObject {
                value: None,
                data: "leaf".to_string(),
            }
        } else {
            NestedObject {
                value: Some(Box::new(create_nested(depth - 1))),
                data: format!("level {}", depth),
            }
        }
    }

    for depth in 1..=100 {
        let obj = create_nested(depth);
        println!("Serializing depth {}", depth);
        match to_bytes(&obj) {
            Ok(bytes) => match crate::from_bytes::<NestedObject>(&bytes) {
                Ok(decoded) if decoded == obj => println!("Success at depth {depth}"),
                Ok(decoded) => panic!("Failed to deserialize depth {depth}: {decoded:?}"),
                Err(e) => {
                    panic!("Failed to deserialize depth {depth} with {obj:?}: {e:?} {bytes:x?}")
                }
            },
            Err(e) => panic!("Failed to serialize at depth {}: {:?}", depth, e),
        }
    }
}

#[test]
fn test_serialize_empty_object() {
    let obj: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let result = to_bytes(&obj).unwrap();
    assert_eq!(result, vec![0xC0]);
}

#[cfg(feature = "specialization")]
#[test]
fn test_serialize_owned_bytes() {
    let value = b"hello".to_vec();
    let serialized = to_bytes(&value).unwrap();
    assert_eq!(serialized, vec![0x01, 0x05, b'h', b'e', b'l', b'l', b'o']);
}

#[cfg(feature = "specialization")]
#[test]
fn test_serialize_borrowed_bytes() {
    let value = b"hello".as_slice();
    let serialized = to_bytes(&value).unwrap();
    assert_eq!(serialized, vec![0x01, 0x05, b'h', b'e', b'l', b'l', b'o']);
}

#[cfg(feature = "specialization")]
#[test]
fn test_serialize_array_bytes() {
    let value = [b'h', b'e', b'l', b'l', b'o'];
    let serialized = to_bytes(&value).unwrap();
    assert_eq!(serialized, vec![0x01, 0x05, b'h', b'e', b'l', b'l', b'o']);
}

#[cfg(not(feature = "specialization"))]
#[test]
fn test_serialize_vec_bytes() {
    let value = vec![b'h', b'e', b'l', b'l', b'o'];
    let serialized = to_bytes(&value).unwrap();
    assert_eq!(
        serialized,
        vec![0xA1, 0x0A, 0x21, b'h', 0x21, b'e', 0x21, b'l', 0x21, b'l', 0x21, b'o']
    );
}

#[test]
fn test_serialize_mixed_array() {
    #[derive(Serialize)]
    struct MixedType {
        i: u8,
        s: String,
        b: bool,
    }
    let obj = MixedType {
        i: 1,
        s: "abc".to_string(),
        b: true,
    };
    let result = to_bytes(&obj).unwrap();
    assert_eq!(
        result,
        vec![
            0xC1, 0x0D, // Object start
            0xE1, b'i', 0x21, 0x01, // Integer 1
            0xE1, b's', 0x63, b'a', b'b', b'c', // String "abc"
            0xE1, b'b', 0x12, // Boolean true
        ]
    );
}

#[test]
fn test_serialize_negative_integer() {
    let value = -42;
    let serialized = to_bytes(&value).unwrap();
    assert_eq!(serialized, vec![0x31, 0x29]);
}

#[test]
fn test_serialize_nested_objects() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct NestedObj {
        obj: std::collections::HashMap<String, String>,
    }
    let mut inner = std::collections::HashMap::new();
    inner.insert("key".to_string(), "value".to_string());
    let obj = NestedObj { obj: inner };
    let result = to_bytes(&obj).unwrap();
    assert_eq!(
        result,
        vec![
            0xC1, 0x10, // Object start
            0xE3, b'o', b'b', b'j', // Key "obj"
            0xC1, 0x0A, // Nested object start
            0xE3, b'k', b'e', b'y', // Key "key"
            0x65, b'v', b'a', b'l', b'u', b'e', // Value "value"
        ]
    );

    assert_eq!(crate::from_bytes::<NestedObj>(&result).unwrap(), obj);
}

#[test]
fn test_serialize_deeply_nested() {
    #[derive(Serialize, Debug, Deserialize, PartialEq)]
    struct DeepNested {
        a: String,
        b: Option<Box<DeepNested>>,
    }

    let mut nest = DeepNested {
        a: "level 1".to_string(),
        b: None,
    };
    for i in 0..250 {
        nest = DeepNested {
            a: format!("level {}", i + 1),
            b: Some(Box::new(nest)),
        };
    }
    let result = to_bytes(&nest).unwrap();
    let decoded = crate::from_bytes::<DeepNested>(&result).unwrap();
    assert_eq!(decoded, nest);
    // println!("{:?}", result);
}

#[test]
fn test_serialize_primitives() {
    let mut serializer = Serializer::new();

    // Test bool
    true.serialize(&mut serializer).unwrap();
    assert_eq!(serializer.output, vec![0x12]);
    serializer.output.clear();

    false.serialize(&mut serializer).unwrap();
    assert_eq!(serializer.output, vec![0x11]);
    serializer.output.clear();

    // Test integers
    42u8.serialize(&mut serializer).unwrap();
    assert_eq!(serializer.output, vec![0x21, 42]);
    serializer.output.clear();

    1000u16.serialize(&mut serializer).unwrap();
    assert_eq!(serializer.output, vec![0x22, 0x03, 0xE8]);
    serializer.output.clear();

    (-42i8).serialize(&mut serializer).unwrap();
    assert_eq!(serializer.output, vec![0x31, 41]);
    serializer.output.clear();

    // Test floats
    3.14f32.serialize(&mut serializer).unwrap();
    assert_eq!(serializer.output, vec![0x44, 0x40, 0x48, 0xF5, 0xC3]);
    serializer.output.clear();

    0.0f32.serialize(&mut serializer).unwrap();
    assert_eq!(serializer.output, vec![0x44, 0x00, 0x00, 0x00, 0x00]);
    serializer.output.clear();

    // Test char
    'A'.serialize(&mut serializer).unwrap();
    assert_eq!(serializer.output, vec![0x61, 0x41]);
    serializer.output.clear();
}

#[test]
fn test_serialize_strings() {
    let mut serializer = Serializer::new();

    // Short string
    "Hello".serialize(&mut serializer).unwrap();
    assert_eq!(serializer.output, vec![0x65, b'H', b'e', b'l', b'l', b'o']);
    serializer.output.clear();

    // Longer string
    "This is a longer string that should use normal encoding"
        .serialize(&mut serializer)
        .unwrap();
    assert_eq!(serializer.output[0], 0x51); // Normal UTF8 type
    assert_eq!(
        &serializer.output[2..],
        b"This is a longer string that should use normal encoding"
    );
    serializer.output.clear();
}

#[test]
fn test_serialize_option() {
    let mut serializer = Serializer::new();

    // Some value
    Some(42u8).serialize(&mut serializer).unwrap();
    assert_eq!(serializer.output, vec![0x21, 42]);
    serializer.output.clear();

    // None value
    Option::<u8>::None.serialize(&mut serializer).unwrap();
    assert_eq!(serializer.output, vec![0x00]); // TODO Make this have the correct field type
    serializer.output.clear();
}

#[derive(Serialize)]
struct TestStruct {
    field1: u8,
    field2: String,
}

#[test]
fn test_serialize_struct() {
    let mut serializer = Serializer::new();

    let test_struct = TestStruct {
        field1: 42,
        field2: "test".to_string(),
    };

    test_struct.serialize(&mut serializer).unwrap();

    // Check that it's an object
    assert_eq!(serializer.output[0] & 0xF0, 0xC0);

    // Check for field1
    assert!(serializer
        .output
        .windows(9)
        .any(|window| { window == [0xE6, b'f', b'i', b'e', b'l', b'd', b'1', 0x21, 42] }));

    // Check for field2
    assert!(serializer.output.windows(12).any(|window| {
        window
            == [
                0xE6, b'f', b'i', b'e', b'l', b'd', b'2', 0x64, b't', b'e', b's', b't',
            ]
    }));
}

#[derive(Serialize)]
enum TestEnum {
    Unit,
    Tuple(u8, String),
    Struct { x: i32 },
}

#[test]
fn test_serialize_enum() {
    let mut serializer = Serializer::new();

    // Unit variant
    TestEnum::Unit.serialize(&mut serializer).unwrap();
    assert_eq!(serializer.output, vec![0x64, b'U', b'n', b'i', b't']);
    serializer.output.clear();

    // Tuple variant
    TestEnum::Tuple(42, "test".to_string())
        .serialize(&mut serializer)
        .unwrap();
    assert_eq!(serializer.output[0] & 0xF0, 0xC0); // Object
    println!("{:?}", serializer.output);
    assert!(serializer
        .output
        .windows(7)
        .any(|window| { window == [0xE5, b'T', b'u', b'p', b'l', b'e', 0xA1] }));
    serializer.output.clear();

    // Struct variant
    TestEnum::Struct { x: -10 }
        .serialize(&mut serializer)
        .unwrap();
    assert_eq!(serializer.output[0] & 0xF0, 0xC0); // Object
    println!("{:?}", serializer.output);
    assert!(serializer
        .output
        .windows(8)
        .any(|window| { window == [0xE6, b'S', b't', b'r', b'u', b'c', b't', 0xC1] }));
    assert!(serializer
        .output
        .windows(4)
        .any(|window| { window == [0xE1, b'x', 0x31, 9] }));
    serializer.output.clear();
}
