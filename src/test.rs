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
        assert!(matches!(field, RionField::Short(_)));
        assert_eq!(field.as_bytes().len(), 11);
    }

    #[test]
    fn test_encode_decode() {
        let original = RionField::from("Test");
        let mut encoded = Vec::new();
        original.encode(&mut encoded).unwrap();
        let decoded = RionField::from_slice(&encoded).unwrap();
        assert_eq!(original, decoded);
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
    fn test_encode_decode_object() {
        let mut obj = RionObject::new();
        obj.add_field("name", "Alice");
        obj.add_field("age", 30i64);
        obj.add_field("is_student", true);

        let encoded = obj.encode();
        // let decoded = RionField::from_slice(&encoded).unwrap();
        let decoded_obj = RionObject::from_slice(&encoded).unwrap();

        assert_eq!(obj, decoded_obj);
    }

    #[test]
    fn test_empty_object() {
        let obj = RionObject::new();
        let encoded = obj.encode();
        // let decoded = RionField::from_slice(&encoded).unwrap();
        let decoded_obj = RionObject::from_slice(&encoded).unwrap();
        assert_eq!(obj, decoded_obj);
    }

    #[test]
    fn test_nested_object() {
        let mut inner_obj = RionObject::new();
        inner_obj.add_field("inner_key", "inner_value");

        let mut outer_obj = RionObject::new();
        outer_obj.add_field("outer_key", "outer_value");
        outer_obj.add_field("nested", inner_obj);

        let encoded = outer_obj.encode();
        // let decoded = RionField::from_slice(&encoded).unwrap();
        let decoded_obj = RionObject::from_slice(&encoded).unwrap();

        assert_eq!(outer_obj, decoded_obj);
    }
}

mod error_handling {
    use field::{NormalField, ShortField};
    use types::{NormalRionType, ShortRionType};

    use super::*;

    #[test]
    fn test_data_too_large_for_short_field() {
        let result = ShortField::read_with_lead(
            Vec::new(),
            ShortRionType::UTF8,
            16,
            &mut std::io::Cursor::new(vec![0; 16]),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_utf8() {
        let invalid_utf8 = vec![0xFF, 0xFF];
        let field = RionField::Normal(NormalField {
            field_type: NormalRionType::UTF8,
            length_length: 1,
            data: invalid_utf8.into(),
        });
        assert_eq!(field.as_str(), None);
    }
}
