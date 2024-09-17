use super::*;
use chrono::Utc;

mod rion_field {
    use types::LeadByte;

    use super::*;

    #[test]
    fn test_tiny_field() {
        let field = RionField::Tiny(LeadByte(0x11));
        assert!(matches!(field, RionField::Tiny(_)));
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
        assert!(matches!(field_true, RionField::Tiny(_)));
        assert!(matches!(field_false, RionField::Tiny(_)));
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
        original.write(&mut encoded).unwrap();
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
