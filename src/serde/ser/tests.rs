use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::to_bytes;
use crate::RionObject;

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

    for depth in (1..=100).rev() {
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
