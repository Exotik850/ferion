use super::*;
use ::serde::{Deserialize, Serialize};
use chrono::Utc;

mod rion_field {
    

    use super::*;

    #[test]
    fn test_tiny_field() {
        let field = RionField::Bool(Some(true));
        assert!(matches!(field, RionField::Bool(_)));
        assert!(!field.is_null());
    }

    #[test]
    fn test_empty_utf8_field() {
        let field = RionField::from("");
        assert!(matches!(field, RionField::Normal(_)));
        assert_eq!(field.as_str(), Some(""));
    }

    #[test]
    fn test_short_field() {
        let field = RionField::from("Hello");
        assert!(matches!(field, RionField::Short(_)));
        assert_eq!(field.as_str(), Some("Hello"));
    }

    #[test]
    fn test_normal_field() {
        let long_string = "a".repeat(20);
        let field = RionField::from(long_string.as_str());
        assert!(matches!(field, RionField::Normal(_)));
        assert_eq!(field.as_str(), Some(long_string.as_str()));
    }

    #[test]
    fn test_from_i64() {
        let field = RionField::from(42i64);
        assert!(matches!(field, RionField::Short(_)));
        assert_eq!(field.as_bytes(), &[42]);
    }

    #[test]
    fn test_from_negative_i64() {
        let field = RionField::from(-42i64);
        assert!(matches!(field, RionField::Short(_)));
        assert_eq!(field.as_bytes(), &[41]);
    }

    #[test]
    fn test_from_u64() {
        let field = RionField::from(1000u64);
        assert!(matches!(field, RionField::Short(_)));
        assert_eq!(field.as_bytes(), &[3, 232]);
    }

    #[test]
    fn test_from_bool() {
        let field_true = RionField::from(true);
        let field_false = RionField::from(false);
        assert!(matches!(field_true, RionField::Bool(_)));
        assert!(matches!(field_false, RionField::Bool(_)));
        assert_eq!(field_true.as_bytes(), &[]);
        assert_eq!(field_false.as_bytes(), &[]);
    }

    #[test]
    fn test_from_f32() {
        let field = RionField::from(3.14f32);
        assert!(matches!(field, RionField::Short(_)));
        assert_eq!(field.as_bytes().len(), 4);
    }

    #[test]
    fn test_from_f64() {
        let field = RionField::from(3.14159265359f64);
        assert!(matches!(field, RionField::Short(_)));
        assert_eq!(field.as_bytes().len(), 8);
    }

    #[test]
    fn test_from_datetime() {
        let now = Utc::now();
        let field = RionField::from(now);
        println!("{:?}", field);
        assert!(matches!(field, RionField::Short(_)));
        // assert_eq!(field.as_bytes().len(), 11);
    }

    #[test]
    fn test_encode_decode() {
        let original = RionField::from("Test");
        let mut encoded = Vec::new();
        original.encode(&mut encoded).unwrap();
        let decoded = RionField::from_slice(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_null() {
        let field = RionField::from_slice(&[0x50]).unwrap();
        assert!(field.is_null());
    }
}

mod rion_object {
    use super::*;

    #[test]
    fn test_new_object() {
        let obj = RionObject::new();
        assert!(obj.fields.is_empty());
    }

    #[test]
    fn test_add_field() {
        let mut obj = RionObject::new();
        obj.add_field("key", "value");
        assert_eq!(obj.fields.len(), 1);
        assert!(obj.fields.contains_key("key".as_bytes()));
    }

    #[test]
    fn test_decode_object() {
        let data = vec![
            0xC1, 0x15, 0xE3, 0x01, 0x01, 0x01, 0x22, 0xFF, 0xFF, 0xE3, 0x02, 0x02, 0x02, 0x22,
            0xAB, 0xCD, 0xE3, 0x03, 0x03, 0x03, 0x22, 0x01, 0x23,
        ];
        let obj = RionObject::from_slice(&data).unwrap();
        assert_eq!(obj.fields.len(), 3);
        assert!(obj.fields.contains_key([1, 1, 1].as_ref()));
    }

    #[test]
    fn test_encode_decode_object() {
        let mut obj = RionObject::new();
        obj.add_field("name", "Alice");
        obj.add_field("age", 30i64);
        obj.add_field("is_student", true);

        let encoded = obj.encode();
        println!("{:?}", encoded);
        let decoded_obj = RionObject::from_slice(&encoded).unwrap();

        assert_eq!(obj, decoded_obj);
    }

    #[test]
    fn test_empty_object() {
        let obj = RionObject::new();
        let encoded = obj.encode();
        let decoded_obj = RionObject::from_slice(&encoded).unwrap();
        assert_eq!(obj, decoded_obj);
    }

    #[test]
    fn test_nested_object() {
        let mut inner_obj = RionObject::new();
        inner_obj.add_field("ik", "iv");

        let mut outer_obj = RionObject::new();
        outer_obj.add_field("ok", "ov");
        outer_obj.add_field("n", inner_obj);

        let encoded = outer_obj.encode();
        println!("{:x?}", encoded);
        let decoded_obj = RionObject::from_slice(&encoded).unwrap();

        assert_eq!(outer_obj, decoded_obj);
    }
}

use crate::{from_bytes, to_bytes};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct NumbersStruct {
    u8: u8,
    u16: u16,
    u32: u32,
    u64: u64,
    i8: i8,
    i16: i16,
    i32: i32,
    i64: i64,
    f32: f32,
    f64: f64,
}

#[test]
fn test_numbers() {
    let original = NumbersStruct {
        u8: u8::MAX,
        u16: u16::MAX,
        u32: u32::MAX,
        u64: u64::MAX,
        i8: i8::MIN,
        i16: i16::MIN,
        i32: i32::MIN,
        i64: i64::MIN,
        f32: 1.0,
        f64: 1.0,
    };

    let serialized = to_bytes(&original).unwrap();
    let deserialized: NumbersStruct = from_bytes(&serialized).unwrap();

    assert_eq!(original, deserialized);
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum EnumVariants {
    Unit,
    Tuple(u64),
    Struct { arg: u64 },
}

#[test]
fn test_enums() {
    let variants = vec![
        EnumVariants::Unit,
        EnumVariants::Tuple(42),
        EnumVariants::Struct { arg: 100 },
    ];

    for variant in variants {
        let serialized = to_bytes(&variant).unwrap();
        println!("{variant:?} : {:x?}", serialized);
        let deserialized: EnumVariants = match from_bytes(&serialized) {
            Ok(v) => v,
            Err(e) => {
                panic!("{e}");
            }
        };
        assert_eq!(variant, deserialized);
    }
}

#[test]
fn test_option() {
    let some_value = Some(42u64);
    let none_value: Option<u64> = None;

    let serialized_some = to_bytes(&some_value).unwrap();
    let deserialized_some: Option<u64> = from_bytes(&serialized_some).unwrap();
    assert_eq!(some_value, deserialized_some);

    let serialized_none = to_bytes(&none_value).unwrap();
    let deserialized_none: Option<u64> = from_bytes(&serialized_none).unwrap();
    assert_eq!(none_value, deserialized_none);
}

#[test]
fn test_vectors() {
    let vec_u64 = vec![1, 2, 3, 4];
    let vec_string = vec!["hello".to_string(), "world".to_string()];

    let serialized_u64 = to_bytes(&vec_u64).unwrap();
    let deserialized_u64: Vec<u64> = from_bytes(&serialized_u64).unwrap();
    assert_eq!(vec_u64, deserialized_u64);

    let serialized_string = to_bytes(&vec_string).unwrap();
    let deserialized_string: Vec<String> = from_bytes(&serialized_string).unwrap();
    assert_eq!(vec_string, deserialized_string);
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct NestedStruct {
    field1: u32,
    field2: String,
    field3: Vec<EnumVariants>,
}

#[test]
fn test_nested_structures() {
    let nested = NestedStruct {
        field1: 42,
        field2: "Hello, world!".to_string(),
        field3: vec![
            EnumVariants::Unit,
            EnumVariants::Tuple(100),
            EnumVariants::Struct { arg: 200 },
        ],
    };

    let serialized = to_bytes(&nested).unwrap();
    let deserialized: NestedStruct = from_bytes(&serialized).unwrap();

    assert_eq!(nested, deserialized);
}
